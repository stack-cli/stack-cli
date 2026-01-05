# Realtime

Stack can run Supabase Realtime for your namespace so you can subscribe to database changes over WebSockets.

## Enable the Realtime component

```yaml
apiVersion: stack-cli.dev/v1
kind: StackApp
metadata:
  name: stack-app
  namespace: stack-demo
spec:
  components:
    realtime: {}
  services:
    web:
      image: ghcr.io/stack/demo-app:latest
      port: 7903
```

Apply the manifest, then run a reconciliation:

```bash
stack install --manifest demo-stack-app.yaml
stack operator --once
```

## Get the JWT

Use the status command to print the anon JWT:

```bash
stack status --manifest demo-stack-app.yaml
```

Look for the `JWTs` section and copy the `Anon` token.

## Quick curl check

Realtime uses WebSockets, but you can confirm the endpoint responds with curl:

```bash
curl -i host.docker.internal:30010/realtime/v1/health \
  -H "Authorization: Bearer <ANON_JWT>"
```

If Realtime is running, you should see a 200 response.
