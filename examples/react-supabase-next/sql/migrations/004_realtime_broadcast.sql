-- Enable Supabase Realtime broadcast sends/receives for authenticated demo users.
-- Without these policies, channel.send({ type: 'broadcast', ... }) can return "error".

grant usage on schema realtime to authenticated;
grant select, insert on table realtime.messages to authenticated;

alter table realtime.messages enable row level security;

drop policy if exists realtime_messages_broadcast_select_authenticated on realtime.messages;
create policy realtime_messages_broadcast_select_authenticated
  on realtime.messages
  for select
  to authenticated
  using (true);

drop policy if exists realtime_messages_broadcast_insert_authenticated on realtime.messages;
create policy realtime_messages_broadcast_insert_authenticated
  on realtime.messages
  for insert
  to authenticated
  with check (true);
