---
format: aep.planning-md/3
id: story:synthesis-and-validation-refusals-reproduced
kind: story
status: draft
title: Four reported synthesis and validation refusals are reproduced and answered
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/tests/reported_refusals.rs
- confidence: cited
  path: website/docs/guides/verify/mutation-audit.md
- confidence: cited
  path: website/docs/reference/diagnostics.md
revision: 8
---
## Outcome

Each of four synthesis and validation refusals an adopter reported is reproduced on `main` with a
neutral specification, then either fixed (its own story) or answered with the rule it follows.

## Evidence

An adopter hardening OAuth specifications on ess 0.56.0 reported, without reproducers readable
here:

1. An `example:` on an input narrows synthesis to that one value, and ESS-SYNTH-003 then blames a
   related row. Possibly related: f5994b7804 (invariant-bound inputs grounded).
2. Any `external:` sibling triggers ESS-SYNTH-003.
3. ESS-COMMAND-005 refuses mutants that compare an open String field.
4. ESS-COMMAND-012 forbids a command whose outcomes are all `external:`.

## Reproduction attempts on 0.92.0

Each starts from the workbench base specification
(`.engineering/repro/workbench-2026-11-09/examples/00-base/spec`).

1. Not reproduced. `CreateItem` given `no-parent: when_related: {via: input.parent_id, exists:
   false}` and a related refusal, with `example:` on the compared input:
   - `predicate: name == "archived"`, `example: parent-1` on `parent_id`;
   - `predicate: name == input.title`, `example: shelf` on `title`;
   - `predicate: name != input.title`, `example: shelf` on `title`.

   `ess verify conform synthesize` gives 10 scenarios and 0 refusals for each, the same as without
   `example:`, and the interpreted run passes them.
2. Not reproduced. `FinishItem` (moves, `unknown_instance`, `wrong_state`) given
   `archive-offline: {external: the archive answers that it is offline, error: NoSuchItem}`: 9
   scenarios, 0 refusals. A command with a `when:` refusal over an open String input, a
   `when_subject: {predicate: title == "locked"}` refusal and an `external:` refusal beside an
   `updates:` branch: 13 scenarios, 0 refusals. The one remaining ESS-SYNTH-003 for an `external:`
   branch is deliberate: an `external:` branch after an accepting `when:` branch that claims every
   input (CHANGELOG 0.87.0).
3. Not reproduced. `ess verify conform mutate --target interpreted` on that command (a `when:
   title == "reserved"` refusal over an open String input, and the `when_subject:` one) produces no
   ESS-COMMAND-005: both `guard-negate` mutants are killed. ESS-COMMAND-005 is
   `non_exhaustive_branches` ("every outcome of `X` is conditional"), which a mutant leaving no
   default branch would get.
4. A rule, answered. A command whose only outcomes are `external:` is refused as ESS-COMMAND-012
   (`unreachable_branch`): "every outcome of `X` is decided outside its input, so the command has no
   specified behaviour a test can construct an input for", help "declare what it does when nothing
   goes wrong" (`crates/specify/ess-domain/src/command.rs:167`, `:4323-4339`). Such a command also
   gets ESS-COMMAND-005, because no outcome is unconditional.

Answered by design on `main`, recorded here so the adopter gets one reply:

- ESS-COMMAND-004 depends on whether a sibling compares a String field: a guard the prover
  declines counts as overlapping (`docs/design/cross-record-and-stored-field-guards.md:912`).
- `mutate --class from-drop` needs at least two `from` states and a `wrong_state` sibling
  (`crates/verify/ess-conformance/src/mutate.rs:106-109`).
- ESS-AUTHOR-041 does not see a `wrong_state` step a `when_subject` sibling answers first: a guard
  reading a held state or a stored row claims nothing
  (`crates/verify/ess-conformance/src/authored.rs:1323-1328`).

## Acceptance

- `cargo test -p ess-conformance --locked --test reported_refusals` (new) passes. It holds the six
  shapes of attempts 1 and 2 above over the workbench base specification and asserts, for each, the
  scenario count recorded there (10, 10, 10, 9 and 13, and the 10 of the base `CreateItem` without
  `example:`) and 0 refusals.
- The same target holds attempt 3's command and asserts that `mutate` against the interpreted
  target reports no ESS-COMMAND-005 and both `guard-negate` mutants killed.
- `website/docs/reference/diagnostics.md` states item 4's rule beside ESS-COMMAND-012 and the
  ESS-COMMAND-004 and ESS-AUTHOR-041 rules above, each with its reason: `grep -n` for each code on
  the page finds a line holding that reason.
- `website/docs/guides/verify/mutation-audit.md` states the `from-drop` precondition (two `from`
  states and a `wrong_state` sibling): `grep -n from-drop` on the page finds it.

## Decisions

- Items 1 to 3 are closed as not reproduced, with the shapes tried pinned in the `reported_refusals`
  target, instead of waiting on a reproducer from the adopter: a reply from outside the repository
  cannot be checked here, and a reproducer that arrives later is filed as its own draft story.

## Scope

This story moves only the `CHANGELOG.md` shared surface.

- inferred: crates/verify/ess-conformance/tests/reported_refusals.rs
- cited: website/docs/reference/diagnostics.md
- cited: website/docs/guides/verify/mutation-audit.md

confidence: medium
