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
  name: stack-app
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

The controller injects database URLs from the `database-urls` secret. Rename the target env vars if needed:

```yaml
spec:
  services:
    web:
      database_url: DATABASE_URL
      superuser_database_url: SUPERUSER_DATABASE_URL
```

## Tips

- Keep secrets small and scoped to a single app namespace.
- Rotate secrets by updating the Kubernetes Secret; the controller will roll deployments as needed.
- For external secret managers, sync into Kubernetes first (e.g., external-secrets) and then reference via `secret_env`.
