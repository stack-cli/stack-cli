# Stack

Stack is a K8s operator the simplifies web application deployment as well as providing a Supabase style Backed as a Service (Baas).

![Stack Architecture](crates/stack-cli.com/content/docs/architecture/architecture.svg "Stack Architecture")

## Supabase Compatibility

All endpoints are compatible with Supabase client libraries.

## stack init

Install K8s Operators

```
🔌 Connecting to the cluster...
✅ Connected
🐘 Installing Cloud Native Postgres Operator (CNPG)
⏳ Waiting for Cloud Native Postgres Controller Manager
🛡️ Installing Keycloak Operator
📦 Creating namespace keycloak
⏳ Waiting for Keycloak Operator to be Available
📦 Creating namespace stack-system
📜 Installing StackApp CRD
⏳ Waiting for StackApp CRD
🔐 Setting up roles
🤖 Installing the operator into stack-system
🗄️ Ensuring Keycloak database in namespace keycloak
✅ Keycloak database created.
🛡️ Ensuring Keycloak instance in namespace keycloak
```

## stack deploy

Deploy takes a `yaml` manifest and install your app into a namespace as well as installing and required services such as Postgres.

```
stack deploy --manifest demo.stack.yaml --profile dev
```

```
🔌 Connecting to the cluster...
✅ Connected
📜 Installing StackApp CRD
⏳ Waiting for StackApp CRD
📦 Creating namespace stack-demo
🚀 Applied StackApp `stack-app` in namespace `stack-demo`
```

## stack secrets

Stack creates all the required secrets so you don't have to manage them.

```
stack secrets --manifest demo-stack-app.yaml
```

```
ANON_JWT=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoiYW5vbiIsImlzcyI6InN0YWNrIiwiZXhwIjoyMDgzNDA5MDgwfQ.awDa3WMeqcN0Tg95eZ9bMYXNaxQ0dNJ1vGI8puMw0ao
APPLICATION_URL=postgres://application_user:testpassword@stack-db-cluster-rw:5432/stack-app?sslmode=disable
AUTHENTICATOR_URL=postgres://authenticator:testpassword@stack-db-cluster-rw:5432/stack-app?sslmode=disable
MIGRATIONS_URL=postgres://db-owner:testpassword@stack-db-cluster-rw:5432/stack-app?sslmode=disable
READONLY_URL=postgres://application_readonly:testpassword@stack-db-cluster-rw:5432/stack-app?sslmode=disable
SERVICE_ROLE_JWT=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJyb2xlIjoic2VydmljZV9yb2xlIiwiaXNzIjoic3RhY2siLCJleHAiOjIwODM0MDkwODB9.g3wAUnnMzkF--pUggmZhROFQ66Pkq7HRLe5wVs3xMjc
```

## K3s, K3d or Cloud Kubernetes

Stack deploys to a minimal Kubernetes install running on your laptop so development and production align more closely.

![Stack in K8s](crates/stack-cli.com/assets/landing-page/k9s.png "Stack in k8s")

## /rest/v1

Postgrest compatible API to your database.

## /storage/v1

S3 compatible storage

## /realtime/v1

Subscribe to database changes over WebSockets.

## /oidc

No need to build Authentication screen the `/oidc` route connects you to Keycloak.

## /

Stack also deploys your application and any services secured in Kubernetes.