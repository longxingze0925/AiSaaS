alter table ai_model_routes
  add column if not exists timeout_seconds integer null,
  add column if not exists retryable_statuses jsonb,
  add column if not exists param_override_json jsonb,
  add column if not exists header_override_json jsonb;

update ai_model_routes
set
  retryable_statuses = coalesce(retryable_statuses, '[]'::jsonb),
  param_override_json = case
    when param_override_json is not null then param_override_json
    when config_json is not null and jsonb_typeof(config_json) = 'object' then config_json
    else '{}'::jsonb
  end,
  header_override_json = coalesce(header_override_json, '{}'::jsonb);

alter table ai_model_routes
  alter column weight set default 100,
  alter column retryable_statuses set default '[]'::jsonb,
  alter column retryable_statuses set not null,
  alter column param_override_json set default '{}'::jsonb,
  alter column param_override_json set not null,
  alter column header_override_json set default '{}'::jsonb,
  alter column header_override_json set not null;

alter table ai_model_routes
  drop constraint if exists ai_model_routes_provider_id_fkey,
  add constraint ai_model_routes_provider_id_fkey
    foreign key (provider_id) references ai_providers(id) on delete cascade;

alter table ai_model_routes
  drop constraint if exists ai_model_routes_priority_check,
  drop constraint if exists ai_model_routes_weight_check,
  drop constraint if exists ai_model_routes_timeout_check,
  drop constraint if exists ai_model_routes_retryable_statuses_array_check,
  drop constraint if exists ai_model_routes_param_override_object_check,
  drop constraint if exists ai_model_routes_header_override_object_check,
  add constraint ai_model_routes_priority_check check (priority >= 0 and priority <= 100000),
  add constraint ai_model_routes_weight_check check (weight >= 0 and weight <= 100000),
  add constraint ai_model_routes_timeout_check check (timeout_seconds is null or (timeout_seconds >= 1 and timeout_seconds <= 600)),
  add constraint ai_model_routes_retryable_statuses_array_check check (jsonb_typeof(retryable_statuses) = 'array'),
  add constraint ai_model_routes_param_override_object_check check (jsonb_typeof(param_override_json) = 'object'),
  add constraint ai_model_routes_header_override_object_check check (jsonb_typeof(header_override_json) = 'object');

create index if not exists idx_ai_model_routes_model_order
on ai_model_routes(tenant_id, model_id, enabled desc, priority asc, weight desc, created_at asc);

create index if not exists idx_ai_model_routes_provider
on ai_model_routes(tenant_id, provider_id);

create unique index if not exists idx_ai_model_routes_unique_provider_model
on ai_model_routes(tenant_id, model_id, provider_id, coalesce(provider_model, ''));

insert into ai_model_routes (
  tenant_id,
  model_id,
  provider_id,
  provider_model,
  enabled,
  priority,
  weight
)
select
  tenant_id,
  id,
  provider_id,
  provider_model,
  enabled,
  100,
  100
from ai_models
where provider_id is not null
on conflict do nothing;
