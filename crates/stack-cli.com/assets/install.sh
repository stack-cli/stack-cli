#!/usr/bin/env bash
set -e

SUDO=""
if [ "$(id -u)" != "0" ]; then
  SUDO="sudo"
  echo "This script requires superuser access."
  echo "You will be prompted for your password by sudo."
  sudo -k
fi

$SUDO bash <<'SCRIPT'
set -e

echoerr() { echo "$@" 1>&2; }

if [[ ":$PATH:" != *":/usr/local/bin:"* ]]; then
  echoerr "Your path is missing /usr/local/bin, you need to add this to use this installer."
  exit 1
fi

case "$(uname)" in
  Darwin)
    OS=macos
    ;;
  Linux)
    OS=linux
    ;;
  *)
    echoerr "This installer is only supported on Linux and macOS."
    exit 1
    ;;
 esac

ARCH="$(uname -m)"
case "$ARCH" in
  x86_64)
    ARCH=x64
    ;;
  arm64)
    ARCH=arm64
    ;;
  aarch*)
    ARCH=arm
    ;;
  *)
    echoerr "unsupported arch: $ARCH"
    exit 1
    ;;
 esac

mkdir -p /usr/local/bin

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

URL="https://github.com/stack-cli/stack-cli/releases/latest/download/stack-cli-$OS"
DEST="$TMP_DIR/stack"

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" -o "$DEST"
else
  wget -O "$DEST" "$URL"
fi

chmod +x "$DEST"
rm -f /usr/local/bin/stack
mv "$DEST" /usr/local/bin/stack

SCRIPT

LOCATION=$(command -v stack)
echo "stack installed to $LOCATION"
stack --version
