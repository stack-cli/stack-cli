# Stack Developer Platform

Stack is a self-hosted deployment platform that layers identity, networking, databases, and tooling on top of any Kubernetes cluster. Instead of wiring together operators, CRDs, and manifests by hand, you install Stack once and immediately get a PaaS-like workflow for your applications.

![Stack architecture](./architecture-diagram.svg)

Looking for a deeper dive? Read the [Stack architecture guide](./architecture/) to see how the operator, CRDs, and supporting services interact.

## Install Stack

1. **Grab the CLI.**

   ```bash
   export STACK_VERSION=v1.1.0
   curl -OL https://github.com/stack-cli/stack-cli/releases/download/${STACK_VERSION}/stack-linux \
   && chmod +x ./stack-linux \
   && sudo mv ./stack-linux /usr/local/bin/stack
   ```

2. **Bootstrap the platform operators into your cluster.**

   ```bash
   stack init
   ```

   This command installs CloudNativePG, Keycloak, ingress, the Stack controller, and custom resource definitions that describe your applications.

3. **Apply a StackApp manifest.**

   ```bash
   stack install --manifest demo-stack-app.yaml
   ```

   A minimal `StackApp` looks like this:

   ```yaml
   apiVersion: stack-cli.dev/v1
   kind: StackApp
   metadata:
     name: stack-app
     namespace: stack-demo
   spec:
     components:
       auth:
         danger_override_jwt: "1"
     services:
       web:
         image: ghcr.io/stack/demo-app:latest
         port: 7903
```

   The controller provisions a dedicated CloudNativePG cluster, injects connection strings into secrets, deploys your container as `stack-app`, and keeps everything in sync with the manifest.

   Components belong under `spec.components` (`db`, `auth`, `storage`) while workloads live under `spec.services`. Add application configuration with `env` and `secret_env` on the web service:

   ```yaml
  spec:
    services:
      web:
        env:
          - name: FEATURE_FLAG
            value: "true"
        secret_env:
          - name: API_KEY
            secret_name: app-secrets
            secret_key: api_key
        # Explicitly request DB URLs; nothing is injected unless you set these.
        database_url: DATABASE_URL
        migrations_database_url: DATABASE_MIGRATIONS_URL
        readonly_database_url: DATABASE_READONLY_URL
        # Optional init container
        init:
          image: alpine:3.18
          env:
            - name: INIT_MESSAGE
              value: "warming up"
   ```

### Secrets

- Stack uses standard Kubernetes Secrets; bring your own or create them in the app namespace.
- Wire secrets into services through `services.web.secret_env` to expose specific keys as environment variables.

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
  namespace: stack-demo
stringData:
  api_key: super-secret
---
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

## Operate and debug

- `stack operator --once` runs a single reconciliation loop locally so you can watch what the controller does without staying connected forever. Drop `--once` to keep it running.
- `stack status --manifest demo-stack-app.yaml` prints the Keycloak admin credentials and inspects the `cloudflared` pods inside the namespace defined by your manifest. When a quick tunnel is running you will see the temporary HTTPS URL here.
- Need ingress from the wider internet? Follow the [Cloudflare quick-tunnel guide](./cloudflare/) to create either temporary or authenticated tunnels straight from your StackApp manifest.
- Curious about what `stack init` installs? The [Keycloak operator](./keycloak-operator/) and [PostgreSQL operator](./postgres-operator/) guides explain the shared services Stack keeps healthy for you.
- Ready to ship an existing framework? See the [Rails on Kubernetes](./framework/) and [Flask on Kubernetes](./framework/flask/) guides for concrete end-to-end examples.
- Want a local env? The [Developers workflow](./developers/) page shows how to spin up k3d, patch your kubeconfig, and run the Stack CLI manually.

## What's next?

- `stack operator` lets you run the reconciliation loop locally for rapid debugging.
- `stack status --manifest demo-stack-app.yaml` shows Cloudflare URLs, Keycloak credentials, and other platform details for that namespace.
- Use the generated CRDs (`stackapps.stack-cli.dev`) from your own automation pipelines to manage namespaces and applications at scale.
