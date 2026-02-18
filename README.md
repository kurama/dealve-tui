<div align="center">

  <img src="https://raw.githubusercontent.com/kurama/dealve-tui/main/.github/assets/logo.png" alt="Dealve" />

 Delve into game deals from your terminal 👾

![Demo](https://raw.githubusercontent.com/kurama/dealve-tui/main/.github/assets/demo.gif)

[![Built With Ratatui](https://img.shields.io/badge/Built_With-Ratatui-000?logo=ratatui&logoColor=B7FFA6&labelColor=222322&color=B482FF)](https://ratatui.rs)
[![crates.io](https://img.shields.io/crates/v/dealve-tui.svg?color=B482FF&labelColor=222322)](https://crates.io/crates/dealve-tui)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg?color=B482FF&labelColor=222322)](LICENSE)


<a title="This tool is Tool of The Week on Terminal Trove, The $HOME of all things in the terminal" href="https://terminaltrove.com/"><img src="https://cdn.terminaltrove.com/media/badges/tool_of_the_week/png/terminal_trove_tool_of_the_week_green_on_dark_grey_bg.png" alt="Terminal Trove Tool of The Week" width="150" /></a>

</div>

## Description

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
cargo install --locked dealve-tui
```

Or build from source:

```bash
git clone https://github.com/kurama/dealve-tui
cd dealve-tui
cargo install --locked --path tui
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
