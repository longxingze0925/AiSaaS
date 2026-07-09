# Backend Product Layer

This module is the backend customization boundary for concrete SaaS + AI products.

For the full project customization workflow, start with
`../../../../产品魔改清单.md`.

Keep shared platform capabilities in core modules:

- `auth`, `iam`, `team`, `tenant`
- `customer`, `subscription`
- `ai`, `server_api`
- `web_assets`, `web_works`
- `audit`, `system`, `notification`, `outbox`

Use this module for product-specific defaults, initialization checks, controlled seed execution, and product-only admin APIs.

## Files

| File | Purpose |
| --- | --- |
| `defaults.rs` | Static backend defaults for the current product. Frontend Product config should mirror user-facing values. |
| `seed_plan.rs` | Side-effect-free dry-run plan. This describes what initialization would do. |
| `preflight.rs` | Read-only tenant checks against existing database state. |
| `execute.rs` | Controlled write path for resources that can be safely and idempotently created. |
| `history.rs` | Read-only seed execution history from audit logs. Historical secrets are not recoverable. |
| `admin.rs` | Axum handlers and permission checks for Product admin APIs. |
| `mod.rs` | Module exports. |

## Initialization Contract

The current initializer supports:

- Default application access config in `applications`.
- Default active Server API Key in `server_api_keys`.
- Manual/config-only reporting for subscription plan presets and AI billing reasons.

It intentionally does not create:

- Subscription plan catalog rows, because there is no plan catalog table yet.
- AI wallet or task-operation reason rows, because those values are Product defaults.
- Provider secrets, model provider credentials, or payment configuration.

## Safety Rules

- `GET /api/admin/product/seed-plan` must stay read-only.
- `POST /api/admin/product/seed-plan/execute` must reject when preflight has conflicts.
- Execution must require explicit confirmation and write permissions.
- `app_secret` and Server API `plain_key` may only be returned on newly created resources.
- Audit logs must never contain one-time secrets.
- Historical APIs may expose resource IDs and request IDs, not credentials.
- Database writes must be tenant-scoped.

## Adding Product-Specific Initialization

When a concrete product needs another initialized resource:

1. Add the default shape to `defaults.rs`.
2. Add a dry-run step in `seed_plan.rs`.
3. Add read-only checks in `preflight.rs`.
4. Only then add write logic in `execute.rs`.
5. Add a safe summary in `history.rs` if execution results change.
6. Register routes in `admin.rs` / `router.rs` only when a new endpoint is required.
7. Update `backend/openapi.yaml` and `API接口文档.md`.
8. Add tests before relying on the new path.

If the resource cannot be idempotently identified, do not add it to `execute.rs`; return `manual` from preflight instead.

## Validation

```powershell
cargo fmt --manifest-path backend\Cargo.toml
cargo test --manifest-path backend\Cargo.toml
```

The router test enforces that registered routes are listed in `backend/openapi.yaml`.
