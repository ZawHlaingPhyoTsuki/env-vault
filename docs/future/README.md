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

1. **01-global-search.md** - read-only feature, no key-handling changes, builds on existing DB/query patterns.
2. **02-compare-environments.md** - read-only comparison over existing decrypted-on-demand flows.
3. **03-auto-lock.md** - session/key lifecycle change; needed before stronger backup/password flows are exposed.
4. **05-optional-master-password.md** - extends key hierarchy; design carefully before backup format depends on it.
5. **04-encrypted-backup.md** - depends on stable data model and key hierarchy decisions.
