# Deploy Anywhere: Hetzner Dedicated Servers

This guide walks through taking a single Hetzner dedicated server, installing k3s, and using Stack to run multiple apps on the same box.

## 1) Provision the server

- Create a new dedicated server in the Hetzner Robot or Cloud Console.
- Choose Ubuntu 22.04 LTS.
- Add your SSH key and note the public IP.
- Ensure ports 22, 80, 443, and 6443 are reachable (or use a firewall rule to allow your IP for 6443).

SSH in:

```bash
ssh root@YOUR_SERVER_IP
```

## 2) Install k3s

```bash
curl -sfL https://get.k3s.io | sh -s - --write-kubeconfig-mode 644
```

Set `kubectl` access:

```bash
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
kubectl get nodes
```

You should see the single node in `Ready` state.

## 3) Install the Stack CLI

```bash
export STACK_VERSION=v1.1.1
curl -OL https://github.com/stack-cli/stack-cli/releases/download/${STACK_VERSION}/stack-linux \
&& chmod +x ./stack-linux \
&& sudo mv ./stack-linux /usr/local/bin/stack
```

Bootstrap Stack on the k3s cluster:

```bash
stack init
```

## 4) Install multiple apps

Create one StackApp manifest per namespace. Example: a web frontend and an API service.

`frontend.yaml`:

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: frontend
  namespace: frontend
spec:
  components:
    auth:
      danger_override_jwt: "1"
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

`api.yaml`:

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: api
  namespace: api
spec:
  components:
    auth:
      danger_override_jwt: "1"
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

Apply both:

```bash
stack install --manifest frontend.yaml
stack install --manifest api.yaml
```

Check status:

```bash
stack status --manifest frontend.yaml
stack status --manifest api.yaml
```

Each StackApp gets its own namespace, database, and auth service, all on the same dedicated server.

## 5) Expose apps to the internet

- For quick exposure, use the [Cloudflare Tunnels](../../cloudflare/) guide to add a tunnel per StackApp.
- For production ingress, point your DNS at the server and configure an ingress controller or load balancer.

## Next steps

- Add storage with the [Storage](../../storage/) guide.
- Add a real identity provider via the [Keycloak operator](../../keycloak-operator/) overview.
