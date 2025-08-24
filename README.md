<p align="center">
  <img src="https://splattr.app/favicon.ico" alt="Splattr Converter Ecosystem Logo" width="60">
</p>
<h1 align="center">Splattr Gaussian Converter Tools</h1>
<p align="center">
  A high-performance, open-source toolkit for converting 3D Gaussian Splatting files.
</p>

<p align="center">
  <a href="https://github.com/Splattr-app/gaussian-converter/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
</p>

---

This repository contains a suite of tools for converting 3D Gaussian Splatting files between various formats. The core logic is written in **Rust** for maximum performance and safety, and is deployed across multiple targets, including a native CLI and a WebAssembly-powered web app.

## 📦 Repository Structure

This is a monorepo containing several related packages. Here's what you'll find in each directory:

| Package | Description |
| :--- | :--- |
| **[`./converter_core`](./converter_core)** | The heart of the toolkit. A Rust library that provides the core logic for all import and export operations. If you want to understand the conversion architecture, start here. |
| **[`./gs-flux`](./gs-flux)** | A simple command-line interface (CLI) built on top of `converter-core`.|
| **[`./gs-flux-desktop`](./gs-flux-desktop)** | A desktop application built on top of `converter-core`. Built with Tauri. |
| **[`./wasm`](./wasm)** | A WebAssembly build of `converter-core`, complete with a simple frontend. This demonstrates how to run the converter entirely in the browser. |

## 🚀 Quick Start

Depending on your needs, you can use the tools in a few different ways.

### Using the CLI (`gs-flux`)

For local file conversion, the CLI is the best tool for the job.

```bash
# Example: Convert a .ply file to the compressed .spz format
gs-flux input.ply output.spz
```

For full installation and usage instructions, see the **[gs-flux README](./gs-flux/README.md)**.

### Using the Web App (WASM)

To run the converter directly in your browser, you can build and serve the WebAssembly package.

For detailed instructions, see the **[wasm README](./wasm/README.md)**.

### Using the Desktop app

For detailed instructions, see the **[gs-flux-desktop README](./gs-flux-desktop/README.md)**.

## 🤝 Contributing

We welcome contributions from the community! Whether it's adding support for a new file format, fixing a bug, or improving documentation, please feel free to open an issue or submit a pull request.

Please see the README of the specific package you wish to contribute to for more detailed information.