# Application Deployment

Your app is defined in the `services.web` section of the `StackApp` manifest. Stack deploys that container and wires in secrets, env vars, and database URLs.

## Basic web service

```yaml
spec:
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

## Environment variables

Plain env vars go in `env`:

```yaml
spec:
  services:
    web:
      env:
        - name: FEATURE_FLAG
          value: "true"
```

Secret-backed env vars go in `secret_env`:

```yaml
spec:
  services:
    web:
      secret_env:
        - name: API_KEY
          secret_name: app-secrets
          secret_key: api_key
```

## Database URLs

Request the database URLs you need by naming the env vars you want set:

```yaml
spec:
  services:
    web:
      database_url: DATABASE_URL
      migrations_database_url: DATABASE_MIGRATIONS_URL
      readonly_database_url: DATABASE_READONLY_URL
```

## Init containers

Use `init` when you need migrations or setup work before the web container starts:

```yaml
spec:
  services:
    web:
      init:
        image: alpine:3.18
        env:
          - name: INIT_MESSAGE
            value: "warming up"
```

## Multiple services

Stack currently manages a single `web` service per `StackApp`. If you need extra containers or workers, deploy standard Kubernetes resources alongside your `StackApp` in the same namespace and reuse the same secrets and database URLs, or split them into separate StackApps.
