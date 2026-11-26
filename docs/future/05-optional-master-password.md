# 05 - Optional Master Password

## 1. Goal and user story

**Goal:** Allow users to add an optional master password on top of the OS keychain key for stronger protection.

**User story:** As a security-conscious user, I want to require a master password to unlock EnvVault so that access to my OS account/keychain alone is not enough.

## 2. Why it's deferred and effort estimate

- **Deferred because:** MVP uses OS keychain for simplicity; master password introduces KDF, key wrapping, migration, and unlock UX complexity.
- **Effort:** **L** - key hierarchy design, Argon2id KDF, migration of existing encrypted data or key wrapping, unlock/change-password flows.

## 3. Design

### UX flow
- Settings: "Enable master password". User enters password + confirmation.
- On enable: derive KEK via Argon2id; wrap/encrypt the existing data-encryption key (DEK) and store wrapped key metadata.
- On app start when enabled: prompt for master password, derive KEK, unwrap DEK, keep DEK in memory only.
- Support change password and disable password (with current password confirmation).
- Coordinate with auto-lock: lock clears DEK; unlock requires password.

### UI changes
- New settings/security section.
- Unlock/password prompt dialog/screen.
- Change/disable password dialogs with warnings.

## 4. Data model changes

Use `meta` for key-wrapping metadata; avoid breaking existing schema. Prefer key wrapping (wrap DEK) over re-encrypting all variable values.

```sql
-- Migration vX (example, key-wrapping approach)
-- meta keys:
--   key_mode = 'keychain' | 'keychain+password'
--   kdf = 'argon2id'
--   kdf_salt = <base64>
--   kdf_memory_kib = '65536'
--   kdf_iterations = '3'
--   kdf_parallelism = '1'
--   wrapped_dek_nonce = <base64>
--   wrapped_dek_ciphertext = <base64>
--   wrapped_dek_version = '1'

-- No variable row re-encryption required if DEK wrapping is used.
INSERT INTO meta(key, value) VALUES ('schema_version', 'X')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
```

## 5. Rust commands and TypeScript API

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub fn get_security_status(state: State<'_, AppState>) -> AppResult<SecurityStatus>;

#[tauri::command]
pub fn enable_master_password(state: State<'_, AppState>, password: String) -> AppResult<()>;

#[tauri::command]
pub fn unlock_with_master_password(state: State<'_, AppState>, password: String) -> AppResult<()>;

#[tauri::command]
pub fn change_master_password(state: State<'_, AppState>, current_password: String, new_password: String) -> AppResult<()>;

#[tauri::command]
pub fn disable_master_password(state: State<'_, AppState>, current_password: String) -> AppResult<()>;
```

```ts
export interface SecurityStatus {
  key_mode: "keychain" | "keychain_plus_password";
  locked: boolean;
}

getSecurityStatus(): Promise<SecurityStatus>;
enableMasterPassword(password: string): Promise<void>;
unlockWithMasterPassword(password: string): Promise<void>;
changeMasterPassword(currentPassword: string, newPassword: string): Promise<void>;
disableMasterPassword(currentPassword: string): Promise<void>;
```

## 6. Security considerations

- Use **Argon2id** with sane parameters; store salt/params in meta.
- Never store plaintext password. Zeroize password and derived KEK/DEK in memory where practical.
- Prefer wrapping the existing DEK to avoid bulk value re-encryption and reduce migration risk.
- Constant-time/AEAD failure handling: treat unwrap failure as invalid password; do not leak details.
- Coordinate with OS keychain: decide whether keychain still stores wrapped DEK or raw DEK. Recommended: store only wrapped DEK when password enabled.

## 7. Edge cases

- Forgotten password - data unrecoverable without backup; warn clearly during enable.
- Enabling password when keychain unavailable - block with clear error.
- App crash mid-migration - migration must be transactional/recoverable.
- Password change after backup format changes - backup metadata must include key mode.
- Unicode/whitespace passwords - accept as entered; no trimming that changes meaning.

## 8. Acceptance criteria

- User can enable optional master password.
- App requires password to unlock when enabled.
- Existing variables remain readable after migration.
- User can change/disable password with current password.
- Wrong password does not unlock and does not corrupt data.

## 9. Test plan

- **Rust tests:** Argon2id param encode/decode; wrap/unwrap DEK round trip; wrong password fails; migration from keychain-only to password-enabled preserves data.
- **DB tests:** meta keys written/read correctly; transactional migration rollback on failure.
- **Frontend tests:** enable/unlock/change/disable flows; validation and error states.
- **Manual:** restart app, lock/unlock, verify old data decrypts.

## 10. Where it hooks into the existing code

- `src-tauri/src/crypto.rs` - add KDF, key wrapping/unwrapping, `SecurityStatus` helpers; keep trait extensible.
- `src-tauri/src/keychain.rs` - store wrapped or raw key material depending on mode.
- `src-tauri/src/db.rs` - meta helpers/migrations for key mode and KDF params.
- `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` - startup unlock flow, new commands, lock integration.
- `src/lib/api.ts` / `src/lib/types.ts` - security status/types/commands.
- `src/App.tsx` - unlock screen/state before main UI.
- `docs/SECURITY.md` - update threat model when implemented.
