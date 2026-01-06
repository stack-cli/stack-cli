# Kubernetes Setup

Before you begin, set up a local Kubernetes cluster. See the [Kubernetes Already?](../local-kubernetes/) guide for Docker Desktop and k3s options.

## Install Stack

1. **Grab the CLI.**

   ```bash
   curl -fsSL https://stack-cli.com/install.sh | bash
   ```

2. **Bootstrap the platform operators into your cluster.**

   ```bash
   stack init
   ```

   This command installs CloudNativePG, Keycloak, ingress, the Stack controller, and custom resource definitions that describe your applications.

3. **Apply the demo StackApp manifest.**

   ```bash
   curl -fsSL https://raw.githubusercontent.com/stack-cli/stack-cli/main/demo-apps/demo.stack.yaml \
     -o demo.stack.yaml
   stack install --manifest demo.stack.yaml
   ```

## What just happened?

You now have a Kubernetes namespace with a full backend stack wired together:

- A Postgres cluster created for your app.
- Auth, REST, Realtime, and Storage services ready to use.
- Secrets created for database URLs and JWTs.
- An nginx gateway that exposes `/auth`, `/rest`, `/realtime`, and `/storage`.

If you come from a Next.js background, think of this as a production-ready backend running beside your app, but declared with a single manifest instead of a pile of services.

## Optional: install k9s

[k9s](https://k9scli.io/) is a terminal UI for Kubernetes. It makes it easy to browse pods, logs, and services while you are learning the platform.

```bash
brew install k9s
```

On Linux, follow the install guide in the k9s docs.
