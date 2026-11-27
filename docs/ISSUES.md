# Issues

## 1. [Fixed] Import preview sends every secret to the screen code
- **Where:** `src-tauri/src/commands.rs` (`preview_env_import`), `src-tauri/src/models.rs` (`ImportPreviewRow.value`), `src/components/variables/import-dialog.tsx`
- **Problem:** The Rust backend puts the real secret text into every preview row (`value: Some(v.value.clone())`), even when the row is still hidden. Pasted secrets cross the IPC boundary in the preview payload.
- **Fix:** Do not send `value` in the preview response. Remove preview reveal, or add a separate per-row reveal command. Keep preview masked with no plaintext in the normal payload.
- **If skipped:** Secrets cross the app boundary more often than needed; any future weaker CSP/webview bug can expose pasted secrets.

## 2. [Resolved by removing the feature] Export writes a secret file that is briefly readable by others
- **Where:** `src-tauri/src/commands.rs` (`export_env_file`)
- **Problem:** On Unix, the file is written first, then chmodded to `0600`. Between write and chmod, default umask may make it readable by other users.
- **Fix:** Create the file with owner-only mode from the start using `OpenOptions` with `mode(0o600)`, then write content.
- **If skipped:** On shared machines with normal umask, exported secrets can be read during the short window.

## 3. [Fixed] Duplicating an environment can leave a half-copied result
- **Where:** `src-tauri/src/db.rs` (`duplicate_environment`)
- **Problem:** Creates target environment then inserts variables one by one with no transaction. A mid-loop failure leaves a partially copied environment.
- **Fix:** Wrap the whole operation in one transaction; pass `&tx` through create/copy and commit once.
- **If skipped:** Disk/DB errors can leave incomplete duplicates that look trustworthy.

## 4. [Resolved by removing the feature] Export command accepts any file path from the screen code
- **Where:** `src-tauri/src/commands.rs` (`export_env_file`), `src-tauri/src/lib.rs` command registration
- **Problem:** Command writes to any frontend-supplied path. Tauri custom commands are not fs-scoped by default; this is a dangerous write primitive if webview is compromised.
- **Fix:** Move save-dialog+write into a single Rust command that owns the chosen path, or avoid raw path-write command.
- **If skipped:** A compromised webview could overwrite user-writable files with secret content.

## 5. [Fixed] Compare has no test that would catch a wrong result
- **Where:** `src-tauri/src/commands.rs` (`compare_environments`)
- **Problem:** Compare decrypts and classifies secrets, but there are no tests for compare behavior or masked-only output.
- **Fix:** Add one Rust test covering same/different/only-left/only-right and asserting no plaintext in response.
- **If skipped:** Compare regressions can silently give wrong deployment decisions.
