alter table ai_providers
  add column if not exists last_test_status text null,
  add column if not exists last_test_error text null,
  add column if not exists last_test_at timestamptz null;

do $$
begin
  if not exists (
    select 1
    from pg_constraint
    where conname = 'ai_providers_last_test_status_check'
  ) then
    alter table ai_providers
      add constraint ai_providers_last_test_status_check
      check (
        last_test_status is null
        or last_test_status in ('success', 'failed')
      );
  end if;
end $$;
