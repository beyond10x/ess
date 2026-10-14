---
format: aep.planning-md/3
id: story:enforced-reference-needs-no-missing-target-branch
kind: story
status: draft
title: A reference whose target cannot be missing needs no missing-target branch
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#509
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

Validation does not demand an `exists: false` branch for a related row the model makes
unreachable: a reference whose target is created before the referencing row and never deleted.

## Evidence

GitHub https://github.com/beyond10x/ess/issues/509 (ess 0.56.0): with the `no-server` branch of the
https://github.com/beyond10x/ess/issues/505 model deleted, validation reports ESS-COMMAND-005
`non_exhaustive_branches`, though the creating command guards the target's existence and no
command deletes it.

## Acceptance

- The specification can state that a relation's integrity is enforced (or derive it from the
  creating guard and the absence of a deleting command); a command reading it then needs no
  missing-target branch.
- A command or binding that can delete the target makes the declaration refused, naming that
  command.
- Conformance checks the enforced integrity: a target that deletes the target row fails.

## Second adopter (2026-10-13)

A second adopter, reported 2026-10-13 on https://github.com/beyond10x/ess/issues/509: a downstream
incident-ledger specification (ess 0.59.0, `ess/24`). Two owned records read their owner through a
stored reference; the owner is created first and never deleted.

- Without the branch: `ESS-COMMAND-005` "`review.ApproveReview` reads the related row's fields, and a
  missing row makes every predicate over it unknown: no branch answers when the row does not exist".
- With it, synthesis proves it unreachable, `ESS-SYNTH-003`: "… unreachable: every act writing
  `incident_id` (`review.OpenReview/opened`) refuses an identity no row carries before storing it, so
  no run holds one" (`crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs:195`).

Synthesis already proves the branch unreachable from the writers' guards; validation could accept
the same proof instead of a new authored key. Fits none of the four generic language features
(`docs/design/generic-language-features.md`, being drafted); closest is
`epic:generic-row-queries` (references).
