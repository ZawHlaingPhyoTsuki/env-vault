# Security

> Skeleton - expanded in step 8 hardening pass.

## Threat model

### Protects against

- Casual disk access to the SQLite database file (values are encrypted at rest with AES-256-GCM).
- Backups or copied DB files leaking secret values without the OS keychain master key.

### Does not protect against

- Malware running as the same user with access to the running app, memory, OS keychain, or desktop session.
- An unlocked desktop session where an attacker can use the app.
- Compromised OS/user account, root access, or kernel-level attacks.

## Notes

- The app is local-first and makes zero network requests.
- The encryption key never leaves Rust, is never written to disk/logs/DB, and is not sent to the frontend.
- Decrypted values are only exposed to the frontend for explicit reveal/copy/export flows.
