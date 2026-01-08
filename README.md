# Stack

Stack ships two deliverables:

- **`stack-cli`** — a Rust CLI/operator that bootstraps the Stack platform (Postgres, Keycloak, ingress, and StackApp CRDs) into a Kubernetes cluster.
- **`stack-cli.com`** — the public documentation site (Dioxus SSR + Tailwind) that explains how to build Stack apps.

Everything lives in this monorepo so code, docs, and manifests version together.

## Repository layout

| Path | Description |
| --- | --- |
| `crates/stack-cli` | Source for the CLI/operator plus Kubernetes manifests under `config/` and Keycloak realm templates under `keycloak/`. |
| `crates/stack-cli.com` | Documentation + marketing site. Contains markdown content in `content/`, assets in `assets/`, Tailwind entrypoints, and the Dioxus app in `src/`. |
| `demo-apps/` | Example Stack applications used in the docs and demos. |
| `Justfile` | Workflow shortcuts for cluster setup, docs watch tasks, etc. |

## Prerequisites

- Rust toolchain (install via `rustup`, keep it on the stable channel reported in `rust-toolchain` or `rust-toolchain.toml` if present).
- `cargo`, `just`, and `npm` (Tailwind watcher) available locally or through the dev container.
- A local Kubernetes cluster. The repo assumes `k3d` via the `just dev-init` recipe, but any kubeconfig reachable at `~/.kube/config` works.

## Cluster bootstrap workflow

Most flows start with a local k3d cluster:

```bash
just dev-init          # delete/recreate the k3d-stack cluster
just get-config        # merge the kubeconfig and rewrite endpoints to host.docker.internal
just dev-setup         # run stack-cli init/deploy/operator once against the cluster
```

`just dev-setup` maps to:

1. `cargo run --bin stack-cli -- init --no-operator`
2. `cargo run --bin stack-cli -- deploy --manifest demo-stack-app.yaml`
3. `cargo run --bin stack-cli -- operator --once`

Feel free to run those commands individually when iterating on specific areas.

## Developing `stack-cli`

- `cargo run --bin stack-cli -- -h` to inspect the CLI.
- `cargo run --bin stack-cli -- init` / `deploy` / `operator` for real workflows.
- `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` must pass before merging.
- The operator uses kube-rs; logs go through `tracing`. Use `RUST_LOG=debug cargo run --bin stack-cli -- operator` for verbose output.
- Kubernetes manifests live under `crates/stack-cli/config/`. Regenerate or edit them there and keep CRDs/operators in sync.
- Keycloak bootstrap realms are under `crates/stack-cli/keycloak/realm.json`.

## Building the documentation site (`stack-cli.com`)

The Dioxus/Tailwind site is self-contained in `crates/stack-cli.com`.

### Live development

From the repo root (or use `direnv`/`just` aliases):

```bash
just wts    # tailwind-extra -i ./input.css -o ./dist/tailwind.css --watch
just ws     # cargo watch ... -x "run --bin static-website"
```

Visit http://localhost:8080 to preview the rendered site. The Axum dev server reads from `dist/` and injects live reload.

### Production build / Cloudflare Pages

Generate the site without starting the dev server:

```bash
cd crates/stack-cli.com
DO_NOT_RUN_SERVER=1 cargo run --bin static-website
```

Cloudflare Pages runs `cloudflare-build.sh`, which performs the same build step in CI.

## Contributing

- Follow the agent responsibilities in `AGENTS.md` to decide whether a change belongs to the Platform (CLI) or Documentation team.
- Keep docs and CLI behavior in sync—CLI flags referenced on the site must exist, and new CLI features should land with docs.
- Prefer ASCII in docs/comments, surface follow-up work with `// TODO(username): ...`, and coordinate across agents when touching both areas.
