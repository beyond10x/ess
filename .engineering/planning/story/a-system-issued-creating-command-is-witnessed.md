---
format: aep.planning-md/3
id: story:a-system-issued-creating-command-is-witnessed
kind: story
status: draft
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
revision: 3
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

## Scope

- cited: docs/design/system-issued-commands.md
- cited: crates/verify/ess-conformance/src/system_issued.rs
- cited: crates/verify/ess-conformance/src/synthesize/system_issued.rs
- inferred: crates/verify/ess-conformance/src/target.rs
- inferred: crates/verify/ess-conformance/src/runner.rs
- inferred: crates/verify/ess-conformance/src/interpret
- inferred: crates/verify/ess-conformance/src/go/runtime.go
- inferred: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/tests/system_issued_synthesis.rs

confidence: medium

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
6. Cost: a suite format change, because `await_outcome` gains an instance-less form. That is a new
   suite major, a target-interface change for Rust, Go and TypeScript adapters (breaking for an
   adapter that requires `instance`), and a CHANGELOG entry. No `ess/` source format change.
7. Alternatives: (a) change nothing and keep `ESS-SYNTH-023`, which the help already explains;
   (b) drive the creating step as an ordinary caller, which contradicts the system actor's grant
   (beyond10x/ess#542); (c) an instance-less await that captures the created identity, preferred if
   the target interface can carry it.

## Decisions

- defer: the need is real, but it needs a decision on how an await names a record that does not
  exist yet (alternative c, its suite major and target interface). Record a `decision-blocker` with
  `blocks:` to this story before it leaves draft.
- Same gap as `story:a-create-only-the-system-issues-gets-a-synthesized-scenario` (adopter report of
  2026-10-26, no reproduction). Keep this story, which carries the reproduction, and fold the other
  into it.
