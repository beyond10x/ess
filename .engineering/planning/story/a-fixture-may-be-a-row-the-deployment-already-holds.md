---
format: aep.planning-md/3
id: story:a-fixture-may-be-a-row-the-deployment-already-holds
kind: story
status: active
title: A fixture may be a row of an entity the specification does not create
summary: 'No fixture provider for an external record a command binds to: ESS-CF-TARGET on the interpreted target, ESS-SYNTH-004 in synthesis (downstream deployment, 0.77.0).'
tags:
- adopter-report
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/fixture_inputs.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/fixtures.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/fixtures.go
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/setup.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-diff/src
- confidence: cited
  path: docs/design/pre-execution-fixture-values.md
- confidence: inferred
  path: schemas/generated/ess.schema.json
revision: 21
transitions:
- {from: "draft", to: "proposed", at: "2026-10-31T11:37:41Z", actor: "human:timo", revision: 20}
- {from: "proposed", to: "active", at: "2026-10-31T11:37:42Z", actor: "human:timo", revision: 21}
---
## Need

Reported by a downstream deployment's specification session (2026-10-30, ESS 0.76.0, reproduced
on 0.77.0). A command binds a new record to an external record the deployment already holds and
that no command of the specification creates. The accepted branch needs such a record to exist,
and nothing can provide one:

- an authored scenario that takes the record's identity as a target-provided fixture
  (`fixtures: {existing-app: …AppId}`, `app_id: {$fixture: existing-app}`) is reported unsupported
  on the interpreted target: `ESS-CF-TARGET: cannot expose fixture values: no pre-execution
  fixture provider`;
- synthesis refuses the accepted branch: `ESS-SYNTH-004 … it needs an instance of
  `…App` to change without moving, and no outcome creates one`;
- a synthesis seed holding the related row is selected and not applied (`1 selected, 0 applied`),
  because the row is not the command's subject;
- `fixture_inputs: {app_id: existing-app}` keeps the `ESS-SYNTH-004` refusal and turns the
  remaining `unknown-app` scenario unsupported on the interpreted target;
- an authored `arrange:` row passes on the interpreted target, but has the runner write the row
  under an identity the suite chooses, which a deployment that only reads such records cannot
  accept.

Brand-free reproduction: entities `App` (no command creates it) and `Link` (references `App` via
`app_id`); `CreateLink(app_id)` with outcomes `unknown-app` (`when_related: {via: input.app_id,
exists: false}`) and `linked` (creates `Link`).

## Triage on main (f692aab, 0.77.0)

Re-run 2026-10-31 with the 0.77.0 binary: output identical to the report. Not fixed.

## Requester's proposal (theirs)

1. A fixture can be declared as an instance of an entity the specification does not create (here
   an `App` row in state `Active`); the interpreted target provides it by creating the row before
   execution and exposing its identity as the fixture value; an implementation's runner gets the
   value from its own fixture provider, as `fixture_inputs:` values do now.
2. With such a fixture declared, synthesis arranges the accepted branch on the fixture's row instead
   of refusing it with `ESS-SYNTH-004`. Alternatively, synthesis seeds could apply to rows that
   `when_related:` reads.

## Where it lives

Lines on `main` f692aab (0.77.0), read by the scoper on 2026-10-31.

- `crates/verify/ess-conformance/src/target.rs:108-117`: the default `fixture_values` answers
  unsupported ("no pre-execution fixture provider"); `interpret.rs:322-328` does not override it.
  `runner.rs:3158-3188` `resolve_fixtures`; `go/fixtures.go:136` and `ts/runtime.ts:3556` refuse alike.
- `src/fixtures.rs:35-44`: a fixture field is `{name, type}` with `deny_unknown_fields`.
- `src/synthesize.rs:6564-6568`: `arrange` returns `Unreachable::NothingCreates`; the related row is
  arranged through `synthesize/related_guard.rs:3136` (`arranged_at` → `with_row`), ending at
  `subject_fact.rs:5408-5412` (`InstanceRequired{NothingCreates}`, ESS-SYNTH-004).
- `src/synthesize/subject_fact.rs:8904-8908`: seeds apply only to the command's subject entity
  (`docs/design/synthesis-seeds.md:127-130`).
- `crates/specify/ess-domain/src/command/fixture_inputs.rs:88-103` refuses a `fixture_inputs` field an
  outcome predicate reads; for `when_related`, `command.rs:706-716` `predicate()` returns only the
  optional input guard, never `via`.

## Fit review

1. **Need.** A command binds a new row to a row of an entity no command creates, because the
   deployment holds those rows; the accepted branch needs one, and nothing can provide it without the
   runner writing it. Requester's proposal: an authored fixture declared as `{entity, state}`.
2. **Class.** Gap, plus a defect: `fixture_inputs:` on a field read by `when_related.via` validates,
   and synthesis then sends the fixture to the `exists: false` branch (probe output), against
   `docs/design/pre-execution-fixture-values.md:7`.
3. **Already expressible?** Partly: an authored `arrange:` row passes on the interpreted target but
   has the target write a row under a suite-chosen identity, which a read-only deployment refuses, and
   synthesis still refuses the branch.
4. **Fit.** It reuses the `fixture_values` seam and `establish_entity` (`interpret/setup.rs:6`).
   Every position that needs an existing row (subject, owner, related row, view population) must take
   a provided row the same way.
5. **Second adopter.** A ticketing system assigns a ticket to a user an identity directory owns; no
   `CreateUser`, and `AssignTicket(user_id)` refuses an unknown user through `when_related`.
6. **Cost.** `ess/33` (one entity key) and the suite pair `ess-conformance/60`/`61`.
7. **Considered.** Options A-E in `decision-blocker:provided-entity-rows-for-conformance`.

## Decisions

**accept, redesigned**: option B, `architecture-decision-record:a-deployment-may-provide-an-entitys-rows`
(the operator's proxy, 2026-10-31). Format: `ess/33` (shared with the wave) and suite `/60`, `/61`.

- From `ess/33` an entity may declare that the deployment provides its rows (the unit chooses the
  key's spelling and records it here). Below `ess/33` the key is refused as
  `unsupported_format_version` naming `ess/33`.
- Where synthesis needs a row of such an entity in a state and ordinary arrangement ends in
  `NothingCreates`, the scenario begins with a fixture resolution for `{entity, state}` and sends the
  fixture's identity; where the row must not exist, a fresh literal identity. Ordinary arrangement
  keeps precedence.
- The interpreted target answers such a fixture by creating the row in that state; a type-only
  fixture stays unsupported there. Go and TypeScript runtimes admit the entity fixture from `/60`
  and pass it to their provider.
- Defect, at every format: `fixture_inputs:` on an input `when_related: {via: input.<field>}` reads is
  refused as `unsupported_construct`, as a `when:` read is.
- A guard over a provided row's fields cannot be steered and is refused by name; decoy rows beside a
  related row are omitted with a note unless they are provided fixtures too (the unit records which).
- CHANGELOG `[Unreleased]`: Added (provided rows, `ess/33`, suite `/60`, `/61`) and Fixed (the
  `fixture_inputs` defect).

## Acceptance

- `crates/verify/ess-conformance/tests/pre_execution_fixtures.rs`:
  `fixture_inputs_cannot_replace_a_related_row_via` (before: assembles and `unknown-app` is sent the
  fixture; after: refused).
- `crates/verify/ess-conformance/tests/related_guard.rs`:
  `a_related_row_of_a_provided_entity_is_a_fixture_in_the_state_the_branch_needs` (before:
  ESS-SYNTH-004; after: `linked` resolves a `{App, Active}` fixture and sends it; `unknown-app` has no
  prelude and a literal identity).
- `pre_execution_fixtures.rs`: `the_interpreted_target_provides_an_entity_fixture_by_creating_its_row`
  (before: unsupported; after: passed; a type-only fixture stays unsupported);
  `entity_fixtures_require_suite_60_and_old_bytes_hold` (an entity fixture under `/58` refused;
  fixture-free and type-only suites keep their bytes; the key below `ess/33` refused naming `ess/33`).
- The TypeScript and Go runtime tests: an entity fixture contract is admitted from `/60` only.
- End to end, `crates/edge/ess-cli/tests/interpreted_target.rs`:
  `a_provided_related_row_synthesizes_and_passes_on_the_interpreted_target` on a brand-free fixture
  (before: 1 refusal; after: 0 refusals, 2 passed, 0 unsupported on the interpreted target).
