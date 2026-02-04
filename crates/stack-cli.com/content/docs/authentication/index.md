# Authentication

Stack ships Supabase Auth (GoTrue) so you can handle email/password and token-based flows without wiring extra services. The auth API is exposed through nginx at `/auth`.

This page continues the demo flow from the [Database](../database/), [REST](../rest/), and [Storage](../storage/) guides.

## Enable Supabase Auth

```yaml
spec:
  components:
    auth:
      api_external_url: http://localhost:30010/auth
      site_url: http://localhost:30010/auth
```

When `components.auth` is present, Stack deploys Supabase Auth and routes `/auth` through nginx. JWT and database credentials are wired automatically.

## Quick local test (demo manifest)

Export the anon JWT from `stack status`:

```bash
export ANON_JWT="$(stack status --manifest demo.stack.yaml | awk -F'Anon: ' '/Anon: /{print $2; exit}')"
```

Create a test user:

```bash
curl --location --request POST 'http://localhost:30090/auth/v1/signup' \
  --header "apikey: ${ANON_JWT}" \
  --header "Authorization: Bearer ${ANON_JWT}" \
  --header 'Content-Type: application/json' \
  --data-raw '{"email": "user@example.com", "password": "password"}'
```

Verify in Postgres:

```bash
kubectl -n stack-demo exec -it stack-demo-db-cluster-1 -- psql -d stack-demo \
  -c 'select email from auth.users;'
```

## What the controller creates

- An `auth` Deployment using `supabase/gotrue` on port `9999`.
- Uses the `database-urls` secret for migrations.
- Uses the `jwt-auth` secret for JWT signing.
- Creates the `auth` schema in your application database.

## Customising with the CRD

```yaml
spec:
  components:
    auth:
      api_external_url: https://example.com/auth
      site_url: https://example.com/auth
```
