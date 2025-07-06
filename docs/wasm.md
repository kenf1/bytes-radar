# WebAssembly Support

bytes-radar now supports WebAssembly (WASM) compilation, allowing you to run the code analysis tools directly in browsers or WASI environments.

## Prerequisites

To build for WASM, you'll need:

1. Rust toolchain (latest stable)
2. wasm-pack: `cargo install wasm-pack`
3. For WASI: `rustup target add wasm32-wasi`
4. For browser WASM: `rustup target add wasm32-unknown-unknown`

## Building

### For Browsers

```bash
# Build with wasm-pack
wasm-pack build --target web --features wasm

# Or build with cargo
cargo build --target wasm32-unknown-unknown --features wasm
```

### For WASI

```bash
cargo build --target wasm32-wasi --features wasm
```

## Usage Example

### Browser

```javascript
import init, { analyze_repository } from 'bytes-radar';

async function main() {
    await init();
    
    const result = await analyze_repository('https://github.com/user/repo');
    console.log(result);
}

main();
```

### Node.js with WASI

```javascript
import { WASI } from '@wasmer/wasi';
import { WasmFs } from '@wasmer/wasmfs';

const wasmFs = new WasmFs();
const wasi = new WASI({
  args: [],
  env: {},
  bindings: {
    ...WASI.defaultBindings,
    fs: wasmFs.fs
  }
});

const instance = await WebAssembly.instantiate(
  await WebAssembly.compile(
    await fs.readFile('path/to/bytes-radar.wasm')
  ),
  {
    wasi_snapshot_preview1: wasi.wasiImport
  }
);

wasi.start(instance);
```

## API Reference

When using the WASM build, the following functions are available:

- `analyze_repository(url: string) -> Promise<Analysis>`
- `compare_repositories(urls: string[]) -> Promise<ComparisonResult>`

For detailed API documentation, please refer to the [API Reference](./api.md).

## Limitations

When running in WASM, some features might have limitations:

1. Network requests are handled differently in browser vs WASI environments
2. File system access is limited in browser environments
3. Memory usage might be more constrained compared to native builds

## Contributing

If you find any issues or have suggestions for the WASM implementation, please open an issue or submit a pull request on our GitHub repository. 