# Rust on Nails Agents Guide

This repository mixes several deliverables that move at different speeds. To keep things tidy, we split the work into three "agents" that each own one slice of the Rust on Nails experience. Use this guide to decide where changes belong and how to validate them before opening a PR.

## Documentation Agent (`crates/stack-cli.com`)
- **Purpose**: Publish the public docs + marketing site that explains Nails concepts and walks users through the CLI flows.
- **Primary tasks**: Update markdown in `content/`, assets in `assets/`, and Dioxus components in `src/`. Keep code samples aligned with the default Nails template and live CLI flags.
- **Working locally**: `just wts` to watch Tailwind builds and `just ws` to run the Dioxus/Axum dev server (see README). Preview at `http://localhost:8080`.
- **Before shipping**: Run `DO_NOT_RUN_SERVER=1 cargo run --bin static-website` to ensure the site renders without the dev server, then trigger the Cloudflare Pages preview when build logic changes.

## Platform Agent (`crates/stack-cli`)
- **Purpose**: Own the CLI/operator that installs the Nails platform (CNPG, Keycloak, ingress, StackApps) into Kubernetes clusters.
- **Primary tasks**: Extend CLI commands, reconcile loop logic, and curated manifests in `config/` and `keycloak/`. Ensure docs reference real flags/behavior.
- **Working locally**: `just dev-init`, `just get-config`, and `just dev-setup` to smoke test against k3d. Use `cargo run --bin stack-cli -- init/install/operator` for targeted workflows.
- **Before shipping**: `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test`. For manifest or controller changes, validate against a local K3d/K3s cluster and document new flags in the README/docs.

## Cross-Cutting Expectations
- Follow `CONTRIBUTING.md` for code review and branching conventions.
- If a change touches more than one agent area, coordinate early so reviewers from each area can weigh in.
- Prefer ASCII in docs and comments unless you have a compelling reason otherwise.
- Surface follow-up work with TODO comments (`// TODO(username):`) or GitHub issues so the right agent can pick them up.
