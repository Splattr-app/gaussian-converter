# Web version of [Splattr](https://splattr.app)'s Gaussian converter
This is a high-performance, open-source converter for Gaussian Splatting files (.splat, .ply, .spz), written in Rust and running entirely in the browser via WebAssembly.

## How to run
First of all, make sure that your current working directory is set to `wasm`
```bash
cd wasm
```

Install deps & build project
```bash
cargo build
```

Build WebAssembly package
```bash
 ~/.cargo/bin/wasm-pack build . --target web
```

Serve the web page! You can do however you want, just make sure to serve the whole root of the `wasm` folder
- With python `http.server` (will be accessible at [http://localhost:8000/frontend](http://localhost:8000/frontend))
```bash
python -m http.server
```
- With npm (will be accessible at [http://localhost:3000/frontend](http://localhost:3000/frontend))
```bash
npx serve
```