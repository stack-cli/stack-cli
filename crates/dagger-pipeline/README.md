# Dagger Pipeline

This crate runs build/publish tasks via Dagger for Stack.

## Prerequisites

- Dagger engine available
- Rust toolchain
- From repo root: `cargo`

## Commands

Run from repository root:

```bash
cargo run -p dagger-pipeline -- <command>
```

### Build CLI + operator image path

Build all CLI targets:

```bash
cargo run -p dagger-pipeline -- build
```

Build one target:

```bash
cargo run -p dagger-pipeline -- build linux
cargo run -p dagger-pipeline -- build macos
cargo run -p dagger-pipeline -- build windows
```

Outputs:

- `artifacts/linux/stack`
- `artifacts/macos/stack`
- `artifacts/windows/stack.exe`
- `artifacts/operator/stack-operator.tar`

Operator image publish uses env vars already supported by `src/operator.rs`.

### Build/publish demo app image

Build and optionally publish the Next.js demo app image:

```bash
cargo run -p dagger-pipeline -- demo-app
```

Build context:

- `examples/react-supabase-next`

Artifact output:

- `artifacts/demo-app/react-supabase-next.tar`

Publish behavior:

- Publishes only when `GITHUB_REF_NAME=main`
- Defaults to `ghcr.io/stack-cli/react-supabase-next`

Useful env vars:

- `GITHUB_REF_NAME` (must be `main` to publish)
- `STACK_DEMO_APP_TAGS` (comma-separated, e.g. `latest,dev`)
- `STACK_VERSION` (optional extra tag)
- `GITHUB_SHA` (optional extra `sha-xxxxxxx` tag)
- `STACK_DEMO_APP_REGISTRY` (default `ghcr.io`)
- `STACK_DEMO_APP_REPOSITORY` (default `stack-cli/react-supabase-next`)
- `GHCR_USERNAME` or `GITHUB_ACTOR`
- `GHCR_TOKEN` or `GITHUB_TOKEN`

Example local publish test:

```bash
GITHUB_REF_NAME=main \
STACK_DEMO_APP_TAGS=latest \
GHCR_USERNAME=<your-gh-user> \
GHCR_TOKEN=<your-ghcr-token> \
cargo run -p dagger-pipeline -- demo-app
```
