# Ingress

Stack includes an opinionated Cloudflare deployment so you can expose namespaces without writing additional manifests. Configure it directly on your `StackApp` so the namespace and ingress target stay aligned.

## Quick tunnels (no Cloudflare account)

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: stack-demo
  namespace: stack-demo
spec:
  components:
    cloudflare: {}
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

- Omitting `components.cloudflare.secret_name` tells Stack to start a temporary tunnel.  
- The operator installs `cloudflared` into the same namespace and points it at the nginx service Stack created earlier.  
- Run `stack status --manifest demo.stack.yaml` to print the generated HTTPS URL.

Temporary tunnels are great for demos, development sessions, and any workflow where you just need to share access for a few minutes.

## Authenticated tunnels (bring your Cloudflare account)

When you want a long-lived hostname, create a Cloudflare tunnel token and a secret that also includes the tunnel name. The operator reads the secret and configures the tunnel.

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: cloudflare-tunnel
  namespace: stack-demo
stringData:
  token: "$CLOUDFLARE_TUNNEL_TOKEN"
  tunnel_name: "stack"
  # Optional: override the target URL (defaults to nginx in the namespace)
  # ingress_target: "http://nginx.stack-demo.svc.cluster.local:80"
---
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: stack-demo
  namespace: stack-demo
spec:
  components:
    cloudflare:
      secret_name: cloudflare-tunnel
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

The operator reuses the same nginx target as the quick tunnel. Because everything comes from your manifest:

- The namespace always matches your application.
- Switching environments (dev/staging/prod) is as simple as pointing to a different manifest.

## Verifying the tunnel

Use the status command any time you need credentials or the public URL:

```bash
stack status --manifest demo.stack.yaml
```

You will see:

- Keycloak admin username/password read from the `keycloak-initial-admin` secret in the shared Keycloak namespace.
- The latest Cloudflare URL scraped from the `cloudflared` pod logs in your manifest namespace.
- Helpful hints if the tunnel pod is not running.

Update your `StackApp` manifest with `spec.components.oidc.hostname-url` once you have a stable domain so Keycloak and OAuth2 Proxy can enforce proper redirects.
