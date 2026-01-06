# Local Kubernetes

## Tutorial Setup: Kubernetes Baseline

This tutorial works on **macOS, Windows, and Linux**. Rather than standardising on a single tool, we standardise on a **common Kubernetes baseline**.

By the end of this section, you will have:

- A **single-node Kubernetes cluster**
- `kubectl` installed and working
- A compatible Kubernetes version
- A working cluster that responds to:
  ```bash
  kubectl get nodes
  ```

How you reach this baseline depends on your operating system and experience level.

## Choose your setup path

### Path A — Recommended (most users)

Docker Desktop with Kubernetes enabled.

This is the easiest and most reliable option for:

- macOS
- Windows
- Linux (desktop environments)

Steps:

1. Install Docker Desktop for your operating system.
2. Open Docker Desktop settings.
3. Enable Kubernetes.
4. Wait until Docker Desktop reports that Kubernetes is running.
5. Verify your setup:
   ```bash
   kubectl get nodes
   ```

Docker Desktop installs and configures `kubectl` automatically.

### Path B — Alternative (advanced / Linux-native)

[k3s](https://k3s.io/).

This option is best suited for:

- Linux users
- Headless servers
- CI environments
- Users already familiar with containers and Kubernetes

Steps:

```bash
curl -sfL https://get.k3s.io | sh -
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml
kubectl get nodes
```

This setup results in the same functional Kubernetes environment used throughout the tutorial.

## Verify your environment

Before continuing, ensure the following command succeeds:

```bash
kubectl get nodes
```

You should see a single node in the `Ready` state.

Optional (recommended) verification:

```bash
kubectl get pods -A
kubectl version --short
```

### Important notes

- Minor Kubernetes version differences are acceptable unless explicitly stated.
- From this point forward, all instructions apply equally regardless of setup path.
- The remainder of the tutorial uses only standard Kubernetes concepts and tooling.

Once your cluster is running, proceed to the next section.
