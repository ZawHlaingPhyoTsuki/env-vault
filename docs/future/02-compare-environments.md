# 02 - Compare Environments

## 1. Goal and user story

**Goal:** Compare two environments to identify key differences.

**User story:** As a developer, I want to compare `dev` and `prod` so I can see missing keys, extra keys, and differing values before deploying.

## 2. Why it's deferred and effort estimate

- **Deferred because:** MVP focuses on CRUD/import/export; comparison adds diff UX and careful masked-value handling.
- **Effort:** **M** - query + diff logic + UI table; optional reveal is sensitive but uses existing decrypt-on-demand pattern.

## 3. Design

### UX flow
- From an environment header, user clicks "Compare".
- User selects source and target environments (same project default; optionally cross-project later).
- App shows diff table: `Only in A`, `Only in B`, `Same`, `Different`.
- Values are masked by default. User can reveal a specific differing value with existing reveal interaction pattern and auto-remask.
- Optional "Missing keys report" view: only keys missing in target.

### UI changes
- Add compare action in `src/App.tsx` header or environment actions.
- New component `src/components/variables/compare-dialog.tsx` or dedicated compare view.
- Diff table with status badges and masked values; follow `docs/DESIGN.md`.

## 4. Data model changes

No schema changes required. Comparison uses existing `variables` rows.

```sql
-- No migration required for MVP-compatible comparison.
-- Optional future index if comparing by key frequently:
CREATE INDEX IF NOT EXISTS idx_variables_env_key ON variables(environment_id, key);
```

## 5. Rust commands and TypeScript API

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub fn compare_environments(
    state: State<'_, AppState>,
    left_environment_id: String,
    right_environment_id: String,
) -> AppResult<EnvironmentComparison>;
```

```ts
// src/lib/types.ts / api.ts
export type CompareStatus = "only_left" | "only_right" | "same" | "different";

export interface CompareRow {
  key: string;
  status: CompareStatus;
  left_masked: string | null;
  right_masked: string | null;
  left_variable_id: string | null;
  right_variable_id: string | null;
}

export interface EnvironmentComparison {
  left_environment_id: string;
  right_environment_id: string;
  rows: CompareRow[];
}

compareEnvironments(leftEnvironmentId: string, rightEnvironmentId: string): Promise<EnvironmentComparison>;
```

## 6. Security considerations

- Default output must be masked; no plaintext values in comparison payload.
- "Same" vs "Different" for secret values requires decryption in Rust. Perform comparison in Rust, return only status + masked placeholders.
- Do not log decrypted values or diff contents.
- Reuse `reveal_variable` for explicit per-value reveal; do not add bulk reveal without confirmation.

## 7. Edge cases

- Empty environments.
- Same environment selected on both sides - block or show no-op.
- Keys with empty values.
- Large environments - keep diff O(n log n) by key map.
- Comments differ but values same - decide whether comment-only diff is shown (recommend separate indicator, default ignore for MVP of feature).

## 8. Acceptance criteria

- User can select two environments and see only-left/only-right/same/different statuses.
- Values are masked by default.
- Missing keys report can be derived/filtered from results.
- No plaintext secrets returned in compare response.

## 9. Test plan

- **Rust unit tests:** diff algorithm for all statuses; same value different ciphertext compares equal after decrypt; empty envs.
- **Command tests:** response contains no plaintext values.
- **Frontend tests:** compare dialog renders statuses, filter for missing keys, reveal interaction works.
- **Manual:** compare dev/stage/prod with known differences.

## 10. Where it hooks into the existing code

- `src-tauri/src/db.rs` - helpers to fetch variable records for two environments.
- `src-tauri/src/crypto.rs` - reuse `ValueCrypto` decrypt for equality check in Rust.
- `src-tauri/src/models.rs` - add comparison DTOs.
- `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` - add/register `compare_environments`.
- `src/lib/api.ts` / `src/lib/types.ts` - TS API/types.
- `src/components/variables/variable-editor.tsx` or App header - add Compare action.
- New `src/components/variables/compare-dialog.tsx` - diff UI.
