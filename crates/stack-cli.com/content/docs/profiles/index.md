# Profiles

Profiles let you keep one StackApp manifest and apply environment-specific overrides at deploy time.

Stack merges the selected profile into `spec` before it applies the manifest. The operator never sees
`spec.profiles`.

## Example

Base manifest with a dev profile:

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: my-app
  namespace: my-app
spec:
  components:
    db: {}
    auth: {}
    rest: {}
    storage: {}
  services:
    web:
      image: ghcr.io/acme/my-app:latest
      port: 7903
  profiles:
    dev:
      components:
        db:
          expose_db_port: 30011
        auth:
          expose_auth_port: 30013
```

Deploy with the profile:

```bash
stack deploy --manifest stack.yaml --profile dev
```

If you omit `--profile`, Stack uses the base `spec` as-is.

## Rules

- Profiles are optional.
- A profile only needs to include fields that differ from the base spec.
- Profile values replace base values unless both are maps, in which case they merge.

## Common use cases

- Expose DB or auth ports in dev only.
- Use different ingress settings per environment.
- Change cloudflare settings without duplicating manifests.
