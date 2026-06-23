# Backlog

Raw ideas and shaped work for this fork. New items get here via the workflow on the
**Planning surface** page (`site/docs/planning/backlog.md`); each follows the shape in
`templates/backlog-item.template.md`.

Read the **Index** first — it is the "what's ready right now" surface. Full items follow below it.

## Index

| id | title | size | status |
|---|---|---|---|
| B1 | Dark-mode toggle in the navbar | S | ready |
| B2 | Search across the handbook | M | ready |
| B3 | "Edit on GitHub" points at my fork | S | done |

## Inbox

<!-- Raw, unshaped ideas. Move one up to Items once it has a Problem and a Done-when. -->

- A printable one-page "first session" checklist.
- A short screen-recording of the workspace bootstrap.

## Items

## B1 — Dark-mode toggle in the navbar

- **Problem:** readers in low light can't switch theme from the page; they fall back to OS settings.
- **Who it helps:** anyone reading the handbook at night.
- **Value:** small, visible polish — a good first autonomous task.
- **Rough size:** S — one sitting.
- **Done when:** `cd site && npm run build` is green and the navbar shows a working light/dark control.
- **Depends on:** —
- **Status:** ready

## B2 — Search across the handbook

- **Problem:** there is no way to search the handbook; readers scroll the sidebar to find a page.
- **Who it helps:** returning readers who know the term but not the page.
- **Value:** removes the most common navigation friction once the handbook has more than a few pages.
- **Rough size:** M — a few sittings (pick and wire a search plugin, then verify it indexes).
- **Done when:** the built site exposes a search box that returns this page for the query "backlog".
- **Depends on:** —
- **Status:** ready

## B3 — "Edit on GitHub" points at my fork

- **Problem:** the inherited "Edit this page" links pointed at the upstream, not this fork.
- **Who it helps:** anyone trying to propose an edit from the rendered site.
- **Value:** closes the loop from reading to contributing.
- **Rough size:** S — one sitting.
- **Done when:** the footer "Edit this page" link on any doc opens this fork's GitHub editor.
- **Depends on:** —
- **Status:** done <!-- this is what Task 1 of the sample plan accomplishes -->
