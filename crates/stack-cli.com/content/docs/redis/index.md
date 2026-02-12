# Redis

Stack can deploy Redis as an optional component for caching, short-lived app state, and queue-like workloads.

## Enable Redis

```yaml
spec:
  components:
    redis: {}
```

Default behavior:

- Image: `redis:7-alpine`
- Port: `6379`
- Persistence: enabled (`1Gi` PVC)
- Password: generated and stored in secret `redis-auth`
- URL secret: `redis-urls` with key `redis-url`

## Inject REDIS_URL into your app

You can map the generated Redis URL into your web service or init container using `redis_url`.

```yaml
spec:
  components:
    redis: {}
  services:
    web:
      image: ghcr.io/example/app:latest
      port: 8080
      redis_url: REDIS_URL
```

Stack sets `REDIS_URL` from:

- Secret: `redis-urls`
- Key: `redis-url`

## Customize Redis

```yaml
spec:
  components:
    redis:
      image: redis:7.2-alpine
      port: 6379
      size: 5Gi
      persistence: true
      expose_redis_port: 30020
```

## Use your own password secret

If you already manage Redis credentials, reference a secret that contains key `password`.

```yaml
spec:
  components:
    redis:
      password_secret_name: my-redis-secret
```

## What Stack creates

- `redis` Deployment and ClusterIP Service
- Optional PVC `redis-data` when persistence is enabled
- Secret `redis-urls` containing `redis://` connection URL
- NetworkPolicy for Redis pod traffic
