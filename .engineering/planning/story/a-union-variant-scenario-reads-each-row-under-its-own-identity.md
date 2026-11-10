---
format: aep.planning-md/3
id: story:a-union-variant-scenario-reads-each-row-under-its-own-identity
kind: story
status: active
title: A union-variant scenario reads each row under its own identity
summary: An observed identity resolves to the first event, not the latest, so the second variant row is read under the first id; reporter repro union-identity.
tags:
- adopter-report
- defect
- ess-0.71.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/union_variant_row_identity.rs
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:56Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:56Z", actor: "human:timo", revision: 8}
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, from a downstream repository specification (which
quarantines 9 scenarios for gaps 1 and 2).

A scenario that covers each union variant in turn creates two rows and reads the second under the
first event's observed identity: `{"kind":"observed"}` resolves to the first event, not the latest.
Reporter repro `union-identity` (1 scenario, fails): the view check expects
`box_id = …0001, kind = Plain`, where the target correctly holds the second row under its own id.

Status on 0.92.0:

- Bullet 1 is fixed (0.91.0, CHANGELOG `### Fixed`, naming this story). Synthesis captures each
  further variant run's row and reads it under that capture.
  `crates/verify/ess-conformance/tests/union_variant_row_identity.rs::a_second_variant_row_is_read_under_its_own_identity`
  pins it, and the neutral model of that test synthesizes 1 scenario that passes the interpreted
  target. This story's acceptance does not repeat it.
- Bullet 2 is open. The runners still resolve `{kind: observed}` to the event's first occurrence
  (`crates/verify/ess-conformance/src/runner.rs:3426-3432`, `.find` over `seen`; the same in
  `src/go/runtime.go` and `src/ts/runtime.ts`), and the 0.91.0 entry says "the runners' reading of
  `observed` is unchanged". Reproduction: take the suite synthesized for that model and append a
  view check reading the second (`Plain`) row as
  `{kind: observed, event: store.box.BoxMade, field: box_id}`:

  ```text
  $ ess verify conform run --target interpreted --path model.yaml --suite ambiguous-suite.json --report-format 2
    failed store.box.MakeBox/outcome/made
  ESS-CF-VIEW expected: store.box.Boxes holds a row where box_id = "…0001", kind = {"kind":"Plain"}
              observed: {box_id = "…0001", kind = Fancy}; {box_id = "…0002", kind = Plain}
  ```

  The reference silently binds to the first `BoxMade`, so a correct target fails with a view
  mismatch instead of the suite being refused for an ambiguous reference.

## Acceptance

All in `crates/verify/ess-conformance/tests/union_variant_row_identity.rs`, run with
`cargo test -p ess-conformance --locked --test union_variant_row_identity`, on the appended suite
above (the neutral model of that file plus the view check reading
`{kind: observed, event: store.box.BoxMade, field: box_id}` after two `BoxMade` events). Each is
red on 0.92.0.

- `an_ambiguous_observed_reference_is_refused_by_the_rust_runner`: the interpreted target's run
  reports `ESS-CF-SUITE` for `store.box.MakeBox/outcome/made`, the message names
  `{kind: observed, event: store.box.BoxMade, field: box_id}` and says the event was observed 2
  times, and the report holds no `ESS-CF-VIEW` finding.
- `an_ambiguous_observed_reference_is_refused_by_the_go_runner`: the same suite through the Go
  runner fails with a line holding `ESS-CF-SUITE`, the same reference and the count 2, and no
  `ESS-CF-VIEW` line.
- `an_ambiguous_observed_reference_is_refused_by_the_typescript_runner`: the same through the
  TypeScript runner, with the same three facts.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` and the
  `cargo test -p ess-conformance --locked` run pass with no row of
  `synthesis-suite-bytes-table.tsv` moved: no synthesized suite holds an `observed` reference to
  an event observed twice before it.

## Decisions

- An ambiguous `observed` reference is refused by name as `ESS-CF-SUITE`, not resolved to the
  latest occurrence: the report asks for the refusal ("instead of the suite being refused"), and
  rebinding to the latest would change, without a word, which row an existing suite reads, which
  needs a new suite major, while a by-name refusal changes no reference that has one meaning.
- `src/go/mod.rs` leaves the scope: the refusal admits no new suite major
  (`NEWEST_ADMITTED_SUITE_MAJOR` is unchanged).

## Scope

- cited: crates/verify/ess-conformance/src/runner.rs
- cited: crates/verify/ess-conformance/src/go/runtime.go
- cited: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/tests/union_variant_row_identity.rs

confidence: medium

`synthesize.rs` changes only if a synthesized suite would hold an ambiguous `observed` reference;
it then captures the row instead. Shared surfaces: proposes a `CHANGELOG.md` sentence; re-pins no
suite-bytes row.

## Schedule (D401, 2026-11-02)

Left wave W1 (orchestrator D401). Bullet 2 needs the runners' meaning of `observed` to change, so a
new suite major; the story joins the W2 suite-major unit beside `story:a-view-read-checks-every-rows-shape`
(#680), `story:expect-error-checks-generated-field-presence` and
`story:view-and-error-assertions-carry-their-value-shape`. The SYN1 patch for bullet 1 is parked in
the coordinator scratch and rides with that unit. Bullet 1 has since landed in 0.91.0. Revised at
plan-critic round 1: the open bullet is a by-name refusal, which needs no new suite major (see
Decisions).
