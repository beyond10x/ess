---
format: aep.planning-md/3
id: story:conformance-authors-sends-and-observes-a-struct-written-in-its-short-form-and-stops-withdrawing-the-scenarios
kind: story
status: draft
title: Conformance authors, sends and observes a struct written in its short form and stops withdrawing the scenarios that observe one
summary: Authoring refuses a struct written in its short form and synthesis withdraws every scenario that observes one (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:the-generated-rust-go-and-web-targets-read-a-struct-written-in-its-short-form-instead-of-refusing-the-model
- informed_by: story:a-reader-side-conformance-admits-reader-widening
revision: 2
---
## Need

A struct with `shorthand: <field>` (`ess/27`) can be written as the field's scalar alone, but the conformance suite cannot say so.
Two seams refuse it:

- An authored scenario value written in the short form is refused at authoring: `ess verify conform author` answers
  `ESS-AUTHOR-015 ... the input of <command>: <member>: expected <Struct> as a mapping, found a number` (reproduced on installed
  0.98.0, brand-free model `demo.lim.Bound`, `shorthand: max`, input `{bound: 5}`). So a specification cannot pin a request written
  in the form its schema publishes.
- Synthesis withdraws every scenario that observes a struct reaching a shorthand and refuses it naming the type, because "every
  runner's observer reads a struct as its object only" (`crates/verify/ess-conformance/src/synthesize/shorthand.rs:1-20`,
  `withdraw_observed`, called at `synthesize.rs:3697`; hint at `synthesize.rs:1494-1496`). A model that declares one shorthand loses
  the scenarios of every command, event, error or view that carries the type.

The generated servers write the long form always, so this is about what a target answers: "a target may answer a value of such a struct in
its short form" (same file). A target that does, honouring the specification, fails today's observers, and a target that does not is not
exercised on the short form either.

## Findings covered

- Gap 1 of the thirteenth report, the suite half (the reporter removed the shorthands rather than lose scenarios).

## Fit review

1. Need: the suite exercises the short form a specification admits: sends it, and accepts an answer in either form. Domain fact.
2. Class: gap (authoring refuses a value the format admits), with a deliberate withdrawal recorded as owed.
3. Expressible today: no; the long form is the only spelling a scenario can write, and the withdrawal removes scenarios.
4. Fit: the authored short form is normalized to the long form at authoring, so the suite bytes carry one spelling (the same rule
   `ess/27` states for writers). Observers (the interpreted target, the Rust runner, the Go runtime `runtime.go`, the TypeScript runtime
   `runtime.ts`) accept either form for a struct with a shorthand. This is reader widening, as `story:a-reader-side-conformance-admits-reader-widening`.
5. Second adopter: a webhook payload whose member is a bare string or an object.
6. Cost: no source format; no suite format if the suite writes the long form only (the scenarios it adds are ordinary ones). A
   runner change in three languages. The withdrawal's reference text (`synthesize.rs:1494`) goes.
7. Alternatives: keep the withdrawal and document it (today); accept the short form only in authoring and keep observers strict (a
   target answering short still fails).

## Decisions

Accept as proposed. FM-D2, how the conformance suite handles the short form. Decided: A, 2026-11-24 16:36 CET (the coordinator, under the owner's delegation), taken on the default: authoring accepts the short form and writes the long form into the suite; every observer (interpreted, Rust runner, Go and TypeScript runtimes) accepts either form for a struct with a shorthand; the withdrawal is removed. B (authoring only, observers stay strict) and C (change nothing, scenarios stay withdrawn) not taken.

## Acceptance

- authored_short_form_is_accepted: `ess verify conform author` accepts `input: {bound: 5}` for a shorthand struct and writes the
  scenario's input as `{bound: {max: 5}}`; the refusal `ESS-AUTHOR-015` still names a scalar of the wrong type for the shorthand field.
- every_observer_reads_either_form: the interpreted target, the Rust runner, the Go runtime and the TypeScript runtime pass a
  scenario whose answer is `5` or `{max: 5}`, and fail one whose answer is `6`.
- no_scenario_is_withdrawn_for_a_shorthand: the synthesized suite of the reproduction keeps every scenario that observes the
  struct, and `withdraw_observed` and its reference text are removed.
- a_mutant_that_drops_the_field_is_killed: `ess verify conform mutate` on the reproduction kills a mutant answering the short form
  with the wrong field.
- the_test_step_stays_in_budget: each new test under 60 s, the step under 120 s.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/shorthand.rs`, `synthesize.rs` (3697, 1494-1496)
- inferred: `crates/verify/ess-conformance/src/authored.rs` (the `ESS-AUTHOR-015` value check), `interpret/values.rs`
- inferred: `crates/verify/ess-conformance/src/go/runtime.go`, `ts/runtime.ts`, the Rust runner's observer
- inferred: `crates/verify/ess-conformance/tests/undeclared_fields/shorthand.rs`, `ranked_and_shorthand_work.rs`
- Depends on: `story:the-generated-rust-go-and-web-targets-read-a-struct-written-in-its-short-form-instead-of-refusing-the-model`
  for the generated side of the same scenarios (borders it; does not need it to land first).
