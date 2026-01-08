# Deploy Anywhere: Home Server or VM

This guide shows how to run Stack against a remote k3s cluster running on a home server or a single virtual machine.

This setup is ideal when you want:
- full control over your infrastructure
- low operational complexity
- the same workflow locally and in production

Stack runs on your laptop. The workloads run on Kubernetes.

---

## What you'll build

- A single-node k3s cluster (home server or VM)
- Stack installed into that cluster
- Stack CLI running locally, connected via kubeconfig
- A consistent local -> production workflow

---

## 1) Prepare a server

You can use:
- a home server
- a bare-metal machine
- a VM from any provider

Requirements:
- Linux (Ubuntu 22.04+ recommended)
- SSH access
- A reachable network address (public or private)

Open or forward the following ports:
- 22 - SSH
- 80 / 443 - HTTP / HTTPS
- 6443 - Kubernetes API (restrict access if possible)

SSH into the machine:

```bash
ssh user@SERVER_IP
```

---

## 2) Install k3s

On the server:

```bash
curl -sfL https://get.k3s.io | sh -s - --write-kubeconfig-mode 600
```

Verify the cluster:

```bash
kubectl --kubeconfig /etc/rancher/k3s/k3s.yaml get nodes
```

You should see the node in the Ready state.

---

## 3) Connect your laptop to the cluster

Stack communicates with Kubernetes using kubeconfig.

### Copy the kubeconfig

On the server:

```bash
sudo cat /etc/rancher/k3s/k3s.yaml
```

On your laptop, save it as:

```bash
~/.kube/stack-remote.yaml
```

Edit the file and update the API server address:

```yaml
server: https://SERVER_IP:6443
```

Test access from your laptop:

```bash
kubectl --kubeconfig ~/.kube/stack-remote.yaml get nodes
```

If this works, your laptop can control the remote cluster.

---

## 4) Install Stack CLI locally

Install Stack on your laptop (example for Linux):

```bash
export STACK_VERSION=v1.1.1
curl -OL https://github.com/stack-cli/stack-cli/releases/download/${STACK_VERSION}/stack-linux
chmod +x stack-linux
sudo mv stack-linux /usr/local/bin/stack
```

(Use the macOS binary if needed.)

---

## 5) Initialize Stack on the remote cluster

Point Stack at the remote cluster:

```bash
export KUBECONFIG=~/.kube/stack-remote.yaml
```

Install Stack into the cluster:

```bash
stack init
```

The Stack operator now runs inside the remote k3s cluster.

---

## 6) Deploy an application

Create a StackApp manifest locally:

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
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

Apply it from your laptop:

```bash
stack deploy --manifest app.yaml
```

Check status:

```bash
stack status --manifest app.yaml
kubectl get pods -n my-app
```

---

## 7) Local development with a remote backend

In this setup:
- backend services run on the remote server
- your application can still run locally

This allows fast iteration while testing against production-like infrastructure.

Typical options:
- expose services via ingress
- use SSH port forwarding
- use a secure tunnel

Your application environment variables should point to the remote endpoints.

---

## 8) Production access

For production use:
- configure DNS to point at the server
- enable TLS via your ingress controller
- remove any temporary tunnels

No changes to your StackApp definition are required.

---

## Security notes

- Restrict access to port 6443
- Treat kubeconfig files as secrets
- Prefer VPN or SSH tunnels when possible

---

## Why this approach works

- k3s is lightweight and easy to run
- one machine is often enough for many apps
- costs are predictable and low
- Kubernetes behaves the same everywhere

---

## Summary

Stack is designed so that:
- you run the CLI locally
- Kubernetes runs on any machine you own
- the same configuration works everywhere

This makes self-hosting practical without turning infrastructure into a full-time job.
