---
format: aep.planning-md/3
id: story:authored-exact-value-on-generated-field-is-refused
kind: story
status: implemented
title: An authored exact value on a generated field is refused
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-19T21:15:03Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-19T21:15:03Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-19T21:15:04Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

`ess verify conform author` refuses an authored event or stored-field expectation that states an
exact value for a field the outcome marks `{generated: true}` (a payload field, or a field set by
`sets:`), naming the field and pointing at a view or a relation instead.

Closes https://github.com/beyond10x/ess/issues/544.

## Fit review

1. **Need.** A generated field is minted by the implementation; an authored exact value passes only
   when the implementation happens to mint it, and fails a conforming one. One adopter removed 121
   such values by hand after a field became generated; nothing pointed at them.
2. **Class: defect** in `author` validation: the model already says the value is not the scenario's
   to choose.
3. **Already expressible?** No refusal exists; `author` accepts the step (issue reproduction).
4. **Fit.** A refusal in the existing ESS-AUTHOR family, the same shape as the other step checks
   in `crates/verify/ess-conformance/src/author*`. A refusal, not a warning: every such step is
   wrong for some conforming implementation, and `author` has no warning channel.

## Decisions

- Refuse (not warn) with a new `ESS-AUTHOR-*` code; the help names the field, the outcome and says
  to state the value through a view, or drop the field from the expectation (presence and type stay
  checked by the synthesized suite).
- Applies to event payload expectations and stored-row expectations of a field a `sets:` entry
  marks `{generated: true}`. A field the outcome sets from input or a literal is unaffected.
- Reversible: removing the refusal later accepts every document accepted today plus these.

## Acceptance

Each line names the failing test written first.

- `crates/verify/ess-conformance/tests/author_generated_value.rs`
  `an_exact_value_on_a_generated_payload_field_is_refused` (the issue's `connect_seq` example).
- same file, `an_exact_value_on_a_generated_stored_field_is_refused`.
- same file, `a_generated_field_left_out_of_the_expectation_is_accepted`.
- same file, `a_field_set_from_input_keeps_its_exact_value`.
- `crates/edge/ess-cli/tests` (file inferred) `author_refuses_a_pinned_generated_value`: exit code
  and message through `ess verify conform author`.

## Scope

- inferred: `crates/verify/ess-conformance/src/author.rs` (and its step-check modules),
  `crates/specify/ess-domain` `validation_codes!` (new code), `docs/` diagnostics page,
  `task test-xtask` pins (macro-guards digest, review-typed-diagnostics counts).

## Plan

- depends_on: none.
- du estimate: 8G.
- adversary: no (a small refusal).
- shared surfaces: the diagnostics catalogue with every unit that adds a code.
