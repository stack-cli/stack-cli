# Secrets

Stack relies on plain Kubernetes Secrets. Bring your own secret material and place it in the same namespace as your `StackApp`; the controller will not create or manage external vaults for you.

## Bind secrets to services

Reference secrets through `services.web.secret_env` to project individual keys into environment variables:

```bash
kubectl create secret generic app-secrets \
  --namespace stack-demo \
  --from-literal=api_key=super-secret
```

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: stack-demo
  namespace: stack-demo
spec:
  services:
    web:
      secret_env:
        - name: API_KEY
          secret_name: app-secrets
          secret_key: api_key
```

## Database URLs

The controller only injects database URLs when you opt in on the web service:

```yaml
spec:
  services:
    web:
      database_url: DATABASE_URL
      migrations_database_url: DATABASE_MIGRATIONS_URL
      readonly_database_url: DATABASE_READONLY_URL
```

## Stack-generated secrets

Use `stack status` to view the secrets Stack generates, including JWTs and Keycloak admin credentials:

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

## Tips

- Keep secrets small and scoped to a single app namespace.
- Rotate secrets by updating the Kubernetes Secret; the controller will roll deployments as needed.
- For external secret managers, sync into Kubernetes first (e.g., external-secrets) and then reference via `secret_env`.
