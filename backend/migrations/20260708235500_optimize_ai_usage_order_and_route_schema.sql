create index if not exists idx_ai_usage_tenant_created_id
on ai_usage_records(tenant_id, created_at desc, id desc);

create index if not exists idx_ai_usage_tenant_customer_created_id
on ai_usage_records(tenant_id, customer_id, created_at desc, id desc);

create index if not exists idx_ai_usage_tenant_model_created_id
on ai_usage_records(tenant_id, model_id, created_at desc, id desc);

create index if not exists idx_ai_usage_tenant_status_created_id
on ai_usage_records(tenant_id, status, created_at desc, id desc);

create index if not exists idx_ai_usage_tenant_provider_created_id
on ai_usage_records(tenant_id, provider_id, created_at desc, id desc)
where provider_id is not null;

create index if not exists idx_ai_usage_tenant_endpoint_created_id
on ai_usage_records(tenant_id, endpoint, created_at desc, id desc);

drop index if exists idx_ai_usage_tenant_created;
drop index if exists idx_ai_usage_tenant_customer_created;
drop index if exists idx_ai_usage_tenant_model_created;
drop index if exists idx_ai_usage_tenant_status_created;
drop index if exists idx_ai_usage_tenant_provider_created;
drop index if exists idx_ai_usage_tenant_endpoint_created;

drop index if exists idx_ai_model_routes_tenant_model_enabled;
drop index if exists idx_ai_model_routes_tenant_provider;
drop index if exists idx_ai_model_routes_tenant_model_provider;

alter table ai_model_routes
  drop column if exists name,
  drop column if exists config_json;
