---
paths:
  - ".github/workflows/**"
  - "projects/*/.github/workflows/**"
---

# Path-scoped rule

Loads only when a session touches a matching path: checked against its
per-file budget, kept out of the always-loaded aggregate.
