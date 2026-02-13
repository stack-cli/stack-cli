create extension if not exists pgcrypto;

create table if not exists public.demo_items (
  id uuid primary key default gen_random_uuid(),
  user_id uuid not null,
  title text not null,
  created_at timestamptz not null default now()
);

create index if not exists demo_items_user_created_idx
  on public.demo_items (user_id, created_at desc);

alter table public.demo_items enable row level security;

drop policy if exists demo_items_select_own on public.demo_items;
create policy demo_items_select_own
  on public.demo_items
  for select
  using (auth.uid() = user_id);

drop policy if exists demo_items_insert_own on public.demo_items;
create policy demo_items_insert_own
  on public.demo_items
  for insert
  with check (auth.uid() = user_id);

grant usage on schema public to authenticated;
grant select, insert on table public.demo_items to authenticated;

notify pgrst, 'reload schema';
