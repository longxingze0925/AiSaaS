create index if not exists idx_ai_usage_tenant_status_created
on ai_usage_records(tenant_id, status, created_at desc);

create index if not exists idx_ai_usage_tenant_provider_created
on ai_usage_records(tenant_id, provider_id, created_at desc)
where provider_id is not null;

create index if not exists idx_ai_usage_tenant_endpoint_created
on ai_usage_records(tenant_id, endpoint, created_at desc);

create index if not exists idx_ai_usage_tenant_request_id
on ai_usage_records(tenant_id, request_id)
where request_id is not null;

create index if not exists idx_ai_usage_tenant_provider_request_id
on ai_usage_records(tenant_id, provider_request_id)
where provider_request_id is not null;
