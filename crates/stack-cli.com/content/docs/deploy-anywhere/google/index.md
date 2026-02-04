# Deploy Anywhere: Google Cloud Compute Engine

This guide shows how to take a single Google Compute Engine VM, install k3s, and run multiple Stack apps on the same instance.

## 1) Provision the VM

- Create a new VM instance (Ubuntu 22.04 LTS).
- Choose at least 2 vCPU and 4 GB RAM.
- Assign a static external IP.
- Allow inbound TCP on 22, 80, 443, and 6443 (restrict 6443 to your IP if possible).

SSH in:

```bash
gcloud compute ssh YOUR_VM_NAME --zone YOUR_ZONE
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
    auth: {}
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
    auth: {}
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

Apply both:

```bash
stack deploy --manifest frontend.yaml
stack deploy --manifest api.yaml
```

Check status:

```bash
stack status --manifest frontend.yaml
stack status --manifest api.yaml
```

Each StackApp gets its own namespace, database, and auth service, all on the same VM.

## 5) Expose apps to the internet

- For quick exposure, use the [Ingress](../../ingress/) guide to add a tunnel per StackApp.
- For production ingress, point your DNS at the VM and configure an ingress controller or load balancer.

## Next steps

- Add storage with the [Storage](../../storage/) guide.
- Add a real identity provider via the [OIDC (Keycloak)](../../oidc/) overview.
