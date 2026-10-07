---
format: aep.planning-md/3
id: story:an-idle-output-record-carries-no-machine-path
kind: story
status: active
title: An idle .ess-output record carries no absolute path, device or inode
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#484
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/output_ownership/mod.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/output_ownership/state.rs
- confidence: cited
  path: docs/design/review-output-ownership.md
- confidence: cited
  path: models/output-ownership/domains/ownership.yaml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T03:42:51Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-07T03:42:51Z", actor: "human:timo", revision: 3}
---
## Outcome

The `.ess-output/state.json` a repository commits holds nothing about the machine that wrote it:
an Idle checkpoint has no absolute root path and no directory identity
(https://github.com/beyond10x/ess/issues/484).

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`, measured on ess 0.55.0.

1. **Need.** `ess generate` writes the output directory's absolute path (`payload.root`, hex
   components) and its device and inode (`payload.directory`) into `state.json`, format
   `ess-output-state/2`; adopters commit that file, so public repositories publish a home-directory
   path. Reproducer in the issue (neutral path `/srv/work/project`).
2. **Class.** Defect: the binding protects an in-flight transaction only
   (`docs/design/review-output-ownership.md`, #306); an Idle binding is admitted when it does not
   match and decides nothing, yet it is persisted.
3. **Already expressible?** No option suppresses it.
4. **Fit.** `models/output-ownership/domains/ownership.yaml` puts `native_root` on the Anchor. The
   binding belongs to the Transaction, which is what recovery needs it for. Moving it there makes
   the settled anchor carry none, and the state format follows the model.
5. **Second adopter.** Any repository that commits generated output.
6. **Cost.** A state format version, `ess-output-state/3`; the reader keeps `/2`; one migration diff
   in each adopter's committed record.
7. **Considered.** (a) change nothing; (b) a relative path, which for an output-relative binding is
   always `.` and is the same as dropping it; (c) document that the record is never committed and
   stop `--check` requiring it, which weakens what `--check` proves and leaves records already
   committed; (d) move the binding to the transaction. (d) is chosen.

## Decisions

- **accept, redesigned**: the binding moves from the anchor to the transaction (option d).

## Acceptance

- Spec first: `models/output-ownership` moves `native_root` from `Anchor` to `Transaction`, and
  `ess specify validate --path models/output-ownership` passes with the newest `ess`.
- A write-mode `ess generate` writes `ess-output-state/3`. An Idle checkpoint has no `root` and no
  `directory`. A checkpoint with a transaction keeps both, and a mismatched in-flight binding still
  refuses as today.
- The reader admits `/2` and `/3`. A write-mode run that finds a `/2` Idle record rewrites it as
  `/3` even when no owned file changed.
- `--check` admits a `/2` Idle record without reporting drift and prints one warning naming the
  machine-specific fields and the regeneration that removes them.
- An old-reader compatibility test exists for the format change (`AGENTS.md`, Determinism and
  formats), and the design page states the new rule.
- A test decodes a written `/3` Idle record and finds no absolute path component, no device and no
  inode.

## Decision on edited owned files (2026-10-07)

Option B of `decision-blocker:idle-output-record-without-binding`: without a binding, the #306 copy
check runs in every folder. An owned file with other bytes refuses before mutation, listing the
files and the re-enroll route, including in the folder that generated it; a missing owned file is
still recreated. The `--check` advice and the CHANGELOG "Changed" entry say so.
