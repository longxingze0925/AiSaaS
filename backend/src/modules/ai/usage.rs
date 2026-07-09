use axum::{
    extract::{Query, State},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    error::{ApiResponse, AppError},
    http::request_id::RequestId,
    modules::auth::session::AdminContext,
    state::AppState,
};

const DEFAULT_PAGE_SIZE: i64 = 20;
const MAX_PAGE_SIZE: i64 = 100;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AiUsageRecord {
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub provider_name: Option<String>,
    pub model_code: Option<String>,
    pub request_id: Option<String>,
    pub endpoint: String,
    pub status: String,
    pub provider_status: Option<String>,
    pub provider_request_id: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    pub charged_minor: i64,
    pub refunded_minor: i64,
    pub provider_cost_minor: Option<i64>,
    pub currency: String,
    pub price_snapshot: Value,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct AiUsageRecordListResponse {
    pub items: Vec<AiUsageRecord>,
    pub meta: ListMeta,
}

#[derive(Debug, Serialize)]
pub struct ListMeta {
    pub page: i64,
    pub page_size: i64,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
pub struct AiUsageRecordListQuery {
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_query: Option<String>,
    pub provider_id: Option<Uuid>,
    pub model_code: Option<String>,
    pub endpoint: Option<String>,
    pub request_id: Option<String>,
    pub provider_request_id: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

struct UsageRecordFilters<'a> {
    status: Option<&'a str>,
    customer_id: Option<Uuid>,
    customer_query: Option<&'a str>,
    provider_id: Option<Uuid>,
    model_code: Option<&'a str>,
    endpoint: Option<&'a str>,
    request_id: Option<&'a str>,
    provider_request_id: Option<&'a str>,
}

pub async fn list_ai_usage_records(
    State(state): State<AppState>,
    Extension(admin): Extension<AdminContext>,
    Extension(request_id): Extension<RequestId>,
    Query(query): Query<AiUsageRecordListQuery>,
) -> Result<Json<ApiResponse<AiUsageRecordListResponse>>, AppError> {
    ensure_admin_permission(&admin, "ai:read")?;
    let status = normalize_optional_status(query.status.as_deref())?;
    let customer_query =
        normalize_optional_filter(query.customer_query.as_deref(), "customer query", 120)?;
    let model_code = normalize_optional_filter(query.model_code.as_deref(), "model code", 120)?;
    let endpoint = normalize_optional_filter(query.endpoint.as_deref(), "endpoint", 200)?;
    let request_id_filter =
        normalize_optional_filter(query.request_id.as_deref(), "request id", 160)?;
    let provider_request_id_filter = normalize_optional_filter(
        query.provider_request_id.as_deref(),
        "provider request id",
        160,
    )?;
    let (page, page_size) = normalize_page(query.page, query.page_size);
    let filters = UsageRecordFilters {
        status: status.as_deref(),
        customer_id: query.customer_id,
        customer_query: customer_query.as_deref(),
        provider_id: query.provider_id,
        model_code: model_code.as_deref(),
        endpoint: endpoint.as_deref(),
        request_id: request_id_filter.as_deref(),
        provider_request_id: provider_request_id_filter.as_deref(),
    };
    let items = list_usage_records(&state, admin.tenant_id, filters, page, page_size).await?;
    let has_more = items.len() > page_size as usize;
    let items = items.into_iter().take(page_size as usize).collect();

    Ok(Json(ApiResponse::ok(
        AiUsageRecordListResponse {
            items,
            meta: ListMeta {
                page,
                page_size,
                has_more,
            },
        },
        request_id.to_string(),
    )))
}

async fn list_usage_records(
    state: &AppState,
    tenant_id: Uuid,
    filters: UsageRecordFilters<'_>,
    page: i64,
    page_size: i64,
) -> Result<Vec<AiUsageRecord>, AppError> {
    let offset = (page - 1) * page_size;
    let fetch_limit = page_size + 1;

    sqlx::query_as::<_, AiUsageRecord>(
        r#"
        select
          u.id,
          u.customer_id,
          c.email as customer_email,
          c.name as customer_name,
          p.name as provider_name,
          m.code as model_code,
          u.request_id,
          u.endpoint,
          u.status,
          u.provider_status,
          u.provider_request_id,
          u.prompt_tokens,
          u.completion_tokens,
          u.total_tokens,
          u.charged_minor,
          u.refunded_minor,
          u.provider_cost_minor,
          coalesce(u.price_snapshot_json->>'currency', m.currency, 'CNY') as currency,
          u.price_snapshot_json as price_snapshot,
          u.metadata_json as metadata,
          u.created_at,
          u.completed_at
        from ai_usage_records u
        left join customers c
          on c.id = u.customer_id
          and c.tenant_id = u.tenant_id
        left join ai_providers p
          on p.id = u.provider_id
          and p.tenant_id = u.tenant_id
        left join ai_models m
          on m.id = u.model_id
          and m.tenant_id = u.tenant_id
        where u.tenant_id = $1
          and ($2::text is null or u.status = $2)
          and ($3::uuid is null or u.customer_id = $3)
          and (
            $4::text is null
            or c.email ilike '%' || $4 || '%'
            or c.name ilike '%' || $4 || '%'
            or u.customer_id::text = $4
          )
          and ($5::uuid is null or u.provider_id = $5)
          and ($6::text is null or m.code = $6)
          and ($7::text is null or u.endpoint = $7)
          and ($8::text is null or u.request_id = $8)
          and ($9::text is null or u.provider_request_id = $9)
        order by u.created_at desc, u.id desc
        limit $10 offset $11
        "#,
    )
    .bind(tenant_id)
    .bind(filters.status)
    .bind(filters.customer_id)
    .bind(filters.customer_query)
    .bind(filters.provider_id)
    .bind(filters.model_code)
    .bind(filters.endpoint)
    .bind(filters.request_id)
    .bind(filters.provider_request_id)
    .bind(fetch_limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(map_db_error)
}

fn normalize_status(value: &str) -> Result<String, AppError> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "pending" | "running" | "succeeded" | "failed" | "refunded" => Ok(value),
        _ => Err(AppError::validation_failed("ai usage status is invalid")),
    }
}

fn normalize_optional_status(value: Option<&str>) -> Result<Option<String>, AppError> {
    match clean_optional_text(value) {
        Some(value) => normalize_status(&value).map(Some),
        None => Ok(None),
    }
}

fn normalize_optional_filter(
    value: Option<&str>,
    label: &str,
    max_len: usize,
) -> Result<Option<String>, AppError> {
    let Some(value) = clean_optional_text(value) else {
        return Ok(None);
    };
    if value.chars().count() > max_len {
        return Err(AppError::validation_failed(format!(
            "ai usage {label} is too long"
        )));
    }

    Ok(Some(value))
}

fn clean_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn normalize_page(page: Option<i64>, page_size: Option<i64>) -> (i64, i64) {
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);

    (page, page_size)
}

fn ensure_admin_permission(admin: &AdminContext, permission_code: &str) -> Result<(), AppError> {
    if admin
        .permissions
        .iter()
        .any(|permission| permission == permission_code)
    {
        return Ok(());
    }

    Err(AppError::forbidden(format!(
        "missing permission: {permission_code}"
    )))
}

fn map_db_error(error: sqlx::Error) -> AppError {
    AppError::dependency(format!("ai usage database error: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_optional_filter, normalize_optional_status, normalize_page, normalize_status,
    };

    #[test]
    fn usage_status_is_validated() {
        assert_eq!(normalize_status("Succeeded").expect("status"), "succeeded");
        assert!(normalize_status("unknown").is_err());
    }

    #[test]
    fn optional_usage_status_ignores_empty_text() {
        assert_eq!(normalize_optional_status(Some("  ")).expect("status"), None);
        assert_eq!(
            normalize_optional_status(Some("FAILED")).expect("status"),
            Some("failed".to_owned())
        );
    }

    #[test]
    fn optional_usage_filter_is_trimmed_and_bounded() {
        assert_eq!(
            normalize_optional_filter(Some(" req_1 "), "request id", 16).expect("filter"),
            Some("req_1".to_owned())
        );
        assert!(normalize_optional_filter(Some("12345"), "request id", 4).is_err());
    }

    #[test]
    fn page_size_is_bounded() {
        assert_eq!(normalize_page(Some(0), Some(500)), (1, 100));
    }
}
