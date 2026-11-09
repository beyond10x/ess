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
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
revision: 12
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
  target.
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

- Done (0.91.0): `a_second_variant_row_is_read_under_its_own_identity`. The neutral repro is a
  fixture, each variant's view expectation names the identity of the event that created that row,
  and the reference target passes.
- Open: an observed-identity reference that is ambiguous between two events is resolved to the
  latest or refused by name; it never silently binds to the first. A test runs the appended suite
  above through the Rust, Go and TypeScript runners. Each either passes (latest) or reports
  `ESS-CF-SUITE` naming the `observed` reference and both occurrences. None reports `ESS-CF-VIEW`
  against the first row.

## Scope

- cited: crates/verify/ess-conformance/src/runner.rs
- cited: crates/verify/ess-conformance/src/go/runtime.go
- cited: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/src/go/mod.rs
- inferred: CHANGELOG.md

confidence: medium

Changing what `observed` means for existing suites needs a new suite major
(`NEWEST_ADMITTED_SUITE_MAJOR`, `src/go/mod.rs`). A by-name refusal of an ambiguous reference may
fit inside the current major.

## Schedule (D401, 2026-11-02)

Left wave W1 (orchestrator D401). Bullet 2 needs the runners' meaning of `observed` to change, so a
new suite major; the story joins the W2 suite-major unit beside `story:a-view-read-checks-every-rows-shape`
(#680), `story:expect-error-checks-generated-field-presence` and
`story:view-and-error-assertions-carry-their-value-shape`. The SYN1 patch for bullet 1 is parked in
the coordinator scratch and rides with that unit. Bullet 1 has since landed in 0.91.0.
