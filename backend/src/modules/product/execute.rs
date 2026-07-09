//! Controlled product initialization execution.
//!
//! The executor only creates resources that can be safely identified and
//! reconciled by Product Layer defaults. It does not create subscription plan
//! catalogs or AI billing rows because those are currently config-only/manual
//! concerns in this codebase.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    crypto::{
        envelope::encrypt_private_key,
        signing::generate_ed25519_key,
        token::{generate_token, hash_token},
    },
    error::AppError,
    http::request_id::RequestId,
    modules::{
        application::{
            model::{
                Application, ApplicationSummary, CreateApplicationInput, NewApplication,
                NewSigningKey, SigningKey,
            },
            repository::{create_application_in_transaction, create_signing_key_in_transaction},
        },
        audit::{self, AuditLogInput},
        auth::session::AdminContext,
        product::preflight::{self, ProductSeedPreflight},
    },
    state::AppState,
};

use super::defaults::PRODUCT_DEFAULTS;

const PRODUCT_SEED_CONFIRMATION: &str = "INITIALIZE_PRODUCT";
const SERVER_API_KEY_PREFIX: &str = "aissk_";
const SERVER_API_KEY_DISPLAY_PREFIX_LEN: usize = 18;

#[derive(Debug, Deserialize)]
pub struct ProductSeedExecuteRequest {
    pub confirm: String,
    pub create_server_api_key: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ProductSeedExecuteResponse {
    pub result: ProductSeedExecuteResult,
    pub preflight: ProductSeedPreflight,
}

#[derive(Debug, Serialize)]
pub struct ProductSeedExecuteResult {
    pub created_application: Option<ProductSeedApplicationSecret>,
    pub existing_application_id: Option<Uuid>,
    pub created_server_api_key: Option<ProductSeedServerApiKeySecret>,
    pub existing_server_api_key_id: Option<Uuid>,
    pub skipped: Vec<ProductSeedExecuteSkippedStep>,
}

#[derive(Debug, Serialize)]
pub struct ProductSeedApplicationSecret {
    pub id: Uuid,
    pub app_key: String,
    pub app_secret: String,
    pub signing_key: ProductSeedSigningKey,
    pub application: ApplicationSummary,
}

#[derive(Debug, Serialize)]
pub struct ProductSeedSigningKey {
    pub id: Uuid,
    pub kid: String,
    pub key_scope: String,
    pub alg: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ProductSeedServerApiKeySecret {
    pub id: Uuid,
    pub app_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub plain_key: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductSeedExecuteSkippedStep {
    pub key: &'static str,
    pub status: &'static str,
    pub message: &'static str,
}

#[derive(Debug, Clone, FromRow)]
struct ServerApiKeySeedRecord {
    id: Uuid,
    name: String,
    scopes: Value,
}

pub async fn execute_product_seed(
    state: &AppState,
    admin: &AdminContext,
    request_id: &RequestId,
    payload: ProductSeedExecuteRequest,
) -> Result<ProductSeedExecuteResponse, AppError> {
    validate_execute_request(&payload)?;

    let preflight_before =
        preflight::build_product_seed_preflight(&state.db, admin.tenant_id).await?;
    if preflight_before.summary.blocked {
        return Err(AppError::conflict(
            "product seed preflight has conflicts; resolve them before execution",
        ));
    }

    let mut transaction = state.db.begin().await.map_err(map_db_error)?;
    let mut result = ProductSeedExecuteResult {
        created_application: None,
        existing_application_id: None,
        created_server_api_key: None,
        existing_server_api_key_id: None,
        skipped: vec![subscription_plans_skipped(), ai_billing_skipped()],
    };

    let application =
        match find_seed_application_for_update(&mut transaction, admin.tenant_id).await? {
            Some(application) => {
                ensure_application_matches_defaults(&application)?;
                result.existing_application_id = Some(application.id);
                application
            }
            None => {
                let (application, app_secret, signing_key) =
                    create_seed_application(&mut transaction, state, admin).await?;
                audit_application_create(
                    &mut transaction,
                    admin,
                    request_id,
                    &application,
                    &signing_key,
                )
                .await?;
                result.created_application = Some(ProductSeedApplicationSecret {
                    id: application.id,
                    app_key: application.app_key.clone(),
                    app_secret,
                    signing_key: signing_key_result(&signing_key),
                    application: application.clone().into(),
                });
                application
            }
        };

    if payload.create_server_api_key.unwrap_or(true) {
        seed_server_api_key(
            &mut transaction,
            state,
            admin,
            request_id,
            application.id,
            &mut result,
        )
        .await?;
    } else {
        result.skipped.push(ProductSeedExecuteSkippedStep {
            key: "access.server_api_key",
            status: "skipped",
            message: "调用方选择跳过服务端 Key 创建。",
        });
    }

    audit_seed_execution(&mut transaction, admin, request_id, &result).await?;
    transaction.commit().await.map_err(map_db_error)?;

    let preflight = preflight::build_product_seed_preflight(&state.db, admin.tenant_id).await?;

    Ok(ProductSeedExecuteResponse { result, preflight })
}

pub fn seed_confirmation_phrase() -> &'static str {
    PRODUCT_SEED_CONFIRMATION
}

fn validate_execute_request(payload: &ProductSeedExecuteRequest) -> Result<(), AppError> {
    if payload.confirm.trim() != PRODUCT_SEED_CONFIRMATION {
        return Err(AppError::validation_failed(format!(
            "confirm must be {PRODUCT_SEED_CONFIRMATION}"
        )));
    }

    Ok(())
}

async fn find_seed_application_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
) -> Result<Option<Application>, AppError> {
    let access = PRODUCT_DEFAULTS.access;

    sqlx::query_as::<_, Application>(
        r#"
        select
          id,
          tenant_id,
          name,
          slug,
          app_key,
          app_secret_hash,
          auth_mode,
          status,
          heartbeat_interval_seconds,
          offline_tolerance_seconds,
          max_devices_default,
          metadata,
          created_at,
          updated_at,
          deleted_at
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
        for update
        "#,
    )
    .bind(tenant_id)
    .bind(access.application_slug)
    .bind(access.application_name)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(map_db_error)
}

fn ensure_application_matches_defaults(application: &Application) -> Result<(), AppError> {
    if application_matches_defaults(application) {
        return Ok(());
    }

    Err(AppError::conflict(
        "default application exists but does not match product defaults",
    ))
}

async fn create_seed_application(
    transaction: &mut Transaction<'_, Postgres>,
    state: &AppState,
    admin: &AdminContext,
) -> Result<(Application, String, SigningKey), AppError> {
    let access = PRODUCT_DEFAULTS.access;
    let app_key = generate_app_key();
    let app_secret = generate_token();
    let app_secret_hash = hash_token(&state.config.security.token_hash_pepper, &app_secret)?;
    let application_input = NewApplication::from_input(
        admin.tenant_id,
        CreateApplicationInput {
            name: access.application_name.to_owned(),
            slug: Some(access.application_slug.to_owned()),
            auth_mode: Some(access.auth_mode.to_owned()),
            heartbeat_interval_seconds: Some(access.heartbeat_interval_seconds),
            offline_tolerance_seconds: Some(access.offline_tolerance_seconds),
            max_devices_default: Some(access.max_devices_default),
            metadata: Some(json!({
                "product_seed": {
                    "version": 1,
                    "idempotency_key": "product:access:default_application",
                },
            })),
        },
        app_key,
        app_secret_hash,
    )?;
    let generated_signing_key = generate_ed25519_key()?;
    let private_key_envelope_json = encrypted_private_key_json(
        &state.config.security.master_key,
        &generated_signing_key.private_key_pkcs8_der,
    )?;

    let application = create_application_in_transaction(transaction, application_input).await?;
    let signing_key = create_signing_key_in_transaction(
        transaction,
        NewSigningKey::app_request(
            admin.tenant_id,
            application.id,
            generated_signing_key.kid,
            generated_signing_key.public_key_pem,
            private_key_envelope_json,
            admin.team_member_id,
        ),
    )
    .await?;

    Ok((application, app_secret, signing_key))
}

async fn seed_server_api_key(
    transaction: &mut Transaction<'_, Postgres>,
    state: &AppState,
    admin: &AdminContext,
    request_id: &RequestId,
    app_id: Uuid,
    result: &mut ProductSeedExecuteResult,
) -> Result<(), AppError> {
    let access = PRODUCT_DEFAULTS.access;
    let active_keys = list_active_server_api_keys(transaction, admin.tenant_id, app_id).await?;

    if let Some(key) = active_keys
        .iter()
        .find(|key| scopes_cover(&key.scopes, access.server_api_scopes))
    {
        result.existing_server_api_key_id = Some(key.id);
        return Ok(());
    }

    if active_keys
        .iter()
        .any(|key| key.name.as_str() == access.server_api_key_name)
    {
        return Err(AppError::conflict(
            "same-name server api key exists but scopes do not match product defaults",
        ));
    }

    let scopes = access
        .server_api_scopes
        .iter()
        .map(|scope| (*scope).to_owned())
        .collect::<Vec<_>>();
    let plain_key = generate_server_api_key();
    let key_prefix = display_prefix(&plain_key);
    let key_hash = hash_token(&state.config.security.token_hash_pepper, &plain_key)?;
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        insert into server_api_keys (
          id,
          tenant_id,
          app_id,
          name,
          key_prefix,
          key_hash,
          scopes,
          expires_at,
          created_by
        )
        values ($1, $2, $3, $4, $5, $6, $7, null, $8)
        "#,
    )
    .bind(id)
    .bind(admin.tenant_id)
    .bind(app_id)
    .bind(access.server_api_key_name)
    .bind(&key_prefix)
    .bind(key_hash)
    .bind(json!(&scopes))
    .bind(admin.team_member_id)
    .execute(&mut **transaction)
    .await
    .map_err(map_db_error)?;

    audit_server_api_key_create(
        transaction,
        admin,
        request_id,
        id,
        app_id,
        access.server_api_key_name,
        &key_prefix,
        &scopes,
    )
    .await?;

    result.created_server_api_key = Some(ProductSeedServerApiKeySecret {
        id,
        app_id,
        name: access.server_api_key_name.to_owned(),
        key_prefix,
        plain_key,
        scopes,
    });

    Ok(())
}

async fn list_active_server_api_keys(
    transaction: &mut Transaction<'_, Postgres>,
    tenant_id: Uuid,
    app_id: Uuid,
) -> Result<Vec<ServerApiKeySeedRecord>, AppError> {
    sqlx::query_as::<_, ServerApiKeySeedRecord>(
        r#"
        select
          id,
          name,
          scopes
        from server_api_keys
        where tenant_id = $1
          and app_id = $2
          and status = 'active'
          and revoked_at is null
          and (expires_at is null or expires_at > now())
        order by created_at desc, id desc
        for update
        "#,
    )
    .bind(tenant_id)
    .bind(app_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(map_db_error)
}

fn subscription_plans_skipped() -> ProductSeedExecuteSkippedStep {
    ProductSeedExecuteSkippedStep {
        key: "subscription.plans",
        status: "manual",
        message: "当前没有独立套餐目录表，套餐预设需要作为业务配置人工核对。",
    }
}

fn ai_billing_skipped() -> ProductSeedExecuteSkippedStep {
    ProductSeedExecuteSkippedStep {
        key: "ai.billing",
        status: "config_only",
        message: "AI 钱包和任务操作默认值来自 Product defaults，不需要初始化写库。",
    }
}

fn application_matches_defaults(application: &Application) -> bool {
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

fn generate_app_key() -> String {
    format!("app_{}", generate_token())
}

fn generate_server_api_key() -> String {
    format!("{SERVER_API_KEY_PREFIX}{}", generate_token())
}

fn display_prefix(key: &str) -> String {
    key.chars()
        .take(SERVER_API_KEY_DISPLAY_PREFIX_LEN)
        .collect()
}

fn encrypted_private_key_json(
    master_key: &[u8; 32],
    private_key: &[u8],
) -> Result<Value, AppError> {
    let private_key_envelope = encrypt_private_key(master_key, private_key)?;

    serde_json::to_value(private_key_envelope).map_err(|error| {
        AppError::crypto(format!(
            "private key envelope serialization failed: {error}"
        ))
    })
}

fn signing_key_result(signing_key: &SigningKey) -> ProductSeedSigningKey {
    ProductSeedSigningKey {
        id: signing_key.id,
        kid: signing_key.kid.clone(),
        key_scope: signing_key.key_scope.clone(),
        alg: signing_key.alg.clone(),
        status: signing_key.status.clone(),
    }
}

async fn audit_application_create(
    transaction: &mut Transaction<'_, Postgres>,
    admin: &AdminContext,
    request_id: &RequestId,
    application: &Application,
    signing_key: &SigningKey,
) -> Result<(), AppError> {
    audit::record(
        transaction,
        AuditLogInput {
            tenant_id: Some(admin.tenant_id),
            actor_type: "team_member",
            actor_id: Some(admin.team_member_id),
            action: "application.create",
            resource_type: "application",
            resource_id: Some(application.id),
            ip: None,
            user_agent: None,
            request_id: Some(request_id.to_string()),
            before_json: None,
            after_json: Some(json!({
                "id": application.id,
                "name": &application.name,
                "slug": &application.slug,
                "app_key": &application.app_key,
                "auth_mode": &application.auth_mode,
                "status": &application.status,
                "heartbeat_interval_seconds": application.heartbeat_interval_seconds,
                "offline_tolerance_seconds": application.offline_tolerance_seconds,
                "max_devices_default": application.max_devices_default,
                "metadata": &application.metadata,
                "signing_key": {
                    "id": signing_key.id,
                    "kid": &signing_key.kid,
                    "key_scope": &signing_key.key_scope,
                    "alg": &signing_key.alg,
                    "status": &signing_key.status,
                },
            })),
            metadata_json: json!({
                "product_seed": true,
            }),
        },
    )
    .await
}

async fn audit_server_api_key_create(
    transaction: &mut Transaction<'_, Postgres>,
    admin: &AdminContext,
    request_id: &RequestId,
    id: Uuid,
    app_id: Uuid,
    name: &str,
    key_prefix: &str,
    scopes: &[String],
) -> Result<(), AppError> {
    audit::record(
        transaction,
        AuditLogInput {
            tenant_id: Some(admin.tenant_id),
            actor_type: "team_member",
            actor_id: Some(admin.team_member_id),
            action: "server_api_key.create",
            resource_type: "server_api_key",
            resource_id: Some(id),
            ip: None,
            user_agent: None,
            request_id: Some(request_id.to_string()),
            before_json: None,
            after_json: Some(json!({
                "id": id,
                "app_id": app_id,
                "name": name,
                "key_prefix": key_prefix,
                "scopes": scopes,
                "expires_at": null,
            })),
            metadata_json: json!({
                "app_id": app_id,
                "key_prefix": key_prefix,
                "product_seed": true,
            }),
        },
    )
    .await
}

async fn audit_seed_execution(
    transaction: &mut Transaction<'_, Postgres>,
    admin: &AdminContext,
    request_id: &RequestId,
    result: &ProductSeedExecuteResult,
) -> Result<(), AppError> {
    audit::record(
        transaction,
        AuditLogInput {
            tenant_id: Some(admin.tenant_id),
            actor_type: "team_member",
            actor_id: Some(admin.team_member_id),
            action: "product.seed.execute",
            resource_type: "product_seed",
            resource_id: None,
            ip: None,
            user_agent: None,
            request_id: Some(request_id.to_string()),
            before_json: None,
            after_json: Some(json!({
                "created_application_id": result.created_application.as_ref().map(|item| item.id),
                "existing_application_id": result.existing_application_id,
                "created_server_api_key_id": result.created_server_api_key.as_ref().map(|item| item.id),
                "existing_server_api_key_id": result.existing_server_api_key_id,
                "skipped": &result.skipped,
            })),
            metadata_json: json!({
                "confirmation": PRODUCT_SEED_CONFIRMATION,
            }),
        },
    )
    .await
}

fn map_db_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.code().as_deref() == Some("23505") {
            return AppError::conflict("product seed resource already exists");
        }
    }

    AppError::dependency(format!("product seed execution database error: {error}"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        display_prefix, generate_server_api_key, scopes_cover, validate_execute_request,
        ProductSeedExecuteRequest, PRODUCT_SEED_CONFIRMATION, SERVER_API_KEY_DISPLAY_PREFIX_LEN,
    };

    #[test]
    fn execute_request_requires_confirmation_phrase() {
        assert!(validate_execute_request(&ProductSeedExecuteRequest {
            confirm: PRODUCT_SEED_CONFIRMATION.to_owned(),
            create_server_api_key: None,
        })
        .is_ok());

        assert!(validate_execute_request(&ProductSeedExecuteRequest {
            confirm: "yes".to_owned(),
            create_server_api_key: None,
        })
        .is_err());
    }

    #[test]
    fn generated_server_api_key_uses_public_prefix_and_display_prefix() {
        let key = generate_server_api_key();

        assert!(key.starts_with("aissk_"));
        assert_eq!(
            display_prefix(&key).chars().count(),
            SERVER_API_KEY_DISPLAY_PREFIX_LEN
        );
    }

    #[test]
    fn scopes_cover_requires_expected_scopes() {
        assert!(scopes_cover(&json!(["ai:invoke"]), &["ai:invoke"]));
        assert!(!scopes_cover(&json!(["asset:cache"]), &["ai:invoke"]));
    }
}
