---
format: aep.planning-md/3
id: story:feature-request-429
kind: story
status: active
title: No effect for an outcome that changes a record's identity (rename)
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#429
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/outcome_shapes.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/value_expression.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/identity_changing_updates.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/identity_changing_updates.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/change.rs
- confidence: inferred
  path: docs/design/identity-changing-updates.md
- confidence: cited
  path: models/toolchain/domains/specify.yaml
- confidence: cited
  path: website/docs/guides/specify/commands-and-outcomes.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:20Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:20Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome
Resolve beyond10x/ess#429: No effect for an outcome that changes a record's identity (rename).

## Origin
beyond10x/ess#429, filed 2026-10-05; found in a downstream storage specification on ess 0.52.0 (`ess/20`), reproduced minimally in `<fit-review scratch>/probe-429/`.

## Fit review
1. Need: a command addresses a stored record by its identity and leaves it answering to a new identity, every other field carried over; afterwards the old identity names nothing. Minimal reproduction: `demo.vault.Secret` identified by `name`, `Rename {name, new_name}`. Requester's syntax (theirs): `renames: <Entity>` with `instance: address` and `to: {name: input.new_name}`, or one outcome that both deletes and creates. Today the composite is refused: one outcome does one thing to one entity (`crates/specify/ess-domain/src/command.rs:6181-6197`, ESS-COMMAND-004).
2. Class: defect plus gap. Defect: `updates:` with the identity in `sets:` is admitted, and the suite then requires the opposite of a rename. On installed ess 0.52.0, `ess specify validate` prints `demo v1 — 1 file(s), valid` for `updates: demo.vault.Secret, instance: name, sets: {name: input.new_name}`, and the synthesized `demo.vault.Rename/outcome/renamed` ends with `expect_view contains {name: <captured old instance>, value: "value"}` (probe `a-sets-identity.suite.yaml`). A target that renames fails that scenario; a target that ignores the write passes it. In the 0.53.0 tree the only identity-write refusal is `identity_set`, called for set effects only (`command/set_effects.rs:503`, `:556`, `:1136-1158`). Gap: no outcome can say the old identity is gone and the new one holds the row. The reporter wrote "`updates:` with `sets:` cannot write an identity field". That does not reproduce for a single-field identity. For their struct identity it is unclear; ask them.
3. Existing idiom: none expresses it. `deletes:` (`website/docs/guides/specify/commands-and-outcomes.md:163-181`) and `creates:` cannot share an outcome. A surrogate identity with the address as a plain field changes the command surface the implementation exposes, so a retrofit cannot claim it. The collision answer is already expressible: `when_related: {entity, where, exists: true}` with an `error:` (`website/docs/reference/predicates.md:28`, `ess/22`).
4. Fit: give the construct that already parses a meaning, and add no keyword. From `ess/23`, an `updates:` with `instance:` whose `sets:` writes the entity's identity re-keys the record: the row is read under `instance:` and comes to rest under the written identity, and fields `sets:` does not name are carried over. Composes with: guards (`when:`, `when_subject*` read the row before the outcome, as for any update); the collision refusal through the existing `when_related` exists form, whose rows are the store before the branch (predicates.md:28), so a rename to the record's own identity is the collision; `payload:` reads `input.*` as today; and `wrong_state`/unknown-instance on the old identity unchanged. Siblings: `affects:` and `instances:` keep refusing an identity write (set_effects.rs:1134-1158: rows that all take one identity are one row). `compensates: true` refusals and the `unknown_instance:` create-or-update pair (commands-and-outcomes.md:93-130) refuse an identity-writing `updates:` by name, because their pairing rules read `instance:` as the identity that persists. Struct identities are written whole (`sets: {address: input.new_address}`). A dotted write into one member of an identity is not in this cut. Relations: an identity-writing update on an entity that a declared `owns`/`references` relation carries (`docs/design/ess-entity-relations-design-v0.1.md` §2) is refused (`unsupported_construct`), since a re-key would leave every carrier dangling. Cascading the new key to carriers is out of scope. Targets: interpreter re-keys; generated Rust behaviour removes and inserts through its storage port; Go, Web, Clap refuse by name (`MissingRepresentation`); Entity Runtime refuses (`IdentityChangeUnsupported`, beside `SetEffectUnsupported` in `crates/generate/ess-entity-runtime/src/subset.rs`); `ess verify diff` reports the write as an outcome effect change; generated docs say "re-keys".
5. Second adopter: a file catalogue keyed by path whose `Move {path, new_path}` keeps size and owner. Or a username-keyed profile with `ChangeUsername`. Same domain fact, and neither is one adopter's convention.
6. Cost: one source format, `ess/23` (`crates/specify/ess-domain/src/system.rs:53-55` lists 1–22). This story introduces it; #450, #452, #458 and #459 depend on it and extend it. Below `ess/23` the identity write becomes a refusal naming `ess/23`, where today it validates. That is a breaking change for any document that wrote it. Accept it: those documents carry a suite that contradicts them (Q2). No new suite step. The scenario reuses `deletes:`'s absence check (`ess-conformance/22`, commands-and-outcomes.md:176-179; `deletion_witness`, `crates/verify/ess-conformance/src/synthesize.rs:10736`) and `expect_view contains`, so no new conformance major is expected. Confirm when the design note is written. One new Entity Runtime refusal variant and one diff line.
7. Alternatives: (a) change nothing and refuse the identity write below and above: honest, but leaves the gap and still changes today's validate answer. (b) Requester's `renames:` with `to:`: a sixth effect verb plus a key that restates `sets:`; it duplicates `updates:`. (c) Allow `deletes:`+`creates:` in one outcome: breaks the one-thing rule every reader of an outcome relies on, and a carried field has no source. (d) Chosen: give `updates:`+identity-in-`sets:` the rename meaning. It adds no key, fixes the defect, and the requester's witness (old gone, new present with carried fields) is kept unchanged.

## Decisions
accept, redesigned — from `ess/23`, an `updates:` whose `sets:` writes the identity re-keys the record. A collision is answered by a declared `when_related` exists refusal over the written identity; validate refuses an identity-writing update without one (`missing_declaration`). The requester's `renames:`/`to:` and the composite delete+create are not adopted (Q7). Refused by name: on `ess/≤22`, beside `compensates:`, in a create-or-update pair, on a relation-carried entity, and in `affects:`/`instances:` as today. Format: this story introduces `ess/23` (coordinator decision 2026-10-05): `SUPPORTED_FORMATS`, the `ess/23` row of `website/docs/reference/spec-versions.md`, the toolchain model's `SpecificationFormat` variant and the `FORMAT_RELEASES` row. #450, #452, #458 and #459 depend on it; none of them edits `system.rs` or `spec-versions.md`. This story writes the `ess/23` row with its own sentence; each of the four reports one sentence for the row, and the coordinator merges them into it at integration. No conformance major is expected. Design note `docs/design/identity-changing-updates.md` before code.

## Acceptance
- rename_reads_old_identity_absent_and_new_present: after the command, no immediate view holds the old identity, and the row under the new identity carries every field `sets:` did not write.
- rename_old_identity_answers_unknown_instance: resending the command for the old identity takes the command's unknown-instance answer.
- rename_collision_takes_declared_refusal: an arranged second record under the target identity selects the `when_related` refusal, with no event, and both rows read unchanged.
- rename_to_own_identity_is_the_collision: new identity equal to the old one selects the refusal.
- identity_write_without_collision_answer_is_refused: validate refuses with `missing_declaration` at the outcome.
- identity_write_below_ess23_is_refused_naming_ess23: an `ess/22` document is refused at `sets.<identity>`; no cascade diagnostics.
- identity_write_refused_on_relation_carried_entity, identity_write_refused_beside_compensates, identity_write_refused_in_create_or_update_pair: each refused by name.
- rename_targets_refuse_by_name: Go, Web, Clap `MissingRepresentation`; Entity Runtime `IdentityChangeUnsupported`; generated Rust executes the scenario green and an ignore-the-write mutant fails it.
- rename_diff_reports_identity_write: `ess verify diff` names the added identity write.
- ess_23_is_admitted_and_recorded: `SUPPORTED_FORMATS` lists 23; `the_model_lists_every_admitted_specification_format` (`crates/edge/ess-xtask/tests/model_enums.rs:216-232`) passes with `ess/23` in `models/toolchain/domains/specify.yaml`; the docs lane that reads `FORMAT_RELEASES` (`crates/edge/ess-xtask/src/docs.rs:64-89`) passes with an `("ess", 23, …)` row; `spec-versions.md` has an `ess/23` row.

## Scope
- crates/specify/ess-domain/src/command.rs  cited — `subject_of` (6161-6271), the one-thing rule (6181-6197)
- crates/specify/ess-domain/src/command/value_expression.rs  cited — identity in `sets:` read as a source (257-262)
- crates/specify/ess-domain/src/command/set_effects.rs  cited — `identity_set` stays the set-effect refusal (1134-1158)
- crates/specify/ess-domain/src/command/outcome_shapes.rs  cited — create-or-update pair reads `instance:` as the persisting identity
- crates/specify/ess-domain/src/system.rs  cited — `SUPPORTED_FORMATS` gains 23 (53-55); `FormatVersion::V23` beside `V22` (122), which #450, #452, #458 and #459 gate on
- crates/verify/ess-conformance/src/synthesize.rs  cited — `deletion_witness` reused for the old identity (10736)
- crates/verify/ess-conformance/src/interpret/execute.rs  inferred — interpreter re-key
- crates/generate/ess-synth/src/rust/mod.rs  inferred — storage-port remove/insert
- crates/generate/ess-entity-runtime/src/subset.rs  cited — refusal variant beside `SetEffectUnsupported`
- crates/verify/ess-diff/src/change.rs  inferred — identity-write change line
- crates/specify/ess-domain/tests/identity_changing_updates.rs  inferred — new domain cases
- crates/verify/ess-conformance/tests/identity_changing_updates.rs  inferred — new witness cases
- docs/design/identity-changing-updates.md  inferred — design note
- website/docs/guides/specify/commands-and-outcomes.md  cited — new section beside deletes (163-181)
- website/docs/reference/spec-versions.md  cited — the `ess/23` row, created here
- models/toolchain/domains/specify.yaml  cited — `SpecificationFormat` variants gain `"ess/23"` (:42)
- crates/edge/ess-xtask/src/docs.rs  cited — `FORMAT_RELEASES` gains the `ess/23` row (beside `("ess", 22, …)` at :159)
