---
sidebar_position: 5
title: Review
description: The Review stage of the lab's SDD lifecycle — the human checkpoint, where an outsider's-eye read catches what the automated gate cannot.
---

# Review — the human checkpoint

**Review** is the lifecycle's *human* checkpoint, the counterpart to the automated
[Verify](./verify.md) gate. It catches what the gate cannot — failure-recovery bugs,
self-referential coverage gaps, credential-path assumptions. Pre-merge review is **load-bearing,
not optional**.

- **Multi-agent first pass + audit pass.** Cheaper model first; escalate to the stronger model both
  when the first pass returns *zero* findings (rubber-stamp risk) and to recheck severities. The
  audit pass has caught real latent bugs the gate could not see.
- **Outsider's eye.** Review what's there, not what's meant. If you helped author it, declare that
  and harden the review.
- **For a requested review, the review *is* the deliverable** — post it; don't ask permission to
  deliver what was asked. Unsolicited posts under your name still need the approval gate in the
  [global `CLAUDE.md` template](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/global-CLAUDE.template.md).
- **PR template, merge bar, and the documented exception for repos with a single maintainer:**
  [`PR-LIFECYCLE.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/PR-LIFECYCLE.md) for the
  lifecycle; hard rules in
  [`01-workflow.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/.claude/rules/01-workflow.md).

→ Next: [Close](./index.md#close) the work · back to the [lifecycle overview](./index.md).
