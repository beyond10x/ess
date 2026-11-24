---
format: aep.planning-md/3
id: story:generated-rust-and-go-behaviour-reads-the-addressed-row-in-a-related-predicate-and-entity-runtime-refuses-it
kind: story
status: draft
title: Generated Rust and Go behaviour reads the addressed row in a related predicate, and Entity Runtime refuses it by name
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected
- depends_on: story:synthesis-witnesses-a-related-predicate-that-reads-the-addressed-row-on-rows-that-separate-the-held-state-from
- depends_on: story:generated-rust-and-go-behaviour-read-a-related-row-and-the-addressed-instance-through-a-member-of-a-struct
revision: 1
---
## Need

Generated Rust and Go behaviour answers an identity-addressed `when_related:` guard
(`story:related-guard-behaviour`, beyond10x/ess#319; `crates/generate/ess-synth/src/rust/behaviour.rs`
`related_row` 1308, `present_related` 1528; `crates/generate/ess-synth/src/go/behaviour.rs`
`related_row` 1957, `present_related` 2157, `related_guard` 2715). After
`g11-related-reads-subject` its predicate may read `subject.<field>` and `subject.state`. A
generator that renders the predicate over the related row alone would read a `subject.` path as
nothing, which is a silent wrong answer, the red flag "a target would silently ignore the new
construct" (`.agents/skills/assessing-external-requests/SKILL.md`).

## Outcome

The generated Rust and Go behaviour evaluates a related predicate's `subject.` paths over the
addressed row it already loaded before the related row (the precedence rule reads the addressed
row at steps 2 and 3), and answers the reproduction of `g11-related-reads-subject` as the
interpreter does, request by request. Where an emitter cannot read the path (for example a
`subject.` path inside an `all:` element of a several-rows guard, which is owed already,
`crates/generate/ess-synth/src/determined.rs:219-225`), the command stays an owed obligation whose
text names the `subject.` read. Entity Runtime keeps refusing the command as
`RelatedGuardUnsupported` (`docs/design/generic-language-features.md:568-580`).

## Acceptance

- Red first: `crates/generate/ess-synth/tests/adversary_related_guard_behaviour_pass1.rs` (existing
  binary, its served-model harness `related_guard_served`) gains the reproduction as a model and a
  request sequence covering the four answers (`held`, `recalled`, `loaded`, `wrong-state`), answered
  by the interpreter and by the generated Rust and Go network entry points; every step agrees. On
  the base (after `g11-related-reads-subject`) the generated targets either refuse to build the
  model or disagree.
- A variant whose `subject.` read the emitters do not support is a named obligation: a case in
  `crates/generate/ess-synth/tests/related_guard_several_rows.rs` asserts the obligation text
  names the `subject.` path.
- `crates/generate/ess-entity-runtime`: a case in an existing binary asserts the reproduction is
  refused as `RelatedGuardUnsupported`.
- The repository-wide byte pins of `ess-synth` (`go`, `served_codec_names`) and the committed
  example trees are unchanged (no model there reads `subject.` in a related predicate).
- The Go module test and the Rust build of the served model run in one test each, each under 60 s,
  timed in the report; if either crosses 5 s it is named with its time, and if the harness already
  marks such tests for the `slow` lane (`crates/edge/ess-xtask/tests/slow_lane.rs` `MARKED`), the
  new one is marked the same way.
- `website/docs/reference/entity-runtime-lowering.md` unchanged in meaning; the lowered-definitions
  table (`lowered_definitions_table.rs`) unchanged.

## Fit review

Part of the feature of `g11-related-reads-subject` (report item 11); its fit review applies. This
unit is the generated-target half. No new obligation kind; one new reading in existing emitters.

## Decisions

- **Implement or owe?** (a) Implement the read in both emitters, owing only the forms already owed.
  (b) Owe every related predicate that reads `subject.`. **Recommended: (a)**: the emitters hold the
  addressed row at that point, and (b) leaves the generated servers of every adopter using the
  construct unable to answer it. If reading shows the row is not in scope where the predicate is
  rendered, the unit falls back to (b) and says so in its report.

## Scope

- cited: `crates/generate/ess-synth/src/rust/behaviour.rs` (`related_row` 1308, `present_related`
  1528, `related_read` 1842)
- cited: `crates/generate/ess-synth/src/go/behaviour.rs` (`related_row` 1957, `present_related`
  2157, `related_guard` 2715)
- inferred: `crates/generate/ess-synth/src/rust/condition.rs`, `crates/generate/ess-synth/src/go/`
  predicate rendering
- inferred: `crates/generate/ess-synth/src/determined.rs` (obligation text)
- cited: `crates/generate/ess-synth/tests/adversary_related_guard_behaviour_pass1.rs`,
  `crates/generate/ess-synth/tests/related_guard_several_rows.rs` (existing binaries)
- inferred: an existing `crates/generate/ess-entity-runtime/tests/` binary (one case)
- Size: M (about 350 changed lines).
- Overlap: s5 of the wave of 2026-11-24 changes the same behaviour emitters for a member address;
  this unit lands after s5.

## Depends on

- `g11-related-reads-subject`, `g11-subject-read-synthesis` (the conformance suite of the served
  model is the synthesis unit's output).
- `story:generated-rust-and-go-behaviour-read-a-related-row-and-the-addressed-instance-through-a-member-of-a-struct`
  (s5, same emitters).

## Source

A downstream application, 2026-11-24, report item 11 (its G3).
