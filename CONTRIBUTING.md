# Contributing to WasmForge

Thanks for your interest in contributing! This guide will help you set up your environment and submit changes effectively.

## Prerequisites

- Rust (nightly) toolchain with components: `rustfmt`, `clippy`
- Target `wasm32-unknown-unknown` for building the example module
- macOS/Linux recommended

Quick setup:

```bash
rustup toolchain install nightly
rustup component add rustfmt clippy --toolchain nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

## Repository Structure

- `desktop-app/` – Rust MCP server and WASM host (binary)
- `test-module/` – Sample WebAssembly module (Rust, built to `wasm32-unknown-unknown`)
- `docs/` – Documentation and runnable examples

## Build

```bash
# Build the WASM example module
cd test-module
cargo +nightly build --target wasm32-unknown-unknown --release
mkdir -p ../desktop-app/test-modules
cp target/wasm32-unknown-unknown/release/test_module.wasm ../desktop-app/test-modules/

# Build the desktop app
cd ../desktop-app
cargo +nightly build
```

## Test (manual)

From `desktop-app/`:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/debug/desktop-app
```

See more in `docs/EXAMPLES.md`.

## Coding Standards

- Run `cargo +nightly fmt --all` before committing
- Run `cargo +nightly clippy --all-targets --all-features`
- Prefer clear, readable code (see `docs/IMPLEMENTATION_PLAN.md` for architecture)

## Commit & PR Process

- Use descriptive commit messages; Conventional Commits encouraged (e.g., `feat:`, `fix:`, `docs:`)
- Include docs and examples updates where relevant
- Open a PR with a clear description and checklist; CI must be green

## Licensing

By contributing, you agree that your contributions will be licensed under the
dual license of this project: MIT OR Apache-2.0.

## Code of Conduct

Please read and follow our `CODE_OF_CONDUCT.md`.


