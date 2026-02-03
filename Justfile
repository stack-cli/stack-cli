list:
    just --list

dev-init:
    # 30010: nginx (demo)
    # 30011: postgres (demo)
    # 30012: selenium webdriver (demo)
    # 30013: selenium vnc (demo)
    # 30014: mailhog web (demo)
    # 30015: nginx (bionic)
    # 30016: postgres (bionic)
    k3d cluster delete k3d-stack
    k3d cluster create k3d-stack --agents 1 -p "30010-30016:30010-30016@agent:0"
    just get-config

dev-setup:
    cargo run --bin stack-cli -- init --no-operator
    cargo run --bin stack-cli -- deploy --manifest demo-apps/demo.stack.yaml --profile dev
    cargo run --bin stack-cli -- operator --once

dev-status:
    cargo run --bin stack-cli -- status --manifest demo-apps/demo.stack.yaml

dev-secrets:
    cargo run --bin stack-cli -- secrets --manifest demo-apps/demo.stack.yaml --db-host host.docker.internal --db-port 30011

bionic-setup:
    cargo run --bin stack-cli -- init --no-operator
    cargo run --bin stack-cli -- deploy --manifest demo-apps/bionic.stack.yaml --profile uat
    cargo run --bin stack-cli -- operator --once

selenium-setup:
    cargo run --bin stack-cli -- deploy --manifest demo-apps/bionic.stack.yaml --profile test -n bionic-selenium
    cargo run --bin stack-cli -- operator --once

bionic-secrets:
    cargo run --bin stack-cli -- secrets --manifest demo-apps/bionic.stack.yaml --db-host host.docker.internal --db-port 30013

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
