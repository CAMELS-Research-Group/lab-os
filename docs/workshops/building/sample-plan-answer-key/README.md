# Sample plan — Task 3 answer key

**Status:** draft
**Audience:** Facilitator running the Building session. Reference solution for **Task 3** of
[`../sample-plan.md`](../sample-plan.md) (add a planning surface).

These three files are what a participant **produces** in Task 3. They live here as a reference so you
can show the target shape during the demo, or paste a known-good version if a run stalls.

> **Do not commit these to the live paths in the upstream.** Task 3 asks participants to *create*
> `templates/backlog-item.template.md`, `site/docs/planning/backlog.md`, and a root `BACKLOG.md` in
> their own fork. If those files already exist upstream, every fork inherits Task 3 pre-done and the
> exercise is gone. Keep them here under `docs/` (ENG-tier, off the site); copy them to the live paths
> only inside a throwaway demo fork or worktree.

## The files

| Answer-key file | Participant's target path (in their fork) |
|---|---|
| `backlog-item.template.md` | `templates/backlog-item.template.md` |
| `planning-backlog.md` | `site/docs/planning/backlog.md` (+ a `sidebars.ts` entry) |
| `BACKLOG.md` | `BACKLOG.md` (repository root) |

The page is named `planning-backlog.md` here only to avoid a case-insensitive filename clash with
`BACKLOG.md` in this folder (Windows); it becomes `backlog.md` at its target path.

## Using it in the demo

1. In a throwaway fork or worktree, copy the three files to their target paths above and add a
   `sidebars.ts` entry for the new page (`'planning/backlog'` under a new top-level **Planning**
   category, or wherever reads well).
2. Run the Task 3 gate: `cd site && npm run build`. The broken-link-throw proves the page and its
   sidebar wiring resolve.
3. Show the diff. The point participants miss: the build proves the page *resolves*, not that the
   workflow it documents is *useful* — judge that by reading it.
