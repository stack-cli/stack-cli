# RabbitMQ

Stack can deploy RabbitMQ as an optional component for asynchronous messaging and worker-style workloads.

## Enable RabbitMQ

```yaml
spec:
  components:
    rabbitmq: {}
```

Default behavior:

- Image: `rabbitmq:3-management-alpine`
- AMQP port: `5672`
- Management port: `15672`
- Persistence: enabled (`5Gi` PVC)
- Credentials: generated in secret `rabbitmq-auth` (`username`, `password`)
- URL secret: `rabbitmq-urls` (`amqp-url`, `management-url`)

## Inject AMQP URL into your app

Use `rabbitmq_url` to map the generated AMQP URL into your app container env.

```yaml
spec:
  components:
    rabbitmq: {}
  services:
    web:
      image: ghcr.io/example/app:latest
      port: 8080
      rabbitmq_url: AMQP_URL
```

Stack sets `AMQP_URL` from:

- Secret: `rabbitmq-urls`
- Key: `amqp-url`

## Customize RabbitMQ

```yaml
spec:
  components:
    rabbitmq:
      image: rabbitmq:3-management-alpine
      port: 5672
      management_port: 15672
      size: 20Gi
      persistence: true
      expose_amqp_port: 30030
      expose_management_port: 30031
```

## Use your own credentials secret

If you already manage RabbitMQ credentials, set `credentials_secret_name`.
The secret must include:

- `username`
- `password`

```yaml
spec:
  components:
    rabbitmq:
      credentials_secret_name: my-rabbitmq-auth
```

## What Stack creates

- `rabbitmq` Deployment and AMQP ClusterIP Service
- `rabbitmq-management` ClusterIP Service for management UI
- Optional PVC `rabbitmq-data` when persistence is enabled
- Secret `rabbitmq-urls` containing connection URLs
- NetworkPolicy for RabbitMQ pod traffic
