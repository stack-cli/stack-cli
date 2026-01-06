# Quick Start

## k3s for local setup

For quick local setup, [k3s](https://k3s.io/) is the fastest path: it is a lightweight Kubernetes distribution that runs on a single machine while keeping the same APIs you will use in production.

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

3. **Apply a StackApp manifest.**

   ```bash
   stack install --manifest demo-stack-app.yaml
   ```

## Example Config

A minimal `StackApp` looks like this:

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: stack-app
  namespace: stack-demo
spec:
  components:
    ingress:
      port: 30010
    db:
      danger_override_password: testpassword
      expose_db_port: 30011
    rest: {}
    auth: {}
    realtime: {}
    storage:
      install_minio: true
  services:
    web:
      image: mendhak/http-https-echo:latest
      port: 8080
      database_url: DATABASE_URL
      env:
        - name: ECHO_INCLUDE_ENV_VARS
          value: "1"
        - name: JWT_HEADER
          value: Authorization
      init:
        image: alpine:3.18
        env:
          - name: INIT_MESSAGE
            value: "warming up"
```
