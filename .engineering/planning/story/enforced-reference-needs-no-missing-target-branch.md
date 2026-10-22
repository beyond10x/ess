---
format: aep.planning-md/3
id: story:enforced-reference-needs-no-missing-target-branch
kind: story
status: implemented
title: A reference whose target cannot be missing needs no missing-target branch
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#509
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T06:52:55Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-21T06:52:56Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-22T02:41:45Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
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

## Decisions

- **2026-10-21: the integrity is derived, not authored** (decided by the implementing session on
  the coordinator's delegation, batch 0.67.0). From `ess/30` validation drops the missing-row
  refusal where the commands prove the row present: every branch creating the subject sets the
  field, every write of the field writes nothing, its own value or an input its command refuses
  first with an unguarded `exists: false` over the same entity, and no branch deletes a row of the
  referenced entity. A defeating branch is named in the refusal's hint; below `ess/30` the hint
  names `ess/30`. Reasons: both adopters asked for the proof synthesis already makes rather than a
  key; no source key, schema or IR change, so a later key can extend the rule while a published
  key could not be withdrawn. Binding record: `docs/design/generic-language-features.md` § G2,
  "Enforced references (`ess/30`, decided 2026-10-21)".
- Acceptance 2 is met by the derived form: a deleting command or binding (through the command it
  runs) keeps the branch required, naming the deleting branch, since there is no declaration to
  refuse.

## Remaining

First slice landed on `unit/ess-067-g2` (batch 0.67.0): validation (acceptance 1 and 2),
synthesis and the interpreted model over the fixture
`crates/verify/ess-conformance/tests/fixtures/enforced-reference.yaml`, generated Rust and Go
keeping the command an obligation by name, docs and format history.

Acceptance 3 is met (`unit/ess-0680-g2owed`). Under `ess/30`, synthesis reads the referenced row
again after every accepting branch that writes a row of the referencing or the referenced entity,
and requires the row present (`crates/verify/ess-conformance/src/synthesize/reference_integrity.rs`).
`a_target_that_deletes_the_referenced_row_fails` in
`crates/verify/ess-conformance/tests/enforced_reference.rs` is no longer ignored, and it passes.
The reads are filed under the existing `<entity>/invariant/after/<command>/<outcome>` id, so no
suite format was needed. Specs below `ess/30`, and specs that waive no branch, keep their suite
bytes.

Owed:

- Generated Rust and Go: bind the related read without an `exists: false` branch and lift the
  obligation.
- Synthesis's own unreachability proof (`related_guard/stored.rs`, `unreachable`) reads writers
  only; beside a deleting command it still names a present `exists: false` branch unreachable.
  It should read the same proof as validation.
