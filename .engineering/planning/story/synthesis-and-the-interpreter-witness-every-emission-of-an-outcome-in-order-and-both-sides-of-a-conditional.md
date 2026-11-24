---
format: aep.planning-md/3
id: story:synthesis-and-the-interpreter-witness-every-emission-of-an-outcome-in-order-and-both-sides-of-a-conditional
kind: story
status: draft
title: Synthesis and the interpreter witness every emission of an outcome in order, and both sides of a conditional emission
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:at-the-new-format-an-outcome-may-emit-one-event-several-times-in-order-each-with-its-own-payload-and-an
revision: 2
---
## Need

After `ae9a-an-outcome-emits-repeated-and-conditional-occurrences`, a model may state repeated and
conditional emissions. A suite that expects the outcome's events as today would not catch a target
that publishes one occurrence instead of two, publishes them out of order, or ignores a `when:`.

## Outcome

- The interpreter (`crates/verify/ess-conformance/src/interpret/execute.rs` and `execute/`)
  evaluates each occurrence's `when:` against the pre-outcome subject and publishes the occurrences
  in order with their inline payloads.
- Synthesis (`crates/verify/ess-conformance/src/synthesize.rs`) expects every occurrence, in order.
  For each conditional occurrence it arranges two scenarios: one where the predicate holds (the
  event is expected) and one where it fails (that occurrence is expected absent, the others as they
  fall). The arrangements reuse the `when_subject:` witnesses.
- A side that cannot be arranged is refused by name in the synthesis report, never dropped silently.
- Mutation gains `emission-condition-negate` and `occurrence-drop`, and the suite kills both.

If the expected-events shape of a step cannot carry two occurrences of one event in order, the suite
format moves with the source format (format: decided before acceptance).

## Acceptance

1. Red today (with `ae9a` landed): a synthesized suite for the reproduction model of `ae9a`, run on
   the interpreted target, does not fail a target stub that publishes `StatusChanged` once. A case
   in `crates/verify/ess-conformance/tests/` (an existing synthesis target, chosen at
   implementation) asserts that the mutated target is killed. It fails.
2. After: the suite expects two `StatusChanged` in order with their payloads. It has `updated/when/…`
   scenarios for both sides of each of the three conditions (names fixed at implementation). The
   interpreted target passes. The `occurrence-drop` and `emission-condition-negate` mutants are
   killed.
3. A model without occurrences synthesizes the same bytes. The repository byte pins pass unmodified:
   `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
   `adversary_related_values_all_bytes`, `external_beside_held_guard`.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize.rs`, `src/interpret/execute.rs`,
  `src/interpret/execute/`, `src/mutate.rs`, `src/scenario.rs`

## Owned files

The five above, and new cases in existing `ess-conformance` test targets. No new test binary.

## Not yours

Validation and IR (`ae9a`). Generated targets (`ae9c`).

## Tests first

Acceptance 1 on a model of four entities at most. Time the synthesis before and after. A jump is
fixed in this change (the test-budget rule), and any case over 5 s is named.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes. Run
`ess-conformance` one `--test` at a time (its full build is the largest in the workspace).

```console
task fmt-check
cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
cargo test -p ess-conformance --locked --test <the target holding acceptance 1>
cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table
cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read
cargo test -p ess-conformance --locked --test adversary_related_values_all_bytes
cargo test -p ess-conformance --locked --test external_beside_held_guard
cargo clippy -p ess-cli --all-targets --locked -- -D warnings
cargo test -p ess-cli --locked --test tutorial_page
```

## Depends on

`ae9a-an-outcome-emits-repeated-and-conditional-occurrences`.

## Changelog

- Synthesis expects every emission of an outcome in order and witnesses both sides of a conditional
  emission; the interpreter publishes them as the model declares.

## Fit review

As `ae9a`. Question 4, targets: synthesis and the interpreted reference target are the checked
figure. Without this unit the new construct would be a statement no suite checks (policy 1).

## Decisions

**AE-D11** (open, with the owner as AQ4). The suite-format consequence is decided with the source format.

## Size

L, about 550 lines with tests.
