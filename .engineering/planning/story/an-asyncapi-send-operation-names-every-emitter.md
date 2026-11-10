---
format: aep.planning-md/3
id: story:an-asyncapi-send-operation-names-every-emitter
kind: story
status: active
title: The AsyncAPI send operation names every outcome that emits the event
summary: The send operation names one emitter of three (a2a wave 1, G7); defect.
tags:
- adopter-report
- defect
- ess-0.70.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/asyncapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/asyncapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/asyncapi_transport.rs
- confidence: inferred
  path: generated/asyncapi/email-service.yaml
- confidence: inferred
  path: generated/asyncapi/invoice-service.yaml
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:46Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:47Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Reported by a downstream specification on 2026-10-22 against ESS 0.67.0 (finding G7 of its wave-1
gaps): the generated AsyncAPI send operation for an event names only one emitter (the creating
outcome) and omits the others that emit the same event without changing an entity.

Reproduced on 0.92.0. Model: `demo.tasks.TaskUpdated` is emitted by `Create/created` (creates
`demo.tasks.Task`), `Continue/continued` (no subject) and `Message/direct` (no subject), all accepted
by one component `task-service`.

```
$ ess generate --path spec.yaml --kind asyncapi --out out
$ sed -n '/^operations:/,/^components:/p' out/asyncapi/task-service.yaml
operations:
  send.demo.tasks.TaskUpdated:
    ...
    description: |-
      Nothing in this specification reacts to `demo.tasks.TaskUpdated`.
      `demo.tasks.Create` emits it on `created`, which creates a `demo.tasks.Task` in `Open`. It carries the new instance's identity in `task_id`.
```

`Continue/continued` and `Message/direct` are not named.

## Where it lives

- `crates/generate/ess-gen/src/asyncapi.rs:1169` (`send`): the operation's description is the
  consumers plus `state_changes(ir, event)` (`:1226`).
- `asyncapi.rs:1233-1235`: `state_changes` skips every emitting outcome without a subject
  (`let Some(subject) = &outcome.subject else { continue; }`), and further down speaks only where
  the event is the one that publishes the subject's identity. An outcome that emits the event
  without changing an entity is never named.

## Fit review

2026-10-22, assessed with `.agents/skills/assessing-external-requests/SKILL.md`.

1. **Need.** A reader of the AsyncAPI document must see every outcome that sends the event, so a
   consumer knows every situation it arrives in. The reporter proposed no syntax.
2. **Class: defect.** The model declares three emitters and the document names one.
3. **Already expressible?** No authored spelling changes what the projection lists.
4. **Fit.** No authored surface. The send operation names every command outcome that emits the
   event, subject-less ones included, in a fixed order. The sentences `state_changes` writes for
   subject-bearing emitters stay as they are.
5. **Second adopter.** Any service whose notification event is sent by a state change and by a
   pure acknowledgement (`MessageSent` from `Create` and from `Resend`).
6. **Cost.** No format version. Generated AsyncAPI bytes change only for events with a
   subject-less emitter.
7. **Alternatives.** (a) Change nothing. (b) Name subject-less emitters only when the event has
   more than one emitter: keeps more bytes, but an event whose only emitter is subject-less would
   still name nobody. (c) Chosen: name every emitter.

## Decisions

2026-10-22: **accept.** An emitter is every command outcome whose `emits:` names the event,
subject-less outcomes included. Emitters are listed ordered by command name, then outcome name.
An event whose emitters all have a subject keeps its bytes; an event with a subject-less emitter
changes its bytes, and that change is the fix. The checked-in `generated/asyncapi/*.yaml` are
regenerated where they move, and the change names each moved file.

## Acceptance

Fixture, written by the test: an event `demo.tasks.TaskUpdated` emitted by `Create/created`
(creates `Task`), `Continue/continued` (no subject) and `Message/direct` (no subject).

- **Every emitter is named.** Transition: `asyncapi` projection of the fixture. Before: the send
  operation names `Create/created` only. After: it names all three, in the order
  `Continue/continued`, `Create/created`, `Message/direct`. Test:
  `crates/generate/ess-gen/tests/asyncapi.rs`, case `a_send_operation_names_every_emitting_outcome`.
  Failing before: the case finds one emitter on the current tree. Mutation check: restoring the
  subject-less skip at `asyncapi.rs:1233-1235` for emitter naming fails the case.
- **Order is fixed.** Transition: the same fixture with commands declared in reverse order. After:
  byte-identical output. Same file, case `emitters_are_ordered_by_command_then_outcome`.
- **Subject-only events keep their bytes.** Transition: projection of an event whose emitters all
  have a subject. After: byte-identical to the current tree's. Same file, case
  `an_event_whose_emitters_all_have_a_subject_keeps_its_bytes`; every existing case in
  `tests/asyncapi.rs` and `tests/asyncapi_transport.rs` keeps its verdict or is named as re-pinned
  because its event has a subject-less emitter.

## Scope

- cited: crates/generate/ess-gen/src/asyncapi.rs
- cited: crates/generate/ess-gen/tests/asyncapi.rs
- inferred: crates/generate/ess-gen/tests/asyncapi_transport.rs
- inferred: generated/asyncapi/email-service.yaml
- inferred: generated/asyncapi/invoice-service.yaml

confidence: high

## Schedule

Release 0.70.0 (defect, wave A unit A8). The release tag on the artifact marks it.
