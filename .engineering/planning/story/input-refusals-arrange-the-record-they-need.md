---
format: aep.planning-md/3
id: story:input-refusals-arrange-the-record-they-need
kind: story
status: active
title: A refusal scenario on a command that needs an existing record does not arrange that record, so it is skipped
refs:
- provider: github
  reference: beyond10x/ess#209
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:49Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:49Z", actor: "human:timo", revision: 3}
---
# Story: A refusal scenario on a command that needs an existing record does not arrange that record, so it is skipped

## Why

beyond10x/ess#209. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #209 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

The defect reproduces on this tree. An input-guarded refusal on a command that addresses an existing record is sent with no record set up for it. Checked by reading the code only: nothing was built or run.

**(1) Does it reproduce, and is it fixed?**
- cited: `crates/specify/ess-compiler/src/ir.rs:815-824`. A refusal has `subject: None` ("neither does any refusal").
- cited: `synthesize.rs:1954-1955`, then `arranged` (`:2065-2071`), then `prepare`, then `prepare_subject` (`:2632-2634`). With no subject this returns `Setup::none()`, so no creating command runs.
- cited: `synthesize.rs:3304-3334`. `supply` then sends the id's plain witness value, not an arranged instance.
- cited: the only route that arranges a row for a subject-less refusal is `subject_fact::routes`/`common` (`subject_fact.rs:97-121`). It is gated on `uses(command)` (`:45-52`), which is true only when the command has `when_subject*` branches. `Configure` in the issue has none, so it falls through.
- cited: `has_subject_guards` (`synthesize.rs:3344-3351`) counts only `SubjectState` and `StateChange`, so `wrong_state:` does not turn on state arrangement for the input refusals.
- cited: the existing fix pattern, `existence::refusals_on_a_stored_row` (`existence.rs:524-579`, called at `:226`), stores a row and re-sends the refusal. It only runs for create-or-update commands (`unknown_instance:` / `existing_instance:`) and uses the command's own creating branch, not another command's.
- inferred: not already fixed. No test under `crates/verify/ess-conformance/tests/` has a separate creating command plus an input refusal on an update command.
- inferred: the "skipped" status comes from the downstream target or runner answering "no such account". Nothing in `src/` skips a scenario at run time for that reason.
- inferred: the `already-configured` (`wrong_state`) skip may have a different cause. Both paths for `wrong_state` refusals (`is_state_refusal` at `:2158` and `state_refusals` at `:2248`) require `has_subject_guards`. Confirm by synthesizing the reduced shape.

**(2) Where the fix lands**
- inferred: `synthesize.rs`, in `arranged` / `prepare_subject`, or a new pass beside `refusals_on_a_stored_row`. For an input-guarded refusal where `subject_fact::common(command)` names an input-supplied subject (`moves:`/`updates:`/`deletes:` with `instance: <input>`), arrange that entity with `arrange_first` / `arrange` (`:2744`, `:2785`, through `ir.drivers()`, which includes other commands' `creates`) and pass the instance to `supply`.
- inferred: if no arrangement can reach the refusal, withdraw the scenario and file a refusal (`RefusalCause::InstanceRequired`) instead of the plain send, as `existence.rs:573-576` already does.
- inferred: skip `reads_identity` refusals (`subject_fact.rs:129`), for example `id == ""`. Those must stay plain sends (#178).

**(3) Collisions with other issues**
- #198: high. The same `arrange` creator loop (`synthesize.rs:2798-2819`, "ties to the first creator declared"). cited
- #199: medium. `synthesize.rs` route and arrangement, plus `subject_fact.rs`. inferred
- #201: medium. The `wrong_state` refusal families (`is_state_refusal` and `state_refusals`, `synthesize.rs:2158-2305`), plus domain validation. inferred
- #196: low. `witness.rs` (Map witness), which does not touch arrangement. inferred
- #210: none. `mutate.rs` only. inferred
- #195: none. Binding and compiler work. inferred

**(4) Design decisions for the implementor**
- cited: the documented precedence (`docs/design/outcome-shapes.md:267-273` and `input-guard-overlap-precedence.md:33`) says an input refusal is answered before existence. So today's no-record send is a valid witness by design, and a target answering "not found" breaks that rule. The implementor has to choose between:
  - replacing the no-record send with an arranged one (what the issue asks for), or
  - keeping it and adding the arranged half, as `refusals_on_a_stored_row` does for create-or-update.
- inferred: which creating command to use. This should follow whatever #198 settles, not a separate choice.
- inferred: the synthesis refusal code and message when the arrangement cannot reach the refusal.

Confidence: high on the code path (1) and the fix site (2). Medium on why the third refusal skips and on what the runner reports.

Paths:
- crates/verify/ess-conformance/src/synthesize.rs
- crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- crates/verify/ess-conformance/src/synthesize/existence.rs
- crates/specify/ess-compiler/src/ir.rs (read only, no change expected; inferred)
- crates/verify/ess-conformance/tests/ (new regression test; inferred)
- docs/design/outcome-shapes.md (precedence note to extend; inferred)

Verdict: open (the implementor must first choose between replacing the no-record send and adding an arranged half)
