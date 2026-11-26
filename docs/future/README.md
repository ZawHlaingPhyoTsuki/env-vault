# Future features

These features are **not part of the MVP**. Do not implement them unless explicitly asked. Each document contains the design and implementation hook points.

| Feature | Doc | Effort | Status |
|---|---|---:|---|
| Global search | [01-global-search.md](./01-global-search.md) | M | Not started |
| Compare environments | [02-compare-environments.md](./02-compare-environments.md) | M | Not started |
| Auto-lock | [03-auto-lock.md](./03-auto-lock.md) | M | Not started |
| Encrypted backup | [04-encrypted-backup.md](./04-encrypted-backup.md) | L | Not started |
| Optional master password | [05-optional-master-password.md](./05-optional-master-password.md) | L | Not started |

## Suggested implementation order

1. **01-global-search.md** - read-only feature over existing plaintext metadata; no key-handling changes.
2. **02-compare-environments.md** - read-only comparison using existing decrypt-on-demand flows; builds UI patterns for masked diffs.
3. **03-auto-lock.md** - session/key lifecycle change; establishes lock/unlock UX before password-based features.
4. **05-optional-master-password.md** - defines key hierarchy/migration that backup/restore should align with.
5. **04-encrypted-backup.md** - depends on stable data model and key hierarchy; benefits from lock/password decisions.
