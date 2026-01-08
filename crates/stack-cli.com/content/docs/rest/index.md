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
alter table public.instruments enable row level security;
  create policy "read instruments" on public.instruments for select using (true);
```

## Get the JWT

Use the status command to print the anon JWT:

```bash
stack status --manifest demo.stack.yaml
```

Example output:

```text
🔌 Connecting to the cluster...
✅ Connected
🛡️ Keycloak Admin
   Username: temp-admin
   Password: ec74684fb30140ee994bfb5599dbbd37
☁️ Cloudflare deployment not found in namespace 'stack-demo'
🔑 JWTs
   Anon: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoiYW5vbiJ9.E_2RkS5WSHEdZ_nxVMzTQQo-NLFLVFF8YXthl1IQk5g
   Service role: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoic2VydmljZV9yb2xlIn0.vAnalcvp1tM4s94Qa5CHc3ZhT8_OaCmDaukhbkotcMs
```

Look for the `JWTs` section and copy the `Anon` token.

## Query with curl

```bash
curl http://localhost:30090/rest/instruments \
  -H "Authorization: Bearer <ANON_JWT>"
```

If you created the table in the database guide, you should see the rows returned.


