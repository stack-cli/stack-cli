# Authentication

Stack ships Supabase Auth (GoTrue) so you can handle email/password and token-based flows without wiring extra services. The auth API is exposed through nginx at `/auth`.

Stack also supports an optional browser login gateway using oauth2-proxy + Keycloak (configured under `components.oidc`). That OIDC flow is separate from the Supabase Auth API.

This page continues the demo flow from the [Database](../database/), [REST](../rest/), and [Storage](../storage/) guides.

## Enable Supabase Auth

```yaml
spec:
  components:
    auth:
      api_external_url: http://localhost:30010/auth
      site_url: http://localhost:30010
      confirm_email: false
```

When `components.auth` is present, Stack deploys Supabase Auth and routes `/auth` through nginx. JWT and database credentials are wired automatically.
Set `confirm_email: false` in local development when you want signups to auto-confirm.

## Supabase Auth and OIDC are complementary

- `components.auth` deploys Supabase Auth (`supabase/gotrue`) for your app auth API at `/auth`.
- `components.oidc.hostname-url` deploys oauth2-proxy and ensures a Keycloak realm for front-door OIDC login.
- You can use either one independently, or both together in the same `StackApp`.

Example enabling both:

```yaml
spec:
  components:
    auth:
      api_external_url: http://localhost:30010/auth
      site_url: http://localhost:30010
      confirm_email: false
    oidc:
      hostname-url: http://localhost:30010
```

See the [OIDC guide](../oidc/) for realm and oauth2-proxy details.

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
- Creates an init step that ensures the `supabase_auth_admin` role and `auth` schema exist.
- Uses the `jwt-auth` secret for JWT signing.
- Routes `/auth` to the `auth` service via nginx.

## Customising with the CRD

```yaml
spec:
  components:
    auth:
      api_external_url: https://example.com/auth
      site_url: https://example.com
      confirm_email: true
```
