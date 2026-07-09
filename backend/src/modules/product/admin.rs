use axum::{
    extract::{Query, State},
    Extension, Json,
};
use serde::Serialize;

use crate::{
    error::{ApiResponse, AppError},
    http::request_id::RequestId,
    modules::{
        auth::session::AdminContext,
        product::{execute, history, preflight, seed_plan},
    },
    state::AppState,
};

#[derive(Debug, Serialize)]
pub struct ProductSeedPlanResponse {
    pub plan: seed_plan::ProductSeedPlan,
    pub preflight: preflight::ProductSeedPreflight,
}

#[derive(Debug, Serialize)]
pub struct ProductSeedExecuteEnvelope {
    pub confirmation_phrase: &'static str,
    pub execution: execute::ProductSeedExecuteResponse,
}

pub async fn get_product_seed_plan(
    State(state): State<AppState>,
    Extension(admin): Extension<AdminContext>,
    Extension(request_id): Extension<RequestId>,
) -> Result<Json<ApiResponse<ProductSeedPlanResponse>>, AppError> {
    ensure_admin_permission(&admin, "system:read")?;
    let preflight = preflight::build_product_seed_preflight(&state.db, admin.tenant_id).await?;

    Ok(Json(ApiResponse::ok(
        ProductSeedPlanResponse {
            plan: seed_plan::build_product_seed_plan(),
            preflight,
        },
        request_id.to_string(),
    )))
}

pub async fn execute_product_seed_plan(
    State(state): State<AppState>,
    Extension(admin): Extension<AdminContext>,
    Extension(request_id): Extension<RequestId>,
    Json(payload): Json<execute::ProductSeedExecuteRequest>,
) -> Result<Json<ApiResponse<ProductSeedExecuteEnvelope>>, AppError> {
    ensure_admin_permissions(
        &admin,
        &["system:update", "app:create", "server_api_key:update"],
    )?;
    let execution = execute::execute_product_seed(&state, &admin, &request_id, payload).await?;

    Ok(Json(ApiResponse::ok(
        ProductSeedExecuteEnvelope {
            confirmation_phrase: execute::seed_confirmation_phrase(),
            execution,
        },
        request_id.to_string(),
    )))
}

pub async fn list_product_seed_executions(
    State(state): State<AppState>,
    Extension(admin): Extension<AdminContext>,
    Extension(request_id): Extension<RequestId>,
    Query(query): Query<history::ProductSeedExecutionQuery>,
) -> Result<Json<ApiResponse<history::ProductSeedExecutionListResponse>>, AppError> {
    ensure_admin_permission(&admin, "system:read")?;
    let history = history::list_product_seed_executions(&state.db, admin.tenant_id, &query).await?;

    Ok(Json(ApiResponse::ok(history, request_id.to_string())))
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

fn ensure_admin_permissions(
    admin: &AdminContext,
    permission_codes: &[&str],
) -> Result<(), AppError> {
    for permission_code in permission_codes {
        ensure_admin_permission(admin, permission_code)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{error::AppError, modules::auth::session::AdminContext};

    use super::{ensure_admin_permission, ensure_admin_permissions};

    fn admin_with_permissions(permissions: Vec<&str>) -> AdminContext {
        AdminContext {
            session_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            team_member_id: Uuid::new_v4(),
            email: "owner@example.com".to_owned(),
            name: "Owner".to_owned(),
            email_verified: true,
            tenant_name: "Tenant".to_owned(),
            roles: vec!["owner".to_owned()],
            permissions: permissions.into_iter().map(str::to_owned).collect(),
            mfa_enabled: true,
        }
    }

    #[test]
    fn seed_plan_permission_uses_system_read() {
        let admin = admin_with_permissions(vec!["system:read"]);

        assert!(ensure_admin_permission(&admin, "system:read").is_ok());
    }

    #[test]
    fn seed_plan_permission_rejects_missing_system_read() {
        let admin = admin_with_permissions(vec!["ai:read"]);
        let error = ensure_admin_permission(&admin, "system:read").expect_err("forbidden");

        assert!(matches!(error, AppError::Forbidden(_)));
    }

    #[test]
    fn seed_execution_requires_system_app_and_server_key_permissions() {
        let admin =
            admin_with_permissions(vec!["system:update", "app:create", "server_api_key:update"]);

        assert!(ensure_admin_permissions(
            &admin,
            &["system:update", "app:create", "server_api_key:update"]
        )
        .is_ok());
    }

    #[test]
    fn seed_execution_rejects_missing_write_permission() {
        let admin = admin_with_permissions(vec!["system:update", "app:create"]);
        let error = ensure_admin_permissions(
            &admin,
            &["system:update", "app:create", "server_api_key:update"],
        )
        .expect_err("forbidden");

        assert!(matches!(error, AppError::Forbidden(_)));
    }
}
