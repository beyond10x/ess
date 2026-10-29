---
format: aep.planning-md/3
id: approval-record:ess-issues-batch-1
kind: approval-record
status: approved
title: Open-issue batch 1 runs as one wave and one pull request
summary: Operator request of 2026-10-28, approved under the grant by the orchestrator
revision: 6
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

## Gate (D171)

Decided by the coordinator as D171 (2026-10-28), under the operator rule of the same day: units run only targeted tests, and one local gate runs once every unit is on `integration/ess-issues-1`, before the pull request is opened with auto-merge.

The local gate is:
- the whole test set (`--lib` plus every `--test` without required features) of each touched library crate: `ess-http`, `ess-gen`, `schema-contract`, `ess-composition`, `ess-domain`;
- named lanes for `ess-conformance`, `ess-cli`, `ess-synth`, `ess-compiler` and `ess-entity-runtime`, including the four table tests (`synthesis_suite_bytes_table` and its three siblings, `selection_precedence_table`, `lowered_definitions_table`, `adversary_273_pass1`);
- every step of `task projection-check` (`cargo xtask generate --check`, `schema --check`, `whats-changed --check`, `docs`, `cli-reference --check`, `diagnostics --check`, `format-history --check`, `site-data --check`, `admonitions`);
- `cargo clippy -p <crate> --all-targets --locked -- -D warnings` for each of the nine touched crates, and `cargo fmt --check` for them.

The literal `task check` is not run locally: it builds every test binary (about 92 GiB for `ess-conformance` alone), which this machine cannot hold, and `AGENTS.md` gives the full gate to CI. CI runs it on the pull request before the auto-merge lands.

## Gate amendment (2026-10-29)

PR 701's CI failed on three `ess-synth` tests (`feasibility`, `adv319`, `http_binding_rust` go) that compare the committed gatepass example trees byte for byte: #688 added `minimum` and `x-ess-invariants` to `pass-service.openapi.json`, and the D171 gate ran `ess-synth` only in named lanes. The two files were regenerated on the queue branch.

From this batch on, `ess-synth`'s whole test set is part of the local gate whenever a change alters a committed example tree under `generated/` or a projection (orchestrator, 2026-10-29). `AGENTS.md` § The package-gate subset carries the rule.

## Batch 2 (D183, 2026-10-29)

The orchestrator approved batch 2 under the operator's grant on 2026-10-29 as D183: "Start batch 2
(D183) with the same shape: one integration branch, at most 4 units, targeted tests per unit, one
local gate at the end (including ess-synth's whole set when examples or projections move), the
replay against the 5 adopters if output can move, one queue PR, one release."

- Units: U5 (#652, #673), U6 (#647, #648), U7 (#661), U8 (#659), as `docs/plan/wave-ess-issues-1.md`
  § Batches plans them. D135 still holds for U6's #648 and U7: a unit with no non-breaking form
  leaves the batch with a `decision-blocker`.
- Integration branch `integration/ess-issues-2`, cut from `main` at the 0.76.0 merge. The release
  pull request comes from a `queue/` branch, as for 0.76.0.
- Builds: one shared target, capped at 20G (the orchestrator's budget; batch 1 reached 22G).
- This section authorises, beyond the commits above, the release of batch 2 through the
  repository's own process: version bump, annotated tag through the bot, release workflow, the
  verified draft and its publication.

Decider: operator
