# 01 - Global Search

## 1. Goal and user story

**Goal:** Let users find where an environment key is used across all projects and environments.

**User story:** As a developer, I want to search for a key like `STRIPE_KEY` across all projects/environments so I can see where it is defined and avoid missing updates.

## 2. Why it's deferred and effort estimate

- **Deferred because:** MVP prioritizes core vault CRUD/import/export; global search adds cross-project query UX and secure value-search considerations.
- **Effort:** **M** (medium) - DB query + command + UI, with careful security decisions for optional value search.

## 3. Design

### UX flow
- Add a global search affordance in sidebar/top bar (or extend `Ctrl/Cmd+K` to a global mode; current MVP reserves `Ctrl/Cmd+K` for in-environment filter).
- User types key name (default). Results grouped by project > environment, showing key, masked value preview, and comment snippet.
- Clicking a result navigates to that project/environment and focuses/highlights the variable row.
- Optional advanced toggle: "Search values" (off by default). If enabled, require explicit confirmation/warning and decrypt only as needed; never index plaintext values at rest.

### UI changes
- `src/components/layout/sidebar.tsx` or new `src/components/search/global-search-dialog.tsx`.
- Add command-palette/dialog with search input, result list, keyboard navigation.
- Optional setting in future settings UI; MVP has no settings screen.

## 4. Data model changes

No required schema change for key-name search (keys are plaintext). Add indexes for performance.

```sql
-- Migration v2 (example)
CREATE INDEX IF NOT EXISTS idx_variables_key ON variables(key);
CREATE INDEX IF NOT EXISTS idx_variables_key_nocase ON variables(key COLLATE NOCASE);
INSERT INTO meta(key, value) VALUES ('schema_version', '2')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
```

If secure value search is later added, do **not** store plaintext searchable values. Consider opt-in blind index/HMAC with separate key material and explicit security review.

## 5. Rust commands and TypeScript API

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub fn search_variables(state: State<'_, AppState>, query: String, include_values: bool) -> AppResult<Vec<GlobalSearchResult>>;
```

```ts
// src/lib/api.ts
export interface GlobalSearchResult {
  project_id: string;
  project_name: string;
  environment_id: string;
  environment_name: string;
  variable_id: string;
  key: string;
  masked_value: string;
  comment: string | null;
}

searchVariables(query: string, includeValues?: boolean): Promise<GlobalSearchResult[]>;
```

## 6. Security considerations

- Key-name search is safe over plaintext metadata.
- Value search is sensitive: default off. If implemented, decrypt in Rust only for matching candidates and avoid logging query/value matches.
- Do not expose decrypted values in list results; return masked values unless user explicitly reveals in the target row.
- Rate-limit/limit results to avoid accidental large decrypt operations.

## 7. Edge cases

- Empty/short queries - require minimum length (e.g. 2 chars) to avoid noisy scans.
- Duplicate keys across many environments - paginate/limit results.
- Case sensitivity - prefer case-insensitive key search.
- Deleted/renamed items between search and navigation - handle not-found gracefully.
- Value search with large vault - may be slow; show progress/cancel.

## 8. Acceptance criteria

- User can search by key name across all projects/environments.
- Results show project, environment, key, masked value, comment.
- Selecting a result navigates to the correct environment/variable.
- No plaintext values are exposed by default.
- Search works offline and uses no network calls.

## 9. Test plan

- **Rust DB tests:** query returns matches across projects/envs; case-insensitive behavior; limit/pagination; no match.
- **Rust command tests:** masked results only; include_values false never decrypts.
- **Frontend tests:** dialog renders results, keyboard navigation, navigation callback.
- **Security test:** ensure logs/errors do not contain secret values.

## 10. Where it hooks into the existing code

- `src-tauri/src/db.rs` - add `search_variables` query joining projects/environments/variables.
- `src-tauri/src/models.rs` - add `GlobalSearchResult` DTO.
- `src-tauri/src/commands.rs` - add `search_variables` Tauri command; register in `src-tauri/src/lib.rs`.
- `src/lib/types.ts` / `src/lib/api.ts` - add TS type and invoke wrapper.
- `src/components/layout/sidebar.tsx` or new search dialog component - UI entry point.
- `src/App.tsx` - navigation/selection state when opening a result.
- `docs/DESIGN.md` - follow existing dialog/table/kbd styling rules.
