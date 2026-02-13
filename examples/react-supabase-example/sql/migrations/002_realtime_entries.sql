create extension if not exists pgcrypto;

create table if not exists public.realtime_entries (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null,
  message text not null,
  created_at timestamptz not null default now()
);

create index if not exists realtime_entries_user_created_idx
  on public.realtime_entries (user_id, created_at desc);

alter table public.realtime_entries enable row level security;

drop policy if exists realtime_entries_select_own on public.realtime_entries;
create policy realtime_entries_select_own
  on public.realtime_entries
  for select
  using (auth.uid() = user_id);

drop policy if exists realtime_entries_insert_own on public.realtime_entries;
create policy realtime_entries_insert_own
  on public.realtime_entries
  for insert
  with check (auth.uid() = user_id);

grant usage on schema public to authenticated;
grant select, insert on table public.realtime_entries to authenticated;

alter publication supabase_realtime add table public.realtime_entries;

notify pgrst, 'reload schema';
