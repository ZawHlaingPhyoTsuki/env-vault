# EnvVault

A local-first, encrypted desktop vault for managing `.env` files across projects.

EnvVault stores project environment variables in one place with encryption at rest. It does **not** sync with your project folders: you manually create projects/environments in the app, paste `.env` contents, and copy/export them when needed.

## Status

MVP in development. Current scaffold includes Tauri 2 + React + TypeScript + Vite + Tailwind CSS + shadcn/ui foundations.

## Tech stack

- Tauri 2 (Rust backend)
- React + TypeScript + Vite
- Tailwind CSS + shadcn/ui
- SQLite (`rusqlite`, bundled) - planned
- AES-256-GCM (`aes-gcm`) - planned
- OS keychain (`keyring`) - planned

## Development

Prerequisites:

- Node.js + pnpm
- Rust toolchain (`rustup`, `cargo`)
- Tauri Linux prerequisites (Ubuntu/Debian): `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libxdo-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libsecret-1-dev`

Install and run:

```bash
pnpm install
pnpm tauri dev
```

Build:

```bash
pnpm tauri build
```

## Documentation

- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) - module overview and data flow.
- [`docs/DESIGN.md`](./docs/DESIGN.md) - visual design rules.
- [`docs/SECURITY.md`](./docs/SECURITY.md) - threat model and security notes.
- [`docs/future/`](./docs/future/README.md) - deferred feature designs.
- [`AGENTS.md`](./AGENTS.md) - rules for AI coding agents.

## Security

EnvVault is designed for local encrypted storage. See [`docs/SECURITY.md`](./docs/SECURITY.md) for what it protects against and what it does not.
