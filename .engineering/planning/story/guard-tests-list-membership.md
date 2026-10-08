---
format: aep.planning-md/3
id: story:guard-tests-list-membership
kind: story
status: draft
title: A guard tests whether a subject field is an element of a List input
tags:
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
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
