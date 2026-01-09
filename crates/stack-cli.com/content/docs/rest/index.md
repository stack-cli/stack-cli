# REST (PostgREST)

This page assumes you have already created the `instruments` table in the [Database](../database/) guide and applied the demo manifest.

## Configure RLS for PostgREST

PostgREST uses the `anon` role from your JWT. You need to enable RLS and grant access to the table.

Connect to Postgres

```bash
kubectl -n stack-demo exec -it stack-db-cluster-1 -- psql -d stack-app
```

then run:

```sql
alter table instruments enable row level security;
grant usage on schema public to anon;
grant select on table public.instruments to anon;
create policy "read instruments" on public.instruments for select using (true);
```

PostgREST caches schema metadata. After creating tables or changing policies, reload the cache:

```bash
kubectl -n stack-demo exec -it stack-db-cluster-1 -- psql -d stack-app -c "NOTIFY pgrst, 'reload schema';"
```

## Get the JWT

Export the anon token from `stack status`:

```
stack status --manifest demo.stack.yaml
```

```
🔌 Connecting to the cluster...
✅ Connected
🛡️ Keycloak Admin
   Username: temp-admin
   Password: 59a3c5766cb246df9f1a47f3b419270c
☁️ Cloudflare deployment not found in namespace 'stack-demo'
🔑 JWTs
   Anon: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoiYW5vbiJ9.YJ1RABrgii5P1iG6F66qZxZT7DgbfgXlFmACRQ6J1pI
   Service role: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoic2VydmljZV9yb2xlIn0.gGHcBuULser-F3q2XZ7FeJlaHvnKrFveWYigpBcW1ug
```

`export` the variable like below.

```bash
export ANON_JWT="$(stack status --manifest demo.stack.yaml | awk -F'Anon: ' '/Anon: /{print $2; exit}')"
```

## Query with curl

```bash
curl http://localhost:30090/rest/v1/instruments \
  -H "Authorization: Bearer ${ANON_JWT}"
```

If you created the table in the database guide, you should see the rows returned.

```json
[{"id":1,"name":"violin"}, 
 {"id":2,"name":"viola"}, 
 {"id":3,"name":"cello"}]
```
