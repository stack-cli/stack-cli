# Ingress

Stack includes an opinionated Cloudflare deployment so you can expose namespaces without writing additional manifests. Install it with the CLI so it won't be managed by the operator.

## Quick tunnels (no Cloudflare account)

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: stack-demo
  namespace: stack-demo
spec:
  components: {}
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

- Run `stack cloudflare --manifest demo.stack.yaml` to start a temporary tunnel.  
- The CLI installs `cloudflared` into the same namespace and points it at the nginx service Stack created earlier.  
- Run `stack status --manifest demo.stack.yaml` to print the generated HTTPS URL.

Temporary tunnels are great for demos, development sessions, and any workflow where you just need to share access for a few minutes.

## Authenticated tunnels (bring your Cloudflare account)

When you want a long-lived hostname, create a Cloudflare tunnel token. Use the CLI to create the secret and deploy the tunnel.

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: stack-demo
  namespace: stack-demo
spec:
  components: {}
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

```bash
stack cloudflare --manifest demo.stack.yaml --tunnel-name stack --token "$CLOUDFLARE_TUNNEL_TOKEN"
```

The CLI reuses the same nginx target as the quick tunnel. Because everything comes from your manifest:

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
