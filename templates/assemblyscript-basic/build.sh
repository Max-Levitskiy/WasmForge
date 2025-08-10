#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$SCRIPT_DIR"

if ! command -v npm >/dev/null 2>&1; then
  echo "npm is required" >&2
  exit 1
fi

mkdir -p build
npm install
npm run build:release

mkdir -p "$ROOT_DIR/desktop-app/test-modules"
cp build/assemblyscript_basic.wasm "$ROOT_DIR/desktop-app/test-modules/"
echo "Copied assemblyscript_basic.wasm to $ROOT_DIR/desktop-app/test-modules/"
