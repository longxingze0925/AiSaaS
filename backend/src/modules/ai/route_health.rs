use std::collections::HashSet;

use redis::AsyncCommands;
use serde::Serialize;
use uuid::Uuid;

use crate::{metrics, state::AppState};

const FAILURE_THRESHOLD: u32 = 3;
const FAILURE_WINDOW_SECONDS: u64 = 60;
const OPEN_SECONDS: u64 = 120;

#[derive(Debug, Clone, Serialize)]
pub struct RouteHealthSnapshot {
    pub status: String,
    pub open: bool,
    pub failure_count: u32,
    pub failure_ttl_seconds: Option<i64>,
    pub open_ttl_seconds: Option<i64>,
}

impl Default for RouteHealthSnapshot {
    fn default() -> Self {
        Self::healthy()
    }
}

impl RouteHealthSnapshot {
    fn healthy() -> Self {
        Self {
            status: "healthy".to_owned(),
            open: false,
            failure_count: 0,
            failure_ttl_seconds: None,
            open_ttl_seconds: None,
        }
    }

    fn unknown() -> Self {
        Self {
            status: "unknown".to_owned(),
            open: false,
            failure_count: 0,
            failure_ttl_seconds: None,
            open_ttl_seconds: None,
        }
    }

    fn from_values(
        failure_count: u32,
        failure_ttl_seconds: Option<i64>,
        open_ttl_seconds: Option<i64>,
    ) -> Self {
        let open = open_ttl_seconds.is_some();
        let status = if open {
            "open"
        } else if failure_count > 0 {
            "watching"
        } else {
            "healthy"
        };

        Self {
            status: status.to_owned(),
            open,
            failure_count,
            failure_ttl_seconds,
            open_ttl_seconds,
        }
    }
}

pub(crate) async fn unhealthy_route_ids(state: &AppState, route_ids: &[Uuid]) -> HashSet<Uuid> {
    let route_ids = unique_route_ids(route_ids);
    if route_ids.is_empty() {
        return HashSet::new();
    }

    let keys = route_ids
        .iter()
        .map(|route_id| route_open_key(*route_id))
        .collect::<Vec<_>>();
    let mut connection = match state.redis_connection().await {
        Ok(connection) => connection,
        Err(error) => {
            record_route_health_error(format!("redis route health connection failed: {error}"));
            return HashSet::new();
        }
    };
    let values = match redis::cmd("MGET")
        .arg(&keys)
        .query_async::<Vec<Option<String>>>(&mut connection)
        .await
    {
        Ok(values) => values,
        Err(error) => {
            record_route_health_error(format!("redis route health mget failed: {error}"));
            return HashSet::new();
        }
    };

    route_ids
        .into_iter()
        .zip(values.into_iter())
        .filter_map(|(route_id, value)| value.map(|_| route_id))
        .collect()
}

pub(crate) async fn route_health_snapshots(
    state: &AppState,
    route_ids: &[Uuid],
) -> Vec<(Uuid, RouteHealthSnapshot)> {
    let route_ids = unique_route_ids(route_ids);
    if route_ids.is_empty() {
        return Vec::new();
    }

    let mut connection = match state.redis_connection().await {
        Ok(connection) => connection,
        Err(error) => {
            record_route_health_error(format!("redis route health connection failed: {error}"));
            return route_ids
                .into_iter()
                .map(|route_id| (route_id, RouteHealthSnapshot::unknown()))
                .collect();
        }
    };
    let mut snapshots = Vec::with_capacity(route_ids.len());
    for route_id in route_ids {
        let snapshot = match route_health_snapshot_with_connection(&mut connection, route_id).await
        {
            Ok(snapshot) => snapshot,
            Err(error) => {
                record_route_health_error(format!("redis route health snapshot failed: {error}"));
                RouteHealthSnapshot::unknown()
            }
        };
        snapshots.push((route_id, snapshot));
    }

    snapshots
}

pub(crate) async fn record_route_success(state: &AppState, route_id: Option<Uuid>) {
    let Some(route_id) = route_id else {
        return;
    };
    let mut connection = match state.redis_connection().await {
        Ok(connection) => connection,
        Err(error) => {
            record_route_health_error(format!("redis route health connection failed: {error}"));
            return;
        }
    };
    let result: redis::RedisResult<usize> = redis::cmd("DEL")
        .arg(route_failure_key(route_id))
        .arg(route_open_key(route_id))
        .query_async(&mut connection)
        .await;
    if let Err(error) = result {
        record_route_health_error(format!("redis route health success reset failed: {error}"));
    }
}

pub(crate) async fn record_route_failure(state: &AppState, route_id: Option<Uuid>) {
    let Some(route_id) = route_id else {
        return;
    };
    let mut connection = match state.redis_connection().await {
        Ok(connection) => connection,
        Err(error) => {
            record_route_health_error(format!("redis route health connection failed: {error}"));
            return;
        }
    };
    let failure_key = route_failure_key(route_id);
    let count: u32 = match connection.incr(&failure_key, 1_u32).await {
        Ok(count) => count,
        Err(error) => {
            record_route_health_error(format!("redis route health incr failed: {error}"));
            return;
        }
    };
    if count == 1 {
        let expire_result: redis::RedisResult<bool> = connection
            .expire(&failure_key, FAILURE_WINDOW_SECONDS as i64)
            .await;
        if let Err(error) = expire_result {
            record_route_health_error(format!("redis route health expire failed: {error}"));
            return;
        }
    }
    if count < FAILURE_THRESHOLD {
        return;
    }

    let open_result: redis::RedisResult<()> = connection
        .set_ex(route_open_key(route_id), "1", OPEN_SECONDS)
        .await;
    if let Err(error) = open_result {
        record_route_health_error(format!("redis route health open failed: {error}"));
    }
}

pub(crate) async fn clear_route_health(state: &AppState, route_id: Uuid) -> RouteHealthSnapshot {
    let mut connection = match state.redis_connection().await {
        Ok(connection) => connection,
        Err(error) => {
            record_route_health_error(format!("redis route health connection failed: {error}"));
            return RouteHealthSnapshot::unknown();
        }
    };
    let result: redis::RedisResult<usize> = redis::cmd("DEL")
        .arg(route_failure_key(route_id))
        .arg(route_open_key(route_id))
        .query_async(&mut connection)
        .await;
    if let Err(error) = result {
        record_route_health_error(format!("redis route health clear failed: {error}"));
        return RouteHealthSnapshot::unknown();
    }

    RouteHealthSnapshot::healthy()
}

fn unique_route_ids(route_ids: &[Uuid]) -> Vec<Uuid> {
    let mut route_ids = route_ids.to_vec();
    route_ids.sort();
    route_ids.dedup();
    route_ids
}

async fn route_health_snapshot_with_connection(
    connection: &mut redis::aio::MultiplexedConnection,
    route_id: Uuid,
) -> redis::RedisResult<RouteHealthSnapshot> {
    let failure_key = route_failure_key(route_id);
    let open_key = route_open_key(route_id);
    let failure_count = connection
        .get::<_, Option<u32>>(&failure_key)
        .await?
        .unwrap_or(0);
    let failure_ttl_seconds = ttl_seconds(
        redis::cmd("TTL")
            .arg(&failure_key)
            .query_async(connection)
            .await?,
    );
    let open_ttl_seconds = ttl_seconds(
        redis::cmd("TTL")
            .arg(&open_key)
            .query_async(connection)
            .await?,
    );

    Ok(RouteHealthSnapshot::from_values(
        failure_count,
        failure_ttl_seconds,
        open_ttl_seconds,
    ))
}

fn ttl_seconds(value: i64) -> Option<i64> {
    if value > 0 {
        Some(value)
    } else {
        None
    }
}

fn route_failure_key(route_id: Uuid) -> String {
    format!("ai:route_health:{route_id}:failures")
}

fn route_open_key(route_id: Uuid) -> String {
    format!("ai:route_health:{route_id}:open")
}

fn record_route_health_error(message: String) {
    metrics::record_redis_error();
    tracing::warn!(%message, "ai route health redis operation failed");
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{
        route_failure_key, route_open_key, ttl_seconds, unique_route_ids, RouteHealthSnapshot,
    };

    #[test]
    fn route_health_keys_are_scoped_to_route_id() {
        let route_id = Uuid::new_v4();

        assert_eq!(
            route_failure_key(route_id),
            format!("ai:route_health:{route_id}:failures")
        );
        assert_eq!(
            route_open_key(route_id),
            format!("ai:route_health:{route_id}:open")
        );
    }

    #[test]
    fn unique_route_ids_sorts_and_deduplicates() {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let mut expected = vec![first, second];
        expected.sort();

        assert_eq!(unique_route_ids(&[second, first, second]), expected);
    }

    #[test]
    fn route_health_snapshot_status_follows_values() {
        let healthy = RouteHealthSnapshot::from_values(0, None, None);
        assert_eq!(healthy.status, "healthy");
        assert!(!healthy.open);

        let watching = RouteHealthSnapshot::from_values(2, Some(30), None);
        assert_eq!(watching.status, "watching");
        assert_eq!(watching.failure_count, 2);
        assert_eq!(watching.failure_ttl_seconds, Some(30));

        let open = RouteHealthSnapshot::from_values(3, Some(20), Some(100));
        assert_eq!(open.status, "open");
        assert!(open.open);
        assert_eq!(open.open_ttl_seconds, Some(100));
    }

    #[test]
    fn ttl_seconds_ignores_missing_or_persistent_keys() {
        assert_eq!(ttl_seconds(10), Some(10));
        assert_eq!(ttl_seconds(0), None);
        assert_eq!(ttl_seconds(-1), None);
        assert_eq!(ttl_seconds(-2), None);
    }
}
