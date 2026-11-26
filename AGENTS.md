# AGENTS.md

Rules for AI coding agents working in EnvVault.

## Project shape

- **Desktop app:** Tauri 2 + React + TypeScript + Vite.
- **UI:** Tailwind CSS + shadcn/ui, monochrome, zero radius. Follow `docs/DESIGN.md`.
- **Backend:** Rust in `src-tauri/`. All crypto and database access lives in Rust. The frontend calls Tauri commands only.
- **Data:** SQLite via `rusqlite` (`bundled`) in the OS app-data directory. Use migrations; never break existing data.
- **Package manager:** pnpm.

## Architecture rules

- Keep Rust modules separated: `db`, `crypto`, `keychain`, `parser`, `commands`, `models`, `error`.
- The encryption key must never be sent to the frontend, written to disk, logged, or stored in the database.
- Decrypted values are returned to the frontend only when needed (reveal/copy/export). List views return masked placeholders.
- Use a single typed Rust error enum with `thiserror` and return clear UI-safe messages.
- Do not add telemetry, auto-update, analytics, or remote scripts. The app must work fully offline and make zero network requests.
- Tauri capabilities must be minimal. No broad filesystem scope.

## Security rules

- **Never log secret values, even in debug builds.** This includes env values, decrypted plaintext, encryption keys, nonces used as secret material context, and clipboard secrets.
- Do not store secrets in `localStorage`, `sessionStorage`, or frontend state longer than needed.
- Exported `.env` files must get `0600` permissions on Unix.
- If a security decision is ambiguous, ask before proceeding.

## Development

```bash
pnpm install
pnpm tauri dev
pnpm tauri build
```

Rust tests:

```bash
cd src-tauri
cargo test
```

Frontend tests (when present):

```bash
pnpm test
```

## Future features

Future features live in `docs/future/` and are **not implemented unless explicitly asked**. Read the relevant doc before implementing and update its status.
