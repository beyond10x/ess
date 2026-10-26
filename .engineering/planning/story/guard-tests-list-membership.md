---
format: aep.planning-md/3
id: story:guard-tests-list-membership
kind: story
status: implemented
title: A guard tests whether a subject field is an element of a List input
tags:
- adopter-report
- ess-0.69.0
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T17:07:13Z", actor: "human:timo", revision: 5, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T17:07:14Z", actor: "human:timo", revision: 6, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:36Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Outcome

A command guard can test whether a subject field is (or is not) an element of a `List` input:
`when_subject.predicate` holds when `action` is an element of `input.frontier_actions:
List<String>`, and synthesis witnesses both sides with a non-empty list.

## Evidence

Probe on ess 0.56.0, reported by an adopter: as `when_subject` of a command, both
`action not in input.frontier_actions` and `contains(input.frontier_actions, action)` are refused
with ESS-SPEC-012 ("a predicate is either a comparison (`a.b == 0`) or a bare fact path").
The adopter's mutation audit keeps an `is_empty()` mutant green because no scenario sends a
non-empty list against a subject field.

## Acceptance

- A domain whose `when_subject` predicate is `<field> in input.<list>` (and its negation) validates;
  the operand types must agree (`String` field, `List<String>` input), and a mismatch is refused
  with a named code.
- Synthesis produces, for each such guard, one scenario where the list contains the stored value
  and one where it is non-empty and does not, so a target that ignores the list, or reads only its
  first element, fails a scenario.
- The interpreted target executes the predicate.
- `ess verify diff` classifies adding the guard as a breaking change of the command.

## Spec first

The predicate grammar change lands in the specification crates first, with its refusal code and a
schema projection (`cargo xtask schema`) before synthesis or interpretation.

## Fit review (2026-10-23)

The requested `<field> in input.<list>` spelling duplicates a form that ships: `exists: {in: input.<list>, as: x, that: {x: {eq: {fact: <field>}}}}`, negated with `not: {exists: …}`. It already validates under `when_subject` (test `wrong_state_sibling_conjuncts.rs`), a mismatched operand is refused with `type_mismatch`, synthesis builds the list around the stored value (#516), and `ess verify diff` classifies an added `when_subject` refusal as `RefusalAdded`. Findings from reading the code; not yet run.

## Decisions

- 2026-10-23, coordinator: redesigned, option A. No new predicate syntax, no format, schema or ValidationCode change. The unit lands a test that holds the existing `exists` form to every acceptance line (both guard polarities validate; a mismatch is refused; synthesis writes one scenario where the list holds the stored value and one where it is non-empty and does not; the interpreted target executes it; diff calls the addition breaking), fixes only what that test turns red, and makes ESS-SPEC-012 name the `exists` form when a comparison operand is followed by `in` or `not in`. Rejected: B, a second spelling under `ess/31` (parser, `RawSpecFile`, schema, diff, a new code); C, documentation only, which leaves the synthesis pair unproven.

- 2026-10-23, coordinator: acceptance line 4 (diff calls an added guard breaking) leaves this unit. `ess verify diff` documents an added `when_subject:` refusal as undecided (`crates/verify/ess-diff/src/compatibility.rs`, `narrowing`): deciding it needs a stored subject state the earlier revision admitted, which the diff's input-only check does not search. Changing that classification is a separate change to `ess-diff`, recorded as `story:diff-decides-an-added-subject-guard` for 0.70.0. This unit asserts nothing about the diff.
