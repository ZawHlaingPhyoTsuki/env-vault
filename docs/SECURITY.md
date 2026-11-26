# Security

## Overview

EnvVault is a local-first desktop app for storing `.env` values in an encrypted SQLite vault. All crypto and database access lives in Rust; the frontend calls Tauri commands only.

## Threat model

### Protects against

- **Casual disk access / DB leakage:** variable values are encrypted at rest in SQLite using AES-256-GCM. A copied `envvault.db` file does not expose secret values without the master key.
- **Backups leaking the DB file:** backups or copies of the database file remain encrypted at the value level.
- **Accidental UI exposure:** values are masked by default in list views; reveal/copy/export are explicit user actions.

### Does not protect against

- **Malware running as the same user:** attackers can read process memory, access the OS keychain, inspect the desktop session, or hook the app.
- **An unlocked desktop session:** anyone with access to the running/unlocked app can reveal or export secrets.
- **Compromised OS/user/root access:** kernel-level attacks, rooted devices, or full account compromise are out of scope.
- **Memory forensics while secrets are in use:** decrypted values are briefly present in memory when revealed/copied/imported/exported.

## Encryption and key handling

- **Master key:** 32 random bytes generated on first launch, stored in OS keychain (service `envvault`, account `master-key`; debug builds use `envvault-dev`). Key material is wrapped in `zeroize` where practical.
- **Value encryption:** each value encrypted with AES-256-GCM and a fresh random 96-bit nonce. Ciphertext and nonce are stored per variable.
- **Key lifecycle:** on startup, app loads key from OS keychain. If the key is missing/unavailable while encrypted data exists, the app shows a clear error and **does not** generate a new key.
- **Plaintext boundary:** decrypted values are returned to frontend only for explicit reveal/copy/export/import-preview verification flows. List commands return masked placeholders.

## Tauri hardening

- **CSP:** strict CSP in `src-tauri/tauri.conf.json`; no remote script origins; connect limited to Tauri IPC.
- **Capabilities:** minimal permissions only - `core:default`, `clipboard-manager:allow-read-text`, `clipboard-manager:allow-write-text`, `dialog:allow-save`. No broad filesystem scope.
- **No network features:** no telemetry, analytics, auto-update, or remote scripts. App is intended to work fully offline.
- **Exports:** `.env` export files are written by Rust and set to `0600` permissions on Unix.

## Logging and data handling

- **No secret logging:** do not log env values, decrypted plaintext, keys, or clipboard secrets, even in debug builds.
- **No browser secret storage:** no secrets in `localStorage`/`sessionStorage`.
- **Frontend memory:** revealed values are kept only as needed and auto re-masked after ~15 seconds. Clipboard copy clears after ~30 seconds only if clipboard still contains the copied secret.

## Known limitations / follow-ups

- No auto-lock/inactivity lock yet (see `docs/future/03-auto-lock.md`).
- No optional master password yet (see `docs/future/05-optional-master-password.md`).
- No encrypted backup/export-vault yet (see `docs/future/04-encrypted-backup.md`).
- Threat model assumes a non-compromised local user session.
