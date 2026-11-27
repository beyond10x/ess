---
format: aep.planning-md/3
id: story:a-system-issued-creating-command-is-witnessed
kind: story
status: active
title: A command a system actor holds that creates its subject is witnessed
summary: ESS-SYNTH-023 on both outcomes of a system-issued creating command (workbench report item 43)
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/await.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/await_outcome.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner/await_outcome.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/system_issued.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/system_issued.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: cited
  path: crates/verify/ess-conformance/tests/system_issued_synthesis.rs
- confidence: cited
  path: docs/design/system-issued-commands.md
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:42:04Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:42:05Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

A creating command held by a `system: true` actor gets `ESS-SYNTH-023` on both outcomes, so the
step the system takes to create a record, and its refusal, are not exercised by any synthesized
scenario. Item 43 of the 2026-11-09 workbench report; example `43`. beyond10x/ess#542 (closed)
introduced system-issued commands and left this case out of stage 1.

Minimal reproduction (`.engineering/repro/workbench-2026-11-09/examples/43/spec`). `StampItem(id,
parent_id)` is held only by `Engine` (`system: true`):

```yaml
outcomes:
  - name: no-such-parent
    when_related: {via: input.parent_id, exists: false}
    error: NoSuchItem
  - name: stamped
    creates: Item
    instance: id
    sets: {title: stamp, parent_id: input.parent_id, n: 0, tags: []}
    emits: [ItemCreated]
```

```text
$ ess verify conform synthesize --path spec --out suite.json
refused: refusal[ESS-SYNTH-023]: outcome gap.demo.StampItem/no-such-parent has no scenario ...
refused: refusal[ESS-SYNTH-023]: outcome gap.demo.StampItem/stamped has no scenario ...
  `gap.demo.StampItem` is issued by the system itself, and its outcome `stamped` creates the record it addresses, so no record exists before the step; ...
8 scenario(s) (0 authored), 2 refusal(s), written to suite.json
```

This is the documented stage-1 limit, not a regression. An await names the existing record it waits
on (`docs/design/system-issued-commands.md` § The instance, lines 128-134). A `creates:` outcome has
no such record before the step (`crates/verify/ess-conformance/src/system_issued.rs:89`,
`identity_input` returns `Unaddressed::Creates`).

## Acceptance

- `a_system_issued_creating_command_is_witnessed`, in
  `crates/verify/ess-conformance/tests/system_issued_synthesis.rs` with the example-43 model inline:
  synthesis writes `StampItem/outcome/stamped` and awaits it as its system actor. The scenario
  captures the created identity from the answer, checks the `ItemCreated` event and the `Items` view
  row, and passes the interpreted target. No `ESS-SYNTH-023` remains for `stamped`.
- `StampItem/outcome/no-such-parent` is decided by a row the input names. It is refused as
  `ESS-SYNTH-024` naming the branch, as any input-decided refusal of a system-issued step is
  (design page line 184), not as `ESS-SYNTH-023`.
- A model with no system-issued creating command keeps its suite bytes (`an_old_suite_keeps_its_bytes`).

Amendment (tenth gap report, 2026-11-24): a command held only by system actors whose outcomes change rows by `instances:` or `affects:` only is awaited instead of refused `ESS-SYNTH-023`: the scenario arranges the rows, takes the cause, and reads the rows removed and kept.

Amendment (twelfth gap report, 2026-11-24): a command a system actor holds whose outcomes name no `instance:` and change no row (an external call, an event only) is awaited the same way, with no instance: the scenario takes the one issuance in its window and reads the outcome, the error and the events; a second issuance in the window, or none, fails it. It is no longer refused `ESS-SYNTH-023` (`NoInstance`) or `ESS-AUTHOR-045`.

## Scope

- cited: docs/design/system-issued-commands.md
- cited: crates/verify/ess-conformance/src/system_issued.rs
- cited: crates/verify/ess-conformance/src/synthesize/system_issued.rs
- cited: crates/verify/ess-conformance/src/scenario.rs (`ScenarioStep::AwaitOutcome`, its `instance`)
- cited: crates/verify/ess-conformance/src/target.rs (`AwaitRequest`, its `instance`)
- cited: crates/verify/ess-conformance/src/runner.rs (the `AwaitOutcome` step dispatch)
- cited: crates/verify/ess-conformance/src/runner/await_outcome.rs (`Awaited`, its `instance`)
- cited: crates/verify/ess-conformance/src/interpret/await_outcome.rs (the awaited instance compared with the identity input)
- cited: crates/verify/ess-conformance/src/go/await.go (`admitAwaitInstance`, "names no instance")
- inferred: crates/verify/ess-conformance/src/go/runtime.go (`AwaitTarget`)
- cited: crates/verify/ess-conformance/src/ts/runtime.ts (`AwaitRequest`, `awaitOutcome`)
- cited: crates/verify/ess-conformance/tests/system_issued_synthesis.rs

confidence: medium

The story re-pins the suite-bytes rows of models with a system-issued creating command and
proposes its `CHANGELOG.md` sentence; the instance-less await is a suite shape of
`ess-conformance/64` (ordinary) and `ess-conformance/65` (coverage).

## Fit review

1. Need: a record the system creates on its own cannot be checked by a synthesized suite. Its
   creation, event and view row go unchecked. The reproduction is example 43 above. The report
   proposes no syntax.
2. Class: gap. ESS documents this as the stage-1 limit (design page lines 128-134 and 377), so it
   does not contradict documented semantics.
3. Already expressible: partly. An authored `ess-scenario/1` act on a system-issued command compiles
   to `await_outcome`, but that act also needs an instance (design page line 395). The `help:` text
   suggests granting the command to a caller as well. That changes the authorization model and is
   not an answer.
4. Fit: no new authored key. It extends the existing `await_outcome` step to wait on an outcome whose
   instance does not exist yet, and capture the minted identity from the answer, as
   `capture_instance` does after `execute_command`. The await request (`target.rs`, `AwaitRequest`),
   the three runners and the interpreter all read the instance today, so each must accept an await
   without one or refuse it by name.
5. Second adopter: an order service whose scheduler (system actor) opens a monthly invoice record
   with `creates: Invoice`.
6. Cost: a suite format change, because `await_outcome` gains an instance-less form. It lands in
   `ess-conformance/64` (ordinary) and `ess-conformance/65` (coverage), with a target-interface
   change for Rust, Go and TypeScript adapters (breaking for an adapter that requires `instance`),
   and a CHANGELOG entry. No `ess/` source format change.
7. Alternatives: (a) change nothing and keep `ESS-SYNTH-023`, which the help already explains;
   (b) drive the creating step as an ordinary caller, which contradicts the system actor's grant
   (beyond10x/ess#542); (c) an instance-less await that captures the created identity, preferred if
   the target interface can carry it.

## Decisions

- defer: the need is real, but it needs a decision on how an await names a record that does not
  exist yet (alternative c, its suite shape in `ess-conformance/64` and `/65` and the target
  interface). Record a `decision-blocker` with `blocks:` to this story before it leaves draft.
- Same gap as `story:a-create-only-the-system-issues-gets-a-synthesized-scenario` (adopter report of
  2026-10-26, no reproduction). Keep this story, which carries the reproduction, and fold the other
  into it.

- Another downstream adopter (report against 0.90.0, 2026-11-11) needs the same thing for an
  authored scenario: a step that sends a system-issued command which can create a record is refused
  with `ESS-AUTHOR-045`. The case is "the second call is refused": the first call creates the
  record and a second call with the same input answers a refusal. No separate story.

SW-D6, the instance-less await for set effects. A: one acceptance sentence on this story. B: a separate story. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation).

RN-D4, the no-subject system-issued command. A: the acceptance sentence above, as SW-D6 did for set effects. B: a separate story. Decided: A, 2026-11-24 16:29 CET (the coordinator, under the owner's delegation), taken on the default.

## Second adopter report, 2026-11-27 (0.99.8)

Another downstream adopter, on 0.99.8 (read-only probes of its specification), with the same gap on
three commands that only a `system: true` actor holds:

- a command that records a purchase;
- a command that opens a session, which needs a channel that another system-issued command creates
  first, so the arrangement is itself a chain of system-issued steps;
- a command that reports usage.

What 0.99.8 does with them:

- An authored scenario that sends such a command as the system actor is refused `ESS-AUTHOR-044`.
- An authored scenario that awaits it is refused `ESS-AUTHOR-045` (the awaited step names no instance).
- Synthesis refuses its scenarios `ESS-SYNTH-023`.
- The adopter's runtime conformance run therefore skips every awaited system step: 50 of its 62 skips
  are such steps.

Reproduced on 0.99.8 with a minimal model (one entity with a lifecycle, a command that creates a row
and is granted only to a `system: true` actor, a second command whose row-set refusal reads rows that
first command creates). `ess verify conform synthesize` ends "1 scenario(s), 7 refusal(s)": the
creating command's `recorded` outcome is `ESS-SYNTH-023`, and the second command's outcomes are refused
`ESS-SYNTH-023` or `ESS-SYNTH-001` where their rows need the system step. The `ESS-AUTHOR-044`/`045` refusals
were not reproduced on that model (no authored scenario was written for it).

What this adds to the story:

- The chain. A scenario for a system-issued command whose arrangement needs another system-issued
  creating command awaits both in order; the second await names the identity captured from the first
  (the instance-less await this story already decides on, with `capture:`). Acceptance: a model whose
  system-issued command needs a record that another system-issued command creates gets a scenario that
  awaits the creator, captures its identity, and awaits the command on it, and the interpreted target
  passes it.
- The same refusals reach authored scenarios: an authored act that awaits a system-issued creating
  command names no instance and states the identity it expects to capture, rather than being refused
  `ESS-AUTHOR-045`. Acceptance: such an authored scenario compiles and the interpreted target passes
  it.
- The report-usage shape (a command that reads and emits and changes no row, or changes rows by
  `instances:` only) is covered by the two amendments above (2026-11-24); the adopter's command still
  gets no scenario on 0.99.8, so the exact outcome shape is to be read from the adopter's specification
  before this story is closed: a command that matches an amendment and is still refused is a defect of
  that amendment, not a new case.
- The consumer effect is the measure: with the three shapes witnessed, the awaited system steps of the
  adopter's runtime run instead of being skipped. This story does not make a runtime implement the
  await; a runtime that answers "unsupported" still skips, which is the existing contract.

CHANGELOG: checked 2026-11-27 on `main`, sections 0.99.3 to 0.99.8: no entry lifts `ESS-SYNTH-023` for a
creating command or `ESS-AUTHOR-044`/`045`. No new decision: the open
decision of this story (how an await names a record that does not exist yet) now carries a second
adopter.

## Repro, 2026-11-27 (0.99.8): the authored and the synthesized refusals

Reproduced on 0.99.8 with a minimal model, committed under
`.engineering/repro/a-system-issued-creating-command-is-witnessed/B-system-issued-create/` (`README.md`, the
commands, two authored scenarios in `scenarios/`, and `out/`). One entity with a lifecycle, one command that
only a `system: true` actor holds, whose outcome creates the row.

- B1, an authored act that names the system actor: refused `ESS-AUTHOR-044` ("is sent as the system itself;
  the system issues it, so the act names no actor and the scenario awaits it"). This one is the intended
  answer; its help points at B2.
- B2, the same act with no actor (awaited): refused `ESS-AUTHOR-045` ("issued by the system and awaited on a
  record, and its outcome creates the record it addresses, so no record exists before the step").
- `ess verify conform synthesize` writes 0 scenarios and 1 refusal: `ESS-SYNTH-023` on the creating outcome.
  No route in 0.99.8 gives that outcome a scenario without granting the command to a non-system caller.

So `ESS-AUTHOR-044`/`045`, which the section above says were not reproduced, are reproduced, and the three
refusals (two authored, one synthesized) are one gap: a system-issued step that creates its record cannot be
awaited on the creation it claims. Acceptance, added: B2 compiles, the awaited act being checked on the event
and the view row it states; B1 stays refused `ESS-AUTHOR-044`; synthesis writes a scenario for the creating
outcome that the interpreted target passes. Test: the B model with its two scenarios.
