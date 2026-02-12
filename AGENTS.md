# Stack Repo Guide

This repository mixes several deliverables that move at different speeds. Use this guide to decide where changes belong and how to validate them before opening a PR.

## Docs Site (`crates/stack-cli.com`)
- **Purpose**: Publish the public docs + marketing site (a Rust static website) that explains Stack concepts and walks users through workflows.
- **Primary tasks**: Update markdown in `content/`, assets in `assets/`, and Dioxus components in `src/`. Keep code samples aligned with the current CRD and CLI behavior (components like `rest`, `realtime`, `storage`, `auth`, `cloudflare`, `ingress`).
- **Working locally**: `just wts` to watch Tailwind builds and `just ws` to run the Dioxus/Axum dev server (see README). Preview at `http://localhost:8080`.
- **Before shipping**: Run `DO_NOT_RUN_SERVER=1 cargo run --bin static-website` to ensure the site renders without the dev server, then trigger the Cloudflare Pages preview when build logic changes.

## CLI + Operator (`crates/stack-cli`)
- **Purpose**: Own the CLI and operator that install and reconcile Stack services (CNPG, Keycloak, nginx, StackApps) in Kubernetes.
- **Primary tasks**: Extend CLI commands, reconcile loop logic, and manifests in `config/` and `keycloak/`. Maintain CRD schema and component wiring (REST, Realtime, Storage, Auth, Cloudflare, Ingress). Keep demo manifests in `infra-as-code/` in sync with CRD changes.
- **Working locally**: `just dev-init`, `just get-config`, `just dev-setup`, and `just dev-secrets` to smoke test against k3d. Use `cargo run --bin stack-cli -- init/install/operator/status/secrets` for targeted workflows.
- **Before shipping**: `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`. For manifest or controller changes, validate against a local K3d/K3s cluster and document new behavior in docs.

## Cross-Cutting Expectations
- Follow `CONTRIBUTING.md` for code review and branching conventions.
- If a change touches more than one agent area, coordinate early so reviewers from each area can weigh in.
- Prefer ASCII in docs and comments unless you have a compelling reason otherwise.
- Surface follow-up work with TODO comments (`// TODO(username):`) or GitHub issues so the right agent can pick them up.
