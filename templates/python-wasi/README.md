# Python (WASI) Module Template — Planning

Status: Documentation-only. Running Python inside WasmForge requires WASI host support that is not yet enabled.

Recommended approach:
- Use CPython wasm32-wasi builds (or Pyodide) to produce a `.wasm` module.
- Expose thin C-ABI exports that validate inputs and delegate to Python logic.
- Expect larger binaries and slower cold start than Rust/AssemblyScript.

Host requirements to enable:
- `WasmExecutor` must instantiate modules with WASI imports (stdin/stdout/stderr, clocks, filesystem as needed).
- Conditional execution path: when a module requires WASI, supply imports during instantiation.

Interim guidance:
- Prefer Rust or AssemblyScript templates for now to avoid WASI dependencies.
- Track the WASI integration design in `memory-bank/tasks.md` under Module Template Suite plan.
