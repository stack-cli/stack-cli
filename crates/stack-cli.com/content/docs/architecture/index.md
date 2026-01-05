# Stack Architecture

Stack is delivered as a Kubernetes operator plus a set of curated operators that your clusters rarely ship out of the box. Understanding how those pieces fit together helps you diagnose issues or extend the platform.

![Alt text](architecture.svg "Stack Architecture")

## Stack vs Supabase

| Benefit | Stack | Supabase (self-hosted) |
| --- | --- | --- |
| Deploy anywhere | Kubernetes-first, runs on any cluster (k3s, k3d, managed, bare metal). | Docker Compose friendly, great for single-host setups. |
| Production parity | Development matches production because it is the same operator, CRDs, and manifests. | Local dev is simple but can diverge from production Kubernetes. |
| App definition | One `StackApp` manifest that declares services and components. | Multiple services wired in `docker-compose` plus project config. |
| Secrets handling | Secrets are created and managed as Kubernetes Secrets. | You manage env files and compose secrets manually. |
| Multi-app isolation | Namespaces, per-app DBs, per-app services. | Typically one project per compose stack. |
| Extensibility | Add services and operators alongside Stack components. | Extend compose with extra services. |
| Gateway routing | NGINX routes `/auth`, `/rest`, `/realtime`, `/storage` in one place. | Supabase API gateway handles its internal services. |
| Operations model | Declarative, reconciled by the operator. | Imperative docker lifecycle. |

## When Stack is a good fit

- You want a Kubernetes-first platform that runs the same way locally and in production.
- You deploy multiple apps or namespaces and want repeatable isolation.
- You want a single manifest that declares components and services.
- You want Stack to generate and manage secrets for you.

## When Supabase is a good fit

- You want the fastest possible single-host setup with docker-compose.
- You want Supabase UI and built-in workflows with minimal Kubernetes knowledge.
- You are experimenting locally and do not need production parity yet.

If you want a quick migration path, Stack can run Postgres, Auth, REST, Storage, and Realtime equivalents while you keep your application code unchanged.
