---
format: aep.planning-md/3
id: approval-record:ess-issues-batch-1
kind: approval-record
status: approved
title: Open-issue batch 1 runs as one wave and one pull request
summary: Operator request of 2026-10-28, approved under the grant by the orchestrator
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:43:03Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

The operator asked on 2026-10-28: analyse the open ESS issues, close duplicates and issues already
surpassed by merged fixes with evidence, fix the rest in batches (several worktrees merged into one
integration branch, one pull request per batch to `main`), repeat until no issue is open, then
release. Waves for this work were approved in advance. The steward session confirmed batch 1 as
planned ("Batch 1 as planned; your ess/33 call stands"), then handed the work to the orchestrator,
which approved it under the operator's grant on 2026-10-28 ("drive the open beyond10x/ess GitHub
issues (38 open) to zero in batches, then release").

This run is non-interactive: the coordinator is a sub-agent with no operator turn, so the stage-1
proposal stop is recorded here and the wave runs.

## Units

The triage and the unit table are in `docs/plan/wave-ess-issues-1.md`. Batch 1 takes the S and M
issues that need no new `ess/N` source format: #643, #644, #645, #647, #648, #651, #652, #653, #659,
#661, #664, #665, #671, #673, #685, #686, #687, #688, #693, #694, plus the two #554 synthesis
defects and the store drift (D136). Every story serves `vision:O2` and carries a fit review and a
decision before its unit is dispatched. Decisions D132-D138 of 2026-10-28 are recorded beside this
one; a unit D135 takes out leaves the batch.

## Conditions from the orchestrator (2026-10-28)

- Until the `0.75.0` tag exists: store and planning work only, no build. After it: rebase the
  integration branch on `main`, at most two builds at once and only at 40 GB free or more.
- Each tree keeps its own `target/`; no shared `CARGO_TARGET_DIR`.
- The synthesis unit (#645, #693) builds last, after `0.75.0` lands on `main`.
- The release version (0.76.0 unless a later one exists) is confirmed with the orchestrator before
  tagging.

## Commits this approval authorises

One commit per unit on its unit branch, the merges of green units into `integration/ess-issues-1`,
the store commits on that branch, one pull request to `main` and its merge through the App after the
`0.75.0` tag exists. Not a tag, a release or the next batch.

Decider: operator

## Revision (2026-10-28)

A wave holds at most four units (the orchestrator's process rule), so the issues above run as
batches 1-3 of `docs/plan/wave-ess-issues-1.md` § Batches, one pull request each: batch 1 is U1-U4
(#685 #686 #687 #688, #643 #644, #664 #665 #671 and the #666 fixture, #651 #653). This record
authorises the same commits for each of the three batches.
