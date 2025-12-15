list:
    just --list

dev-init:
    k3d cluster delete k3d-stack
    k3d cluster create k3d-stack --agents 1 -p "30010-30012:30010-30012@agent:0"
    just get-config

dev-setup:
    cargo run --bin stack-cli -- init --no-operator
    cargo run --bin stack-cli -- install --manifest demo-apps/demo.stack.yaml
    cargo run --bin stack-cli -- operator --once

codex: 
    sudo npm install -g @openai/codex

# Retrieve the cluster kube config - so kubectl and k9s work.
get-config:
    k3d kubeconfig write k3d-stack --kubeconfig-merge-default
    sed -i "s/127\.0\.0\.1/host.docker.internal/g; s/0\.0\.0\.0/host.docker.internal/g" "$HOME/.kube/config"
    # Disable TLS verification for local dev
    sed -i '/certificate-authority-data/d' "$HOME/.kube/config"
    sed -i '/cluster:/a \ \ \ \ insecure-skip-tls-verify: true' "$HOME/.kube/config"
    echo "✅ kubeconfig updated and TLS verification disabled"

stack:
    cargo run --bin stack-cli

ws:
    cd /workspace/crates/stack-cli.com && cargo watch --workdir /workspace/crates/stack-cli.com -w ./content -w ./src --no-gitignore -x "run --bin stack-cli-website"

wts:
    cd /workspace/crates/stack-cli.com && tailwind-extra -i ./input.css -o ./dist/tailwind.css --watch
