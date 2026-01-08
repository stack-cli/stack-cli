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

Export the anon token from `stack secrets`:

```bash
export ANON_JWT="$(stack secrets --manifest demo.stack.yaml | rg '^ANON_JWT=' | cut -d= -f2-)"
```

## Query with curl

```bash
curl http://localhost:30090/rest/v1/instruments \
  -H "Authorization: Bearer ${ANON_JWT}"
```

If you created the table in the database guide, you should see the rows returned.
