# OIDC (Keycloak)

Stack relies on Keycloak for OAuth2 and OpenID Connect flows. When you run `stack init`, the CLI installs everything required to run a shared Keycloak control plane inside your cluster.

![Alt text](keycloak.svg "Keycloak")

## Enabling OIDC

```yaml
spec:
  components:
    ingress:
      port: 30010
    db: 
    rest: {}
    oidc:
      # Required by Keycloak for OIDC. OIDC requires a stable redirect URL.
      hostname-url: http://localhost:30013
```

When you enable `oidc` in your Stack yaml all traffic to your app will be intercepted and a login/registration page will be shown.

![Alt text](keycloak-login.png "Keycloak")

## How Stack uses Keycloak

- Each `StackApp` with `spec.components.oidc.hostname-url` defined triggers the Stack controller to ensure a Keycloak realm and OAuth2 Proxy configuration exist.
- The CLI creates an initial admin secret named `keycloak-initial-admin` in the Keycloak namespace. `stack status --manifest …` reads this secret so you can log in instantly.
- OAuth2 Proxy is configured to trust Keycloak and inject the right upstream headers toward your app.

## What gets installed

1. **CustomResourceDefinitions** – `keycloaks.k8s.keycloak.org` and `keycloakrealmimports.k8s.keycloak.org` enable the operator to watch realms and servers.
2. **Keycloak Operator** – A deployment that reconciles `Keycloak` and `KeycloakRealmImport` resources.
3. **Dedicated namespace** – Stack creates (or reuses) the `keycloak` namespace so the identity stack stays isolated.
4. **Backing database** – The Keycloak operator provisions a CloudNativePG cluster for Keycloak itself; Stack wires credentials automatically.

## Verifying the installation

```bash
kubectl get pods -n keycloak
kubectl get keycloaks.k8s.keycloak.org -n keycloak
kubectl get secret keycloak-initial-admin -n keycloak -o yaml
```

If you ever need to reinstall Keycloak components (for example after manually deleting the namespace), re-run `stack init`. The CLI reapplies the CRDs, operator deployment, and database manifests idempotently.

## Accessing the Keycloak admin

If you need to reach the Keycloak admin UI quickly, you can spin up a temporary Cloudflare tunnel in the `keycloak` namespace:

```bash
kubectl -n keycloak run cloudflared-quick --restart=Never --image=cloudflare/cloudflared:latest -- \
  tunnel --no-autoupdate --url http://keycloak-service.keycloak.svc.cluster.local:8080
```

Then use `kubectl` to read the initial admin credentials:

```bash
kubectl -n keycloak get secret keycloak-initial-admin \
  -o jsonpath='{.data.username}' | base64 -d && echo
kubectl -n keycloak get secret keycloak-initial-admin \
  -o jsonpath='{.data.password}' | base64 -d && echo
```

The output includes the admin username and password.
