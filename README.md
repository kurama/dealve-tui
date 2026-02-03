# Dealve

The fastest way to browse game deals from your terminal.

<!-- ![Demo](examples/demo.gif)

[![crates.io](https://img.shields.io/crates/v/dealve-tui.svg)](https://crates.io/crates/dealve-tui) -->
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

**Dealve** simplifies finding the best game deals across Steam, GOG, Humble Bundle, Epic Games, and more - all from your terminal. Powered by [IsThereAnyDeal](https://isthereanydeal.com).

## Pre-requisites

You need Rust and Cargo installed. If you don't have them, install via [rustup](https://rustup.rs/).

Rust version 1.70 or later is required. Check your version:

```bash
rustc --version
```

Update if needed:

```bash
rustup update
```

## Installation

```bash
cargo install dealve-tui
```

Or build from source:

```bash
git clone https://github.com/kurama/dealve-tui
cd dealve-tui
cargo install --path tui
```

This installs the `dealve` binary to `~/.cargo/bin/`.

## Usage

```bash
dealve
```

On first launch, you'll be guided through a quick setup to configure your [IsThereAnyDeal API key](https://isthereanydeal.com/apps/) (free).

Configuration is stored in `~/.config/dealve/config.json`.

## Project Structure

```
dealve-tui/
├── core/    # Shared types and domain logic
├── api/     # IsThereAnyDeal API client
└── tui/     # Terminal UI application
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
