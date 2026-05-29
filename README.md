# rtop
## Introduction
`rtop` is a cross platform process viewer written in Rust.

## Usage
- Help `h` / `F1` -> shows a list of supported key commands
- Setup `s` / `F2` -> setup
- Tree `t` / `F5` -> tree view
- Kill `k` / `F9` -> kill process
- Quit `q` / `F10` -> quit

## Build Instructions

### Prerequisites
- Rust

### Building

```bash
git clone https://github.com/randseas/rtop.git
cd rtop
cargo build --release
```

Executable binaries will be located at `target/release`.

## License
This project is licensed under the Apache-2.0 license. See the [LICENSE](LICENSE) file for details.
