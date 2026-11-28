---
format: aep.planning-md/3
id: story:a-component-that-publishes-nothing-keeps-the-events-its-commands-emit-unpublished
kind: story
status: draft
title: A component that publishes nothing keeps the events its commands emit unpublished, the reference says so and a test holds it, and the source of a created identity is the owner's decision
summary: Emission and publication are separate and the generated ports already no-op an unpublished event; the 501 is an executor rule; reading a created identity without an event is EV-D1, owner decision
tags:
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-command-that-emits-an-event-only-another-component-publishes-is-reported-as-an-advisory
revision: 1
---
## Summary

A component that publishes nothing (one HTTP listener, no broker, no event transport) still has to declare
the event in which a creating command publishes the new identity, because ESS reads a created identity from
an emitted event. A downstream application's executor refuses such a create with a 501 saying the component
does not publish the event, and the application's specification carries the event only to satisfy the rule.
The finding has two halves. The first is the executor's alone: ESS never requires an emitted event to be
published, and its own generated code already treats an emitted-but-unpublished event as a no-op. ESS owes a
sentence on that, and a test that pins it. The second is a request to change the rule that reads a created
identity from an event; that is the owner's (`decision-blocker:a-create-names-its-identity-without-a-published-event`).

## Facts (tag 0.99.1, read, no run)

1. The rule. A `creates:` outcome's `instance:` names a field of an event the same branch emits, typed as the
   entity's identity; with no such field the specification is refused `ESS-COMMAND-001` (an undeclared
   reference; family `COMMAND`, class 001: `crates/specify/ess-compiler/src/resolve.rs:592`,
   `website/docs/reference/diagnostics.md:11-22`) naming "no field of an emitted event of it"
   (`resolve.rs:4263-4268`, text of the surface at `crates/specify/ess-domain/src/command.rs:1126`), or
   `ESS-COMMAND-002` (a type mismatch, as `ESS-BINDING-002` and `ESS-VIEW-002` are) when the field is not typed
   as the identity (`resolve.rs:4269-4276`, `diagnostics.md:20`).
   The same check in `ess-domain`: `crates/specify/ess-domain/src/entity.rs:1464-1517`. The surface is a function
   of the verb: `Effect::Creates => InstanceSurface::EmittedEvent` (`command.rs:1096-1103`).
2. The reason, as the code states it: "the instance did not exist when the command was issued, so the caller
   could not have named it; what the specification can say is where the new identity becomes observable"
   (`command.rs:130`). No page under `docs/design/` gives the reason beyond
   `docs/design/outcome-shapes.md:248-262`, which shows `creates:` with `instance:` and keeps the meaning of
   `unknown_instance:` on it; the reason lives in the crate documentation and the diagnostics page.
3. Emission and publication are separate. A component's `publishes:` is a statement about its transport; the
   rules on it are that it names declared events and that a component is not empty
   (`crates/specify/ess-domain/src/component.rs:8-22`), and two publishers of one event are allowed
   (`component.rs:96-112`). The design page says so in as many words: "Publication is independent of operation
   emission: published-but-not-emitted events select their reacting bindings; emitted-but-unpublished events do
   not ... Unpublished emissions retain their event type and complete outcome context, without claiming a
   component transport delivery" (`docs/design/ess-evolution/service-contract.md:88-93`; its test plan asks for a
   separate emitted-but-unpublished fixture, `:127`).
4. No ESS rule and no generated code requires an emitted event to be published. The generated Rust port writes
   a typed outbox for the published events only, and the handler arm of an outcome whose events are all
   unpublished is `{ .. } => {}` (`crates/generate/ess-synth/src/rust/port.rs:116-135`, `:299-318`); the Go port
   filters by `component.publishes` the same way (`crates/generate/ess-synth/src/go/port.rs:137`, `:222`). The
   text "component does not publish" appears nowhere in the tree at 0.99.1: the 501 is the executor's rule.
5. The cost that is ESS's. The component's own suite keeps an event step of an event the component owns
   (`emits` is true when the event is published or its domain is owned,
   `crates/verify/ess-conformance/src/synthesize.rs:4680-4688`), so the creating scenario reads the identity from
   the event (`ScenarioStep::CaptureInstance`, `:4598-4604`) and a target that publishes nothing can only answer
   `observe_events` as `unsupported`, which fails the run as a target failure
   (`docs/design/conformance-target-process.md:17`, `:274`, `:294`). So the identity is observable to a suite only
   through an event, which is what request (a) asks to change.

## Acceptance

1. `website/docs/guides/specify/bindings-and-components.md` and `docs/design/ess-evolution/service-contract.md`
   state that an event a command emits and no component lists in `publishes:` is valid, that publication is a
   statement about a transport, that the generated port treats the event as a no-op, and that a created
   identity is read from the emitted event's field either way. A reader who has no transport is told to leave the
   event out of `publishes:` and not to declare a transport.
2. `an_emitted_event_no_component_publishes_validates_and_generates_a_noop_arm` pins facts 3 and 4 on a generic
   model: one component owning a domain, accepting a creating command whose identity event is in no `publishes:`;
   `ess specify validate` is clean, `ess generate synthesize` for `--target rust` and `--target go` writes a
   handler whose arm for the creating outcome sends nothing to the outbox, and the component's suite keeps the
   create's scenarios. Green on reading at 0.99.1; the test holds it.
3. The conformance reference says what an adapter answers for `observe_events` when it publishes nothing, and
   that a scenario reading a created identity from an unpublished event then fails as `unsupported`; the page
   names the choice of EV-D1 beside it.
4. The decision of the owner (EV-D1) is recorded on its blocker before any change to the rule of fact 1.

## Fit review

1. Need: a component with no transport states a creating command without declaring a transport.
2. Class: half documentation (ESS owes the sentence of Acceptance 1; the 501 is the executor's), half a request
   to change a validation rule that every released specification satisfies. That second half conflicts with a
   rule and is the owner's.
3. Covers: the reporter's request (a) a create may return its identity without an event, request (b) a way to
   mark an emitted event as not published, and the `UNMAPPED` rows for the created identity and for the version
   on replace. (b) is already expressible: leave the event out of `publishes:`. (a) is EV-D1 B. The `UNMAPPED`
   row for the version on replace is not covered here: it is a mapping question on a binding and the reporter's
   text gives no example (not assessed).
4. Borders: `story:a-command-that-emits-an-event-only-another-component-publishes-is-reported-as-an-advisory` (draft;
   it warns where another component publishes the event; this story must not make the same case warn for a
   component that publishes nothing), `story:a-later-event-of-a-creating-outcome-carries-the-minted-identity`,
   `story:a-creating-branch-sends-every-row-it-creates-under-its-own-identity`. A search of the store and the
   open issues found no story on a create without an event.
5. A second adopter: any component behind a plain request-response listener (a service that serves a database).

## Decisions

EV-D1 is on `decision-blocker:a-create-names-its-identity-without-a-published-event`, with the owner; its
fail-closed default is A (the rule stays).

## Scope

- `website/docs/guides/specify/bindings-and-components.md`, `docs/design/ess-evolution/service-contract.md`,
  `docs/design/conformance-target-process.md` (the `observe_events` row)
- a test in `crates/generate/ess-synth/tests/` (one binary, a new module) and the model beside it
- `crates/generate/ess-synth/src/rust/port.rs`, `src/go/port.rs` (read only unless EV-D1 moves)
