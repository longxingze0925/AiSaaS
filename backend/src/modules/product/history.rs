//! Read-only product seed execution history.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Deserialize)]
pub struct ProductSeedExecutionQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedExecutionListResponse {
    pub items: Vec<ProductSeedExecutionRecord>,
    pub meta: ProductSeedExecutionListMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedExecutionListMeta {
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedExecutionRecord {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub actor_name: Option<String>,
    pub request_id: Option<String>,
    pub created_application_id: Option<Uuid>,
    pub existing_application_id: Option<Uuid>,
    pub created_server_api_key_id: Option<Uuid>,
    pub existing_server_api_key_id: Option<Uuid>,
    pub skipped: Vec<ProductSeedExecutionSkippedStep>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedExecutionSkippedStep {
    pub key: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, FromRow)]
struct ProductSeedExecutionRow {
    id: Uuid,
    actor_id: Option<Uuid>,
    actor_email: Option<String>,
    actor_name: Option<String>,
    request_id: Option<String>,
    after_json: Option<Value>,
    created_at: DateTime<Utc>,
}

pub async fn list_product_seed_executions(
    pool: &PgPool,
    tenant_id: Uuid,
    query: &ProductSeedExecutionQuery,
) -> Result<ProductSeedExecutionListResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
    let offset = ((page - 1) * page_size) as i64;
    let limit = page_size as i64;

    let rows = sqlx::query_as::<_, ProductSeedExecutionRow>(
        r#"
        select
          l.id,
          l.actor_id,
          tm.email as actor_email,
          tm.name as actor_name,
          l.request_id,
          l.after_json,
          l.created_at
        from audit_logs l
        left join team_members tm
          on tm.id = l.actor_id
         and tm.tenant_id = l.tenant_id
         and tm.deleted_at is null
        where l.tenant_id = $1
          and l.action = 'product.seed.execute'
          and l.resource_type = 'product_seed'
        order by l.created_at desc, l.id desc
        limit $2 offset $3
        "#,
    )
    .bind(tenant_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    Ok(ProductSeedExecutionListResponse {
        items: rows.into_iter().map(execution_record).collect(),
        meta: ProductSeedExecutionListMeta { page, page_size },
    })
}

fn execution_record(row: ProductSeedExecutionRow) -> ProductSeedExecutionRecord {
    let after_json = row.after_json.as_ref();

    ProductSeedExecutionRecord {
        id: row.id,
        actor_id: row.actor_id,
        actor_email: row.actor_email,
        actor_name: row.actor_name,
        request_id: row.request_id,
        created_application_id: uuid_field(after_json, "created_application_id"),
        existing_application_id: uuid_field(after_json, "existing_application_id"),
        created_server_api_key_id: uuid_field(after_json, "created_server_api_key_id"),
        existing_server_api_key_id: uuid_field(after_json, "existing_server_api_key_id"),
        skipped: skipped_steps(after_json),
        created_at: row.created_at,
    }
}

fn uuid_field(value: Option<&Value>, key: &str) -> Option<Uuid> {
    value?
        .get(key)?
        .as_str()
        .and_then(|value| Uuid::parse_str(value).ok())
}

fn skipped_steps(value: Option<&Value>) -> Vec<ProductSeedExecutionSkippedStep> {
    value
        .and_then(|value| value.get("skipped"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    Some(ProductSeedExecutionSkippedStep {
                        key: item.get("key")?.as_str()?.to_owned(),
                        status: item.get("status")?.as_str()?.to_owned(),
                        message: item.get("message")?.as_str()?.to_owned(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn map_db_error(error: sqlx::Error) -> AppError {
    AppError::dependency(format!(
        "product seed execution history database error: {error}"
    ))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    use super::{execution_record, ProductSeedExecutionRow};

    #[test]
    fn execution_record_extracts_safe_audit_summary() {
        let created_application_id = Uuid::new_v4();
        let created_server_api_key_id = Uuid::new_v4();
        let row = ProductSeedExecutionRow {
            id: Uuid::new_v4(),
            actor_id: Some(Uuid::new_v4()),
            actor_email: Some("owner@example.com".to_owned()),
            actor_name: Some("Owner".to_owned()),
            request_id: Some("req".to_owned()),
            after_json: Some(json!({
                "created_application_id": created_application_id,
                "created_server_api_key_id": created_server_api_key_id,
                "skipped": [
                    {
                        "key": "subscription.plans",
                        "status": "manual",
                        "message": "manual"
                    }
                ]
            })),
            created_at: Utc::now(),
        };

        let record = execution_record(row);

        assert_eq!(record.created_application_id, Some(created_application_id));
        assert_eq!(
            record.created_server_api_key_id,
            Some(created_server_api_key_id)
        );
        assert_eq!(record.skipped[0].key, "subscription.plans");
        assert_eq!(record.actor_email.as_deref(), Some("owner@example.com"));
    }
}
