create table ai_model_routes (
  id uuid primary key default gen_random_uuid(),
  tenant_id uuid not null references tenants(id) on delete cascade,
  model_id uuid not null references ai_models(id) on delete cascade,
  provider_id uuid not null references ai_providers(id) on delete restrict,
  name text null,
  provider_model text null,
  priority integer not null default 100,
  weight integer not null default 1,
  enabled boolean not null default true,
  config_json jsonb not null default '{}'::jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  constraint ai_model_routes_name_check
    check (name is null or (length(trim(name)) > 0 and length(name) <= 128)),
  constraint ai_model_routes_provider_model_check
    check (provider_model is null or (length(trim(provider_model)) > 0 and length(provider_model) <= 256)),
  constraint ai_model_routes_priority_check
    check (priority >= 0 and priority <= 1000000),
  constraint ai_model_routes_weight_check
    check (weight >= 1 and weight <= 1000000),
  constraint ai_model_routes_config_object_check
    check (jsonb_typeof(config_json) = 'object')
);

create unique index idx_ai_model_routes_tenant_model_provider
on ai_model_routes(tenant_id, model_id, provider_id, lower(coalesce(provider_model, '')));

create index idx_ai_model_routes_tenant_model_enabled
on ai_model_routes(tenant_id, model_id, enabled, priority, weight desc);

create index idx_ai_model_routes_tenant_provider
on ai_model_routes(tenant_id, provider_id);
