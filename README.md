# EnvVault

EnvVault is a local-first, encrypted desktop app for storing and managing `.env` files across projects. It keeps project environment variables in one encrypted SQLite vault on your machine.

EnvVault does **not** sync with your project folders. Workflow:

1. Create a project in the app.
2. Create environments inside it (`dev`, `stage`, `prod`, or custom).
3. Paste `.env` contents into an environment; EnvVault parses key/value rows.
4. Copy back as `.env` text when needed.

## Screenshots

> Placeholder - add screenshots here.

## Tech stack

- **Tauri 2** (Rust backend)
- **React + TypeScript + Vite** frontend
- **Tailwind CSS + shadcn/ui** (zero-radius, monochrome)
- **SQLite** via `rusqlite` (bundled)
- **AES-256-GCM** via `aes-gcm` (fresh 96-bit nonce per value)
- **OS keychain** via `keyring` (libsecret/GNOME Keyring on Linux, Keychain on macOS, Credential Manager on Windows)
- **TanStack Query** for frontend async state
- **pnpm** package manager

## Development

### Linux prerequisites (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl wget file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libsecret-1-dev
```

You also need Node.js, pnpm, and Rust (`rustup`/`cargo`).

### Run in dev

```bash
pnpm install
pnpm tauri dev
```

If `cargo` is not on PATH in your shell:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

## Tests

Rust:

```bash
cd src-tauri
cargo test
```

Frontend build/typecheck:

```bash
pnpm build
```

## Security

- Secrets are encrypted at rest in SQLite (AES-256-GCM per value).
- Master key is stored in the OS keychain, never sent to frontend, never written to DB/logs.
- App is local-first, offline, no telemetry/analytics/auto-update.

See [`docs/SECURITY.md`](./docs/SECURITY.md) for the threat model.

## Documentation

- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md)
- [`docs/DESIGN.md`](./docs/DESIGN.md)
- [`docs/SECURITY.md`](./docs/SECURITY.md)
- [`docs/future/README.md`](./docs/future/README.md)
- [`AGENTS.md`](./AGENTS.md)
