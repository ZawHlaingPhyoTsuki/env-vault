# 04 - Encrypted Backup

## 1. Goal and user story

**Goal:** Export and import the entire vault as a single password-encrypted backup file.

**User story:** As a user, I want to create an encrypted backup of all my projects/environments/variables so I can restore them on another machine or after data loss.

## 2. Why it's deferred and effort estimate

- **Deferred because:** MVP focuses on local vault CRUD; backup requires a versioned backup format, password KDF, restore semantics, and careful merge/replace behavior.
- **Effort:** **L** - format design, crypto, migration/versioning, restore conflict handling, and thorough tests.

## 3. Design

### UX flow
- **Export backup:** user chooses destination path via save dialog, enters backup password + confirmation. App writes encrypted backup file.
- **Import backup:** user picks backup file, enters password. App validates format/version, shows summary (projects/envs/vars), asks restore mode: **Merge** or **Replace**.
- **Merge:** import missing projects/envs/vars; on conflicts, prompt or apply deterministic rule (recommend: skip/rename/overwrite per entity type, default skip with report).
- **Replace:** confirm destructive action, replace vault contents after successful decrypt/validation.

### UI changes
- Settings/toolbar actions: "Export backup", "Import backup".
- Password/confirm dialogs; restore mode dialog; summary/report screen.
- Progress indicator for large backups.

## 4. Data model changes

No change to core vault tables required. Backup format is external and versioned. Optionally record last backup timestamp in `meta`.

```sql
-- Optional meta keys
-- last_backup_at = RFC3339 timestamp
-- backup_format_version_known = '1'

-- Backup file envelope (conceptual, not DB table):
-- {
--   "format": "envvault-backup",
--   "version": 1,
--   "kdf": {"name":"argon2id","salt":"...","memory_kib":65536,"iterations":3,"parallelism":1},
--   "cipher": {"name":"aes-256-gcm","nonce":"..."},
--   "ciphertext":"..." // encrypted JSON payload
-- }
```

Payload should include projects, environments, variables (decrypted values inside encrypted envelope only), timestamps, positions, comments, and export metadata.

## 5. Rust commands and TypeScript API

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub fn export_vault_backup(state: State<'_, AppState>, path: String, password: String) -> AppResult<BackupSummary>;

#[tauri::command]
pub fn inspect_vault_backup(path: String, password: String) -> AppResult<BackupSummary>;

#[tauri::command]
pub fn import_vault_backup(state: State<'_, AppState>, path: String, password: String, mode: RestoreMode) -> AppResult<RestoreReport>;
```

```ts
export type RestoreMode = "merge" | "replace";

export interface BackupSummary {
  format_version: number;
  projects: number;
  environments: number;
  variables: number;
  created_at: string;
}

export interface RestoreReport extends BackupSummary {
  mode: RestoreMode;
  imported_projects: number;
  imported_environments: number;
  imported_variables: number;
  skipped: number;
  warnings: string[];
}

exportVaultBackup(path: string, password: string): Promise<BackupSummary>;
inspectVaultBackup(path: string, password: string): Promise<BackupSummary>;
importVaultBackup(path: string, password: string, mode: RestoreMode): Promise<RestoreReport>;
```

## 6. Security considerations

- Backup file must be encrypted with password-derived key (Argon2id + AES-256-GCM). Never write plaintext backup.
- Password is never logged/stored. Zeroize derived keys.
- Export requires unlocking vault and decrypting data in Rust; keep plaintext only in memory during export.
- Import must validate format version, checksums/AEAD authenticity, and schema before applying changes.
- Replace mode is destructive - require explicit confirmation and backup/current-state warning.
- Align with feature 05 key hierarchy if master password is enabled; backup password is separate unless explicitly designed otherwise.

## 7. Edge cases

- Wrong password / tampered backup - fail with clear "invalid password or corrupted backup".
- Unknown backup version - reject safely.
- Merge conflicts (same project/env/key names) - deterministic, reported behavior.
- Partial import failure - use DB transaction; rollback on failure.
- Very large backups - stream/chunk where possible; avoid excessive memory.
- Importing into locked vault - require unlock first.

## 8. Acceptance criteria

- User can export entire vault to an encrypted backup file.
- User can inspect and import backup with correct password.
- Wrong password/corrupted file is rejected without modifying vault.
- Merge and replace modes behave as documented and report results.
- Backup format is versioned.

## 9. Test plan

- **Rust crypto/format tests:** round-trip export/import, wrong password, tamper detection, unsupported version.
- **DB tests:** merge conflict behavior; replace is transactional; rollback on simulated failure.
- **Frontend tests:** export/import dialogs, confirmation for replace, report rendering.
- **Manual:** backup on one profile/machine, restore on another; verify values/comments/order.

## 10. Where it hooks into the existing code

- `src-tauri/src/crypto.rs` - add backup KDF/encrypt/decrypt helpers (or separate `backup_crypto` module).
- `src-tauri/src/db.rs` - add vault export snapshot/import transaction helpers.
- `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` - new commands and registration.
- `src-tauri/src/models.rs` - backup summary/report/restore mode types.
- `src/lib/api.ts` / `src/lib/types.ts` - invoke wrappers and TS types.
- `src/App.tsx` or future settings UI - backup actions.
- `src-tauri/capabilities/default.json` - may need dialog open/save permissions already present; keep fs scope minimal, prefer Rust file write/read via explicit paths.
- `docs/SECURITY.md` - document backup threat model when implemented.
