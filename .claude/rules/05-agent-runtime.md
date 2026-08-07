# Agent Runtime

HARD RULE. Binds any lab-os asset that hosts a guardrailed local coding-agent runtime. The
contract below is enumerable from this file alone — no need to read runtime source. **No asset
in this repo hosts one today: the rule is forward-binding — the contract a first runtime must
meet, not a description of something shipped.** Doc tiers/budgets: `04-docs.md`. Logging:
`03-logging.md`.

## The invariant

**The host drives the runtime; it never becomes it.** The application observes and
commands the agent engine (start/stop/status) over a same-host loopback control seam and
enforces every guardrail below. The engine owns no authoritative state — every state change
flows back through the host's API into its own store. A reasoning brain (if hosted) runs on
a **local model only** (Ollama); it has **no direct `claude` spawn** — its sole path to
coding is the delegate connector through the host's guarded run API.

## Execution guardrails (enforced on every spawn surface)

| Guardrail | Contract |
|---|---|
| **Max-clean invocation** | Coding runs the official `claude` CLI with plain `claude -p` (print mode) on the base Max allowance. **Never `--bare`.** No path drives a Max/Pro OAuth token through a third-party tool; **no metered API key** is ever introduced. The brain runs on local Ollama. $0-by-construction. |
| **Per-run caps** | Every run carries a request (turn) cap and a dollar cap. A cap reached → run records `halted` with the matching halt token; the cap trips deterministically, before overspend. A wall-clock deadline halts the same way. |
| **Max-quota halt (first-class)** | Exhausting the Max allowance is a **named terminal halt** (`halted (max-quota)`), not an error and **never** a fallback to a metered path. Quota exhaustion ends the run; it does not silently re-route spend. |
| **Halt contract** | A halt is a controlled-vocabulary terminal token, not a crash: `cap:requests` / `cap:cost` / `max-quota` / `deadline` / `cancelled` / `gate-red` / `error` / `merge-needed`. Status + token persist on the run row; the run is recoverable/reportable from the row alone. |
| **Self-run gate (unpiped)** | The runtime's own verification gate must be run **unpiped** — piping (`gate \| tail`) swallows the exit code and lets a red gate read green. A red gate is not done. |
| **Per-repo permission allowlist** | A run's tool/permission scope is per-repo policy resolved host-side, fail-closed (read-only by default) — **not** a per-run override the caller can pass. The only way to widen scope is to change the host's per-repo policy. |
| **Commit destination (fail-closed isolation)** | Where a run's commits land is per-repo policy resolved host-side (`agents.commit_destination`), **fail-closed to `merge-to-head`**: the run executes in an isolated worktree + temp branch off HEAD and integrates into the working branch **only** at a success terminal — a non-red self-gate (green, or the honest no-gate unverified success) — via a fast-forward merge-back (`--ff-only` — never an auto-merge-commit, never a clobber of host work). A non-fast-forwardable integration ends `halted (merge-needed)` with the temp branch preserved for manual integration. `head` (in-place) is an explicit per-repo opt-out, never a degradation path. |

## Approval gate

- **Synchronous, deny-by-default.** A gated action pauses at the gate and proceeds only on
  explicit approval; the default outcome with no approver is **deny**.
- **Crash-safe via the row.** The gate is a `pending_approvals` SQLite row — **no LangGraph,
  no checkpointer**. The row is the durable, exactly-once mechanism: a pending gate survives
  an app/host restart and resolves from the row.
- **Risk tags.** Enrolled actions are a code-defined registry tagged `spend` / `destructive` /
  `acts-as-operator`; the gate triggers on tagged actions. `acts-as-operator` is reserved for
  actions taken under the **operator's own identity** — it is **not** the default for agent
  output, which posts under the agent's bot identity.

## Identity & enablement

- **Agents act under their own bot identity.** Each agent run posts/acts under its **own**
  GitHub App / bot identity (the `[bot]` pattern), never the operator's account. Provisioning
  that identity is part of enabling the runtime.
- **Disabled-by-default.** The runtime ships **off**: no schedule specs populated, connector
  unregistered, no default-on config flipped. **Enabling is an explicit operator action** —
  enabling + provisioning the bot identity is a deliberate, documented operator step, never a
  default or an inherited-on state. Claude Code auto-mode self-authorization is the correct
  posture and is preserved; it does not waive any guardrail above.
