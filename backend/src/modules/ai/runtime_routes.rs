use rand_core::{OsRng, RngCore};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Clone, FromRow)]
pub(crate) struct ResolvedAiModelRoute {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub provider_kind: String,
    pub provider_name: String,
    pub provider_base_url: String,
    pub provider_config: Value,
    pub provider_secret_encrypted: Option<String>,
    pub provider_model: Option<String>,
    pub priority: i32,
    pub weight: i32,
    pub timeout_seconds: Option<i32>,
    pub retryable_statuses: Value,
    pub param_override: Value,
    pub header_override: Value,
}

pub(crate) async fn load_enabled_model_routes(
    state: &AppState,
    tenant_id: Uuid,
    model_id: Uuid,
) -> Result<Vec<ResolvedAiModelRoute>, AppError> {
    sqlx::query_as::<_, ResolvedAiModelRoute>(
        r#"
        select
          r.id,
          p.id as provider_id,
          p.kind as provider_kind,
          p.name as provider_name,
          p.base_url as provider_base_url,
          p.config_json as provider_config,
          p.secret_encrypted as provider_secret_encrypted,
          r.provider_model,
          r.priority,
          r.weight,
          r.timeout_seconds,
          r.retryable_statuses,
          r.param_override_json as param_override,
          r.header_override_json as header_override
        from ai_model_routes r
        join ai_providers p
          on p.id = r.provider_id
          and p.tenant_id = r.tenant_id
        where r.tenant_id = $1
          and r.model_id = $2
          and r.enabled
          and p.enabled
        order by r.priority asc, r.created_at asc, r.id asc
        "#,
    )
    .bind(tenant_id)
    .bind(model_id)
    .fetch_all(&state.db)
    .await
    .map_err(|error| AppError::dependency(format!("ai model route database error: {error}")))
}

pub(crate) fn select_model_route(routes: &[ResolvedAiModelRoute]) -> Option<ResolvedAiModelRoute> {
    let roll = OsRng.next_u64();
    select_model_route_with_roll(routes, roll)
}

pub(crate) fn select_model_route_attempts(
    routes: &[ResolvedAiModelRoute],
) -> Vec<ResolvedAiModelRoute> {
    let Some(selected) = select_model_route(routes) else {
        return Vec::new();
    };

    let mut attempts = Vec::with_capacity(routes.len());
    attempts.push(selected.clone());
    attempts.extend(
        routes
            .iter()
            .filter(|route| route.id != selected.id)
            .cloned(),
    );
    attempts
}

fn select_model_route_with_roll(
    routes: &[ResolvedAiModelRoute],
    roll: u64,
) -> Option<ResolvedAiModelRoute> {
    let min_priority = routes.iter().map(|route| route.priority).min()?;
    let candidates = routes
        .iter()
        .filter(|route| route.priority == min_priority)
        .collect::<Vec<_>>();
    let total_weight = candidates
        .iter()
        .map(|route| route.weight.max(0) as u64)
        .sum::<u64>();

    if total_weight == 0 {
        return candidates.first().map(|route| (*route).clone());
    }

    let mut cursor = roll % total_weight;
    for route in candidates {
        let weight = route.weight.max(0) as u64;
        if cursor < weight {
            return Some(route.clone());
        }
        cursor -= weight;
    }

    None
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::{select_model_route_attempts, select_model_route_with_roll, ResolvedAiModelRoute};

    #[test]
    fn route_selection_prefers_lowest_priority() {
        let high_priority = route("primary", 10, 1);
        let low_priority = route("fallback", 100, 1000);

        let selected = select_model_route_with_roll(&[low_priority, high_priority.clone()], 999)
            .expect("route");

        assert_eq!(selected.provider_name, high_priority.provider_name);
    }

    #[test]
    fn route_selection_uses_weight_within_same_priority() {
        let first = route("first", 10, 1);
        let second = route("second", 10, 3);

        let selected =
            select_model_route_with_roll(&[first.clone(), second.clone()], 0).expect("first route");
        assert_eq!(selected.provider_name, first.provider_name);

        let selected =
            select_model_route_with_roll(&[first, second.clone()], 2).expect("second route");
        assert_eq!(selected.provider_name, second.provider_name);
    }

    #[test]
    fn route_attempts_start_with_selected_route_without_duplicates() {
        let first = route("first", 10, 1);
        let second = route("second", 10, 3);
        let third = route("third", 20, 1);

        let attempts = select_model_route_attempts(&[first.clone(), second.clone(), third.clone()]);

        assert_eq!(attempts.len(), 3);
        assert!(attempts.iter().any(|route| route.id == first.id));
        assert!(attempts.iter().any(|route| route.id == second.id));
        assert!(attempts.iter().any(|route| route.id == third.id));
        let mut ids = attempts.iter().map(|route| route.id).collect::<Vec<_>>();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3);
    }

    fn route(name: &str, priority: i32, weight: i32) -> ResolvedAiModelRoute {
        ResolvedAiModelRoute {
            id: Uuid::new_v4(),
            provider_id: Uuid::new_v4(),
            provider_kind: "openai_compatible".to_owned(),
            provider_name: name.to_owned(),
            provider_base_url: "https://api.example.com/v1".to_owned(),
            provider_config: json!({}),
            provider_secret_encrypted: None,
            provider_model: Some(format!("{name}-model")),
            priority,
            weight,
            timeout_seconds: None,
            retryable_statuses: json!([]),
            param_override: json!({}),
            header_override: json!({}),
        }
    }
}
