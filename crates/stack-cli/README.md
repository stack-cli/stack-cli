# stack-cli: Local Development Guide

This crate contains the Stack CLI and in-cluster operator.
Use this guide for local development and smoke testing against Kubernetes.

## CLI Help

```bash
cargo run --bin stack-cli -- -h
```

## Local Cluster Setup

Use k3d for a fast local cluster:

```bash
k3d cluster delete stack-demo || true
k3d cluster create stack-demo --agents 1 -p "30000-30013:30000-30013@agent:0"
```

If kubeconfig is not already available in your environment:

```bash
cp ~/.kube/config /workspace/tmp/kubeconfig
export KUBECONFIG=/workspace/tmp/kubeconfig
```

## Bootstrap Stack Components

Install core operators and CRDs:

```bash
cargo run --bin stack-cli -- init
```

Install Keycloak too (optional):

```bash
cargo run --bin stack-cli -- init --install-keycloak
```

Install manifests without running the in-cluster Stack operator:

```bash
cargo run --bin stack-cli -- init --no-operator
```

## Deploy a Demo StackApp

```bash
cargo run --bin stack-cli -- deploy --manifest ../../infra-as-code/demo.stack.yaml
```

## Run Operator Locally

Run one reconciliation tick:

```bash
cargo run --bin stack-cli -- operator --once
```

Run continuously:

```bash
cargo run --bin stack-cli -- operator
```

Watch StackApp resources:

```bash
kubectl get stackapplications --all-namespaces --watch
```

## Inspect Generated Access Details

```bash
cargo run --bin stack-cli -- status --manifest ../../infra-as-code/demo.stack.yaml
cargo run --bin stack-cli -- secrets --manifest ../../infra-as-code/demo.stack.yaml
```

## Before Opening a PR

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```