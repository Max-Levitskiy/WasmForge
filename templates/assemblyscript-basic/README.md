# AssemblyScript (TypeScript) Module Template

Produces a `.wasm` module compatible with WasmForge using AssemblyScript. Minimal runtime, no heap for inputs.

Build and copy:
```bash
cd /Users/max/git/webtree/WasmForge/templates/assemblyscript-basic
npm install
npm run build:release
mkdir -p /Users/max/git/webtree/WasmForge/desktop-app/test-modules
cp build/assemblyscript_basic.wasm /Users/max/git/webtree/WasmForge/desktop-app/test-modules/
```

Exports:
- `add(a: i32, b: i32) -> i32`
- `validate_url(ptr,len) -> i32`
- `prepare_http_get(ptr,len) -> i32`

Notes:
- Uses `asc` with `--exportMemory` and `--runtime stub` to avoid heavy runtime.
- Input bytes must be read from `memory` using `load<u8>(ptr + i)`.
