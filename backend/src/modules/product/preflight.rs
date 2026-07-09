//! Read-only product initialization preflight checks.
//!
//! This module inspects the current tenant state against Product Layer
//! defaults. It never creates, updates, or deletes database records.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::AppError;

use super::defaults::PRODUCT_DEFAULTS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedPreflight {
    pub summary: ProductSeedPreflightSummary,
    pub checks: Vec<ProductSeedCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedPreflightSummary {
    pub total: usize,
    pub exists: usize,
    pub missing: usize,
    pub conflict: usize,
    pub config_only: usize,
    pub manual: usize,
    pub blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductSeedCheck {
    pub key: &'static str,
    pub target: &'static str,
    pub status: ProductSeedCheckStatus,
    pub message: String,
    pub existing_id: Option<Uuid>,
    pub expected: Value,
    pub actual: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductSeedCheckStatus {
    Exists,
    Missing,
    Conflict,
    ConfigOnly,
    Manual,
}

#[derive(Debug, Clone, FromRow)]
struct ApplicationPreflightRecord {
    id: Uuid,
    name: String,
    slug: Option<String>,
    auth_mode: String,
    status: String,
    heartbeat_interval_seconds: i32,
    offline_tolerance_seconds: i32,
    max_devices_default: i32,
}

#[derive(Debug, Clone, FromRow)]
struct ServerApiKeyPreflightRecord {
    id: Uuid,
    name: String,
    status: String,
    scopes: Value,
    expires_at: Option<DateTime<Utc>>,
}

pub async fn build_product_seed_preflight(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<ProductSeedPreflight, AppError> {
    let default_application = find_default_application(pool, tenant_id).await?;
    let mut checks = vec![default_application_check(default_application.as_ref())];
    checks.push(server_api_key_check(pool, tenant_id, default_application.as_ref()).await?);
    checks.push(subscription_plans_check());
    checks.push(ai_billing_check());

    Ok(ProductSeedPreflight {
        summary: summarize(&checks),
        checks,
    })
}

async fn find_default_application(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Option<ApplicationPreflightRecord>, AppError> {
    let access = PRODUCT_DEFAULTS.access;

    sqlx::query_as::<_, ApplicationPreflightRecord>(
        r#"
        select
          id,
          name,
          slug,
          auth_mode,
          status,
          heartbeat_interval_seconds,
          offline_tolerance_seconds,
          max_devices_default
        from applications
        where tenant_id = $1
          and deleted_at is null
          and (
            lower(coalesce(slug, '')) = lower($2)
            or lower(name) = lower($3)
          )
        order by
          case when lower(coalesce(slug, '')) = lower($2) then 0 else 1 end,
          created_at desc,
          id desc
        limit 1
        "#,
    )
    .bind(tenant_id)
    .bind(access.application_slug)
    .bind(access.application_name)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

fn default_application_check(application: Option<&ApplicationPreflightRecord>) -> ProductSeedCheck {
    let expected = expected_application_json();
    let Some(application) = application else {
        return ProductSeedCheck {
            key: "access.default_application",
            target: "default application access config",
            status: ProductSeedCheckStatus::Missing,
            message: "缺少默认接入配置，后续执行初始化时可创建。".to_owned(),
            existing_id: None,
            expected,
            actual: None,
        };
    };

    let actual = application_actual_json(application);
    if application_matches_defaults(application) {
        return ProductSeedCheck {
            key: "access.default_application",
            target: "default application access config",
            status: ProductSeedCheckStatus::Exists,
            message: "默认接入配置已存在，字段与 Product defaults 一致。".to_owned(),
            existing_id: Some(application.id),
            expected,
            actual: Some(actual),
        };
    }

    ProductSeedCheck {
        key: "access.default_application",
        target: "default application access config",
        status: ProductSeedCheckStatus::Conflict,
        message: "发现默认接入配置候选，但 slug、状态或核心字段与 Product defaults 不一致，需要人工确认。".to_owned(),
        existing_id: Some(application.id),
        expected,
        actual: Some(actual),
    }
}

async fn server_api_key_check(
    pool: &PgPool,
    tenant_id: Uuid,
    application: Option<&ApplicationPreflightRecord>,
) -> Result<ProductSeedCheck, AppError> {
    let expected = expected_server_api_key_json();
    let Some(application) = application else {
        return Ok(ProductSeedCheck {
            key: "access.server_api_key",
            target: "server api key scopes",
            status: ProductSeedCheckStatus::Missing,
            message: "默认接入配置不存在，暂无法检查服务端 Key；初始化执行时应先创建接入配置。"
                .to_owned(),
            existing_id: None,
            expected,
            actual: None,
        });
    };

    if !application_matches_defaults(application) {
        return Ok(ProductSeedCheck {
            key: "access.server_api_key",
            target: "server api key scopes",
            status: ProductSeedCheckStatus::Manual,
            message: "默认接入配置存在冲突，服务端 Key 需要等接入配置确认后再核对。".to_owned(),
            existing_id: Some(application.id),
            expected,
            actual: Some(json!({
                "app_id": application.id,
                "app_slug": &application.slug,
                "app_status": &application.status,
            })),
        });
    }

    let keys = list_active_server_api_keys(pool, tenant_id, application.id).await?;
    let access = PRODUCT_DEFAULTS.access;

    if let Some(key) = keys
        .iter()
        .find(|key| scopes_cover(&key.scopes, access.server_api_scopes))
    {
        return Ok(ProductSeedCheck {
            key: "access.server_api_key",
            target: "server api key scopes",
            status: ProductSeedCheckStatus::Exists,
            message: "已存在满足默认 scopes 的 active 服务端 Key；明文 Key 无法回读。".to_owned(),
            existing_id: Some(key.id),
            expected,
            actual: Some(server_api_key_actual_json(key)),
        });
    }

    if let Some(key) = keys
        .iter()
        .find(|key| key.name.as_str() == access.server_api_key_name)
    {
        return Ok(ProductSeedCheck {
            key: "access.server_api_key",
            target: "server api key scopes",
            status: ProductSeedCheckStatus::Conflict,
            message: "同名 active 服务端 Key 已存在，但 scopes 与 Product defaults 不一致。"
                .to_owned(),
            existing_id: Some(key.id),
            expected,
            actual: Some(server_api_key_actual_json(key)),
        });
    }

    Ok(ProductSeedCheck {
        key: "access.server_api_key",
        target: "server api key scopes",
        status: ProductSeedCheckStatus::Missing,
        message: if keys.is_empty() {
            "缺少 active 服务端 Key；后续执行初始化时可创建并只展示一次明文 Key。".to_owned()
        } else {
            "已存在 active 服务端 Key，但缺少满足默认 scopes 的 Key；后续执行可新增。".to_owned()
        },
        existing_id: None,
        expected,
        actual: Some(json!({
            "app_id": application.id,
            "active_key_count": keys.len(),
        })),
    })
}

async fn list_active_server_api_keys(
    pool: &PgPool,
    tenant_id: Uuid,
    app_id: Uuid,
) -> Result<Vec<ServerApiKeyPreflightRecord>, AppError> {
    sqlx::query_as::<_, ServerApiKeyPreflightRecord>(
        r#"
        select
          id,
          name,
          status,
          scopes,
          expires_at
        from server_api_keys
        where tenant_id = $1
          and app_id = $2
          and status = 'active'
          and revoked_at is null
          and (expires_at is null or expires_at > now())
        order by created_at desc, id desc
        "#,
    )
    .bind(tenant_id)
    .bind(app_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

fn subscription_plans_check() -> ProductSeedCheck {
    ProductSeedCheck {
        key: "subscription.plans",
        target: "subscription plan presets",
        status: ProductSeedCheckStatus::Manual,
        message: "当前没有独立套餐目录表，subscriptions 是用户订阅实例；套餐预设需要作为业务配置人工核对。".to_owned(),
        existing_id: None,
        expected: json!({
            "default_subscription_plan": PRODUCT_DEFAULTS.default_subscription_plan,
            "subscription_plans": PRODUCT_DEFAULTS.subscription_plans,
        }),
        actual: None,
    }
}

fn ai_billing_check() -> ProductSeedCheck {
    ProductSeedCheck {
        key: "ai.billing",
        target: "ai wallet and job operation defaults",
        status: ProductSeedCheckStatus::ConfigOnly,
        message: "AI 钱包调整原因和任务操作原因来自 Product defaults，不需要初始化写库。"
            .to_owned(),
        existing_id: None,
        expected: json!(PRODUCT_DEFAULTS.ai_billing),
        actual: None,
    }
}

fn expected_application_json() -> Value {
    let access = PRODUCT_DEFAULTS.access;
    json!({
        "name": access.application_name,
        "slug": access.application_slug,
        "auth_mode": access.auth_mode,
        "status": "active",
        "heartbeat_interval_seconds": access.heartbeat_interval_seconds,
        "offline_tolerance_seconds": access.offline_tolerance_seconds,
        "max_devices_default": access.max_devices_default,
    })
}

fn expected_server_api_key_json() -> Value {
    let access = PRODUCT_DEFAULTS.access;
    json!({
        "app_slug": access.application_slug,
        "name": access.server_api_key_name,
        "scopes": access.server_api_scopes,
        "status": "active",
    })
}

fn application_actual_json(application: &ApplicationPreflightRecord) -> Value {
    json!({
        "name": &application.name,
        "slug": &application.slug,
        "auth_mode": &application.auth_mode,
        "status": &application.status,
        "heartbeat_interval_seconds": application.heartbeat_interval_seconds,
        "offline_tolerance_seconds": application.offline_tolerance_seconds,
        "max_devices_default": application.max_devices_default,
    })
}

fn server_api_key_actual_json(key: &ServerApiKeyPreflightRecord) -> Value {
    json!({
        "name": &key.name,
        "status": &key.status,
        "scopes": &key.scopes,
        "expires_at": key.expires_at,
    })
}

fn application_matches_defaults(application: &ApplicationPreflightRecord) -> bool {
    let access = PRODUCT_DEFAULTS.access;

    application.name.as_str() == access.application_name
        && application.slug.as_deref() == Some(access.application_slug)
        && application.auth_mode.as_str() == access.auth_mode
        && application.status.as_str() == "active"
        && application.heartbeat_interval_seconds == access.heartbeat_interval_seconds
        && application.offline_tolerance_seconds == access.offline_tolerance_seconds
        && application.max_devices_default == access.max_devices_default
}

fn scopes_cover(actual: &Value, expected: &[&str]) -> bool {
    let Some(actual) = actual.as_array() else {
        return false;
    };
    expected.iter().all(|expected_scope| {
        actual
            .iter()
            .filter_map(Value::as_str)
            .any(|actual_scope| actual_scope == *expected_scope)
    })
}

fn summarize(checks: &[ProductSeedCheck]) -> ProductSeedPreflightSummary {
    let mut summary = ProductSeedPreflightSummary {
        total: checks.len(),
        exists: 0,
        missing: 0,
        conflict: 0,
        config_only: 0,
        manual: 0,
        blocked: false,
    };

    for check in checks {
        match check.status {
            ProductSeedCheckStatus::Exists => summary.exists += 1,
            ProductSeedCheckStatus::Missing => summary.missing += 1,
            ProductSeedCheckStatus::Conflict => summary.conflict += 1,
            ProductSeedCheckStatus::ConfigOnly => summary.config_only += 1,
            ProductSeedCheckStatus::Manual => summary.manual += 1,
        }
    }
    summary.blocked = summary.conflict > 0;

    summary
}

fn map_db_error(error: sqlx::Error) -> AppError {
    AppError::dependency(format!("product seed preflight database error: {error}"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        scopes_cover, summarize, ProductSeedCheck, ProductSeedCheckStatus,
        ProductSeedPreflightSummary,
    };

    #[test]
    fn scopes_cover_requires_all_expected_scopes() {
        assert!(scopes_cover(
            &json!(["ai:invoke", "asset:cache"]),
            &["ai:invoke"]
        ));
        assert!(!scopes_cover(&json!(["asset:cache"]), &["ai:invoke"]));
        assert!(!scopes_cover(
            &json!({"scope": "ai:invoke"}),
            &["ai:invoke"]
        ));
    }

    #[test]
    fn summary_counts_statuses_and_marks_conflict_blocked() {
        let checks = vec![
            check(ProductSeedCheckStatus::Exists),
            check(ProductSeedCheckStatus::Missing),
            check(ProductSeedCheckStatus::Conflict),
            check(ProductSeedCheckStatus::ConfigOnly),
            check(ProductSeedCheckStatus::Manual),
        ];

        assert_eq!(
            summarize(&checks),
            ProductSeedPreflightSummary {
                total: 5,
                exists: 1,
                missing: 1,
                conflict: 1,
                config_only: 1,
                manual: 1,
                blocked: true,
            }
        );
    }

    fn check(status: ProductSeedCheckStatus) -> ProductSeedCheck {
        ProductSeedCheck {
            key: "test",
            target: "test",
            status,
            message: "test".to_owned(),
            existing_id: None,
            expected: json!({}),
            actual: None,
        }
    }
}
