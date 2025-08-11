#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

rustup target add wasm32-unknown-unknown || true
cargo build --target wasm32-unknown-unknown --release

mkdir -p "$ROOT_DIR/desktop-app/test-modules"
cp target/wasm32-unknown-unknown/release/module_template.wasm "$ROOT_DIR/desktop-app/test-modules/"
echo "Copied module_template.wasm to $ROOT_DIR/desktop-app/test-modules/"

