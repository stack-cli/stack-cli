# REST (PostgREST)

Stack can run PostgREST for your namespace so you can query your database over HTTP.

## Enable the REST component

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: stack-app
  namespace: stack-demo
spec:
  components:
    rest: {}
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

Apply the manifest, then run a reconciliation:

```bash
stack install --manifest demo-stack-app.yaml
stack operator --once
```

## Create a table with RLS

Connect to the database and run:

```sql
-- Create the table
create table instruments (
  id bigint primary key generated always as identity,
  name text not null
);

-- Insert some sample data into the table
insert into instruments (name)
values
  ('violin'),
  ('viola'),
  ('cello');

alter table instruments enable row level security;
```

If you are using RLS, you still need to grant table access to the `anon` role:

```sql
grant usage on schema public to anon;
grant select on table public.instruments to anon;
```

## Get the JWT

Use the status command to print the anon JWT:

```bash
stack status --manifest demo-stack-app.yaml
```

Look for the `JWTs` section and copy the `Anon` token.

## Query with curl

```bash
curl host.docker.internal:30010/rest/instruments \
  -H "Authorization: Bearer <ANON_JWT>"
```

You should see the three rows returned.
