# 03 - Auto-Lock

## 1. Goal and user story

**Goal:** Lock the vault after inactivity or on demand, requiring re-authentication before secrets can be revealed/exported.

**User story:** As a user, I want EnvVault to lock itself when I step away so someone using my unlocked desktop session cannot immediately reveal secrets.

## 2. Why it's deferred and effort estimate

- **Deferred because:** MVP needs core vault flows first; auto-lock requires session/key lifecycle changes and careful UX around key availability.
- **Effort:** **M** - frontend inactivity tracking + backend lock state + re-auth flow; OS keychain prompt behavior varies by platform.

## 3. Design

### UX flow
- Add "Lock now" action and optional inactivity timeout setting (e.g. 5/15/30 min, disabled by default or sensible default with opt-out).
- On lock: clear decrypted UI state, disable reveal/copy/export/import commit, show lock screen.
- Unlock: re-fetch master key from OS keychain (and/or master password when feature 05 exists). On success, resume session.
- On failed unlock: remain locked with clear error; do not generate/reset keys.

### UI changes
- New `src/components/security/lock-screen.tsx`.
- App-level lock state in `src/App.tsx` or a small store.
- Settings UI for timeout (future settings area).
- Ensure dialogs close or become inert on lock.

## 4. Data model changes

No required DB schema change for basic inactivity lock. Settings can be stored in `meta`.

```sql
-- Optional settings keys in existing meta table (no schema migration required)
INSERT INTO meta(key, value) VALUES ('auto_lock_minutes', '15')
ON CONFLICT(key) DO NOTHING;
```

If master-password unlock is added (feature 05), coordinate key-check metadata with that design.

## 5. Rust commands and TypeScript API

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) -> AppResult<()>;

#[tauri::command]
pub fn unlock_vault(state: State<'_, AppState>) -> AppResult<()>; // OS keychain path

#[tauri::command]
pub fn get_lock_status(state: State<'_, AppState>) -> AppResult<bool>;
```

```ts
// src/lib/api.ts
lockVault(): Promise<void>;
unlockVault(): Promise<void>;
getLockStatus(): Promise<boolean>;
```

## 6. Security considerations

- Lock must clear master key from in-memory app state (`AppState.master_key`) and zeroize it.
- Frontend must clear revealed values, pending previews containing sensitive data, and any copy timers/state as appropriate.
- Unlock must not silently create a new key if DB has encrypted data.
- Auto-lock reduces casual exposure but does **not** protect against malware or already-compromised session.

## 7. Edge cases

- Lock during import preview, edit, or export - cancel/close sensitive dialogs and clear plaintext state.
- Keychain prompt canceled - remain locked with clear message.
- System sleep/wake - optionally lock on wake.
- Multiple windows (future) - lock state must be shared.
- Clipboard clear timers firing after lock - should be harmless; ensure no secret retained.

## 8. Acceptance criteria

- User can lock manually and app hides secrets behind lock screen.
- After inactivity timeout, app locks automatically.
- Unlock restores access only after successful key retrieval.
- No new key generation occurs during unlock failure.
- Revealed values are cleared on lock.

## 9. Test plan

- **Rust tests:** lock clears in-memory key; commands requiring key fail while locked; unlock restores key when keychain mock returns key.
- **Frontend tests:** inactivity timer triggers lock; lock screen blocks interactions; revealed state cleared.
- **Manual:** verify behavior on Ubuntu with GNOME Keyring/libsecret.

## 10. Where it hooks into the existing code

- `src-tauri/src/commands.rs` - add lock/unlock/status commands; `ready_key`/`conn` checks should account for locked state.
- `src-tauri/src/lib.rs` - initialize lock-aware state; register commands.
- `src-tauri/src/keychain.rs` - reuse key retrieval; no key generation on unlock when DB has data.
- `src/App.tsx` - lock screen routing, inactivity timer, clear selected sensitive UI state.
- `src/components/variables/variable-editor.tsx` - clear revealed timers/state on lock.
- `src/components/variables/import-dialog.tsx` - close/clear preview values on lock.
- `src/lib/api.ts` / `src/lib/types.ts` - lock API wrappers.
