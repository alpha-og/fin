# Fin

Fin is an experimental project and an attempt to create a rust powered desktop launcher based on Tauri 2.0. The frontend is made with react due to my pre-existing web dev knowledge but I'm considering utilising native GUI APIs or using rust frontend frameworks for the same in the future. Caching is done with an SQLite database.

## Usage

The default key-binding (global) is `Shift+Alt+Space` which is hardcoded into the binary. I plan to introduce user defined keybindings eventually.

## Development

### Prerequisites

- Rust
- Node.js
- PNPM

### Setup

1. Clone the repository
2. From the prohect root run `pnpm install`
3. Run `pnpm tauri dev` to start the development server

### Building

Run `pnpm tauri build` to build the application.
