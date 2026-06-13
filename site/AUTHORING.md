# Authoring the lab-os handbook

How to write handbook pages so readers can chase depth without overloading skimmers, and never wonder
where a command goes. These are the working rules; the **why** (and the decisions behind them) lives in
the source of truth: [`docs/superpowers/specs/2026-06-13-handbook-backbone-conventions-design.md`](../docs/superpowers/specs/2026-06-13-handbook-backbone-conventions-design.md).

## 1. Depth model — overview + sub-pages + collapsibles

Each handbook section is a **folder**: a skimmable overview page plus nested sub-pages for depth.

```
docs/
  getting-started/
    index.md          # the skimmable overview — what most readers need
    install-git.md    # a sub-page: self-contained depth
    gh-auth.md        # a sub-page: lab-specific procedure
```

- The sidebar renders the folder as an expandable category; the **category label links to `index.md`**
  (the overview). Skimmers stop at the overview; depth-seekers click into a sub-page.
- The overview carries the core and links inward, e.g. `→ [Full walkthrough](./install-git.md)`.

**Sub-page vs. collapsible — the test:**

- **Sub-page** if it's a self-contained procedure or topic someone would land on or link to directly
  (a from-scratch install, a plugin deep-dive, a lifecycle stage).
- **Inline `<details>` collapsible** if it's a 2–5 sentence "new to this?" aside tied to one spot:

```md
<details>
<summary>New to this? What gh auth does</summary>

Two-to-five sentences of context for the reader who needs it, inline, so the
main flow stays skimmable.

</details>
```

## 2. Mark every command — Terminal vs Claude session

Readers must never guess whether a command goes in their shell or into an active Claude Code session.
Use Docusaurus **code-block titles**:

````md
```bash title="Terminal"
gh auth login
```

```text title="Claude session"
/plugin marketplace add WatsonWBlair/lab-claude-plugins
```
````

- `title="Terminal"` — shell commands the reader runs in their OS terminal.
- `title="Claude session"` — prompts and slash-commands (`/plugin`, `/init`, pasted bootstrap prompts)
  typed into a running Claude Code session.

On any page that mixes both, paste this **legend** once, near the top:

```md
> **Reading the commands:** blocks titled **Terminal** run in your OS terminal;
> blocks titled **Claude session** are typed into an active Claude Code session.
```

## 3. Zero-tech readers — link out, fill gaps

Assume some readers have zero technical background.

- **Link** to canonical install/signup docs (Git, GitHub CLI) instead of re-documenting them — they
  stay current on their own.
- **Write** a lab "from scratch" sub-page (per §1) only where the official path is confusing or has
  lab-specific steps (e.g. `gh auth login` + `gh auth setup-git`).
- **Screenshots** are fine in these sub-pages where they materially help; accept they need refreshing as
  tool UIs change, and prefer text + links where a screenshot would just rot.

## 4. When you move or rename a heading

Some headings are linked from elsewhere. The anchors in `working-with-claude` are cited by
`PR-LIFECYCLE.md` and `TROUBLESHOOTING.md`. If a page round moves or renames a cited heading, **update
every referrer in the same commit** — a broken anchor fails the build (`onBrokenLinks: 'throw'`).
