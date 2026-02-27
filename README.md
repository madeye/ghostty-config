# ghostty-config

A web-based configuration GUI for the [Ghostty](https://ghostty.org) terminal emulator.

Automatically discovers all config options, themes, fonts, and keybindings from your local Ghostty installation and presents them in a browser-based editor.

## Features

- Browse and edit all 180+ Ghostty config options organized by category
- Preview and apply 400+ bundled themes
- Manage keybindings with a visual key capture UI
- Live terminal preview
- Config validation
- Import/export configuration
- Save & apply with automatic Ghostty reload (macOS)

## Requirements

- [Ghostty](https://ghostty.org) installed
- [Rust](https://rustup.rs) toolchain

## Usage

```sh
cargo run --release
```

The UI opens automatically at `http://127.0.0.1:3456`.

## License

[MIT](LICENSE) - Copyright (c) 2026 Max Lv
