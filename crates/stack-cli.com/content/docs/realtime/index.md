# Realtime

Stack can run Supabase Realtime for your namespace so you can subscribe to database changes over WebSockets.

This page continues the demo flow from the [Database](../database/) and [REST](../rest/) guides.

## Quick curl check

Realtime uses WebSockets, but you can confirm the endpoint responds with curl:

```bash
curl -i localhost:30090/realtime/health \
  -H "Authorization: Bearer <ANON_JWT>"
```

If Realtime is running, you should see a 200 response.

## Get the JWT

Use the status command to print the anon JWT:

```bash
stack status --manifest demo-stack-app.yaml
```

Example output:

```text
🔌 Connecting to the cluster...
✅ Connected
🛡️ Keycloak Admin
   Username: temp-admin
   Password: ec74684fb30140ee994bfb5599dbbd37
☁️ Cloudflare deployment not found in namespace 'stack-demo'
🔑 JWTs
   Anon: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoiYW5vbiJ9.E_2RkS5WSHEdZ_nxVMzTQQo-NLFLVFF8YXthl1IQk5g
   Service role: eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoic2VydmljZV9yb2xlIn0.vAnalcvp1tM4s94Qa5CHc3ZhT8_OaCmDaukhbkotcMs
```

Look for the `JWTs` section and copy the `Anon` token.

## Technical notes

- Realtime runs as a separate deployment in your namespace.
- It uses the shared `jwt-auth` secret for API JWT validation.
- The nginx gateway exposes it under `/realtime`.
