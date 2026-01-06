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

You can add extra services alongside `web` by naming them directly under `services`. These are deployed as ClusterIP services (no nginx routing).

```yaml
spec:
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
    llm:
      image: ghcr.io/stack/demo-llm:latest
      port: 9100
      env:
        - name: MODEL
          value: llama3
    embeddings:
      image: ghcr.io/stack/demo-embeddings:latest
      port: 9200
```

If you need public routing for an extra service, add your own ingress or use a separate `StackApp`.
