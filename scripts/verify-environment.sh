#!/usr/bin/env bash
set -euo pipefail

for command in git node npm rustc cargo arduino-cli; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "[missing] $command"
    exit 1
  fi
done

git --version
node --version
npm --version
rustc --version
cargo --version
arduino-cli version

if [[ "$(uname -s)" == "Darwin" ]]; then
  xcode-select -p
fi

echo "Environment is ready."

