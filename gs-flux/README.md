
# GS-Flux

<h3 align="center">A CLI tool for Gaussian Splatting file conversion.</h3>

`gs-flux` is a command-line utility for converting 3D Gaussian Splatting files between various formats, written in Rust for maximum performance and safety.


## Installation

### Pre-compiled Binaries

For a specific version, you can download a pre-compiled binary for your operating system from the [**GitHub Releases**](https://todo.com) page. Just download the archive, extract it, and place the `gs-flux` executable in a directory that is in your system's `PATH`.

## Usage

The basic command structure is `gs-flux <source_file> <output_file>`. The tool infers the format from the file extensions.

### Basic Conversion

Convert a `.ply` file to a `.splat` file:
```bash
gs-flux scene.ply scene.splat
```

Convert a `.spz` file back to a `.ply` file:
```bash
gs-flux input.spz output.ply
```

### Advanced Options

#### PLY Encoding

By default, `.ply` files are written with binary encoding for efficiency. You can specify ASCII encoding using the `--encoding` flag. This option is **only valid for `.ply` output files**.

```bash
gs-flux scene.splat scene.ply --encoding ascii
```

#### Getting Help

You can always see all available options by running:
```bash
gs-flux --help
```

## Building from Source

If you want to build `gs-flux` from the source code:

1.  Clone the repository:
    ```bash
    git clone https://github.com/Splattr-app/gaussian-converter
    cd gs-flux
    ```
2.  Build the release binary:
    ```bash
    cargo build --release
    ```
3.  The executable will be located at `target/release/gs-flux`.

## Contributing

Contributions are welcome! Please feel free to open an issue to report a bug or suggest a feature, or submit a pull request with your improvements.