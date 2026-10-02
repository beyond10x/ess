---
format: aep.planning-md/3
id: story:rust-binding-unused-event
kind: story
status: draft
title: Generated Rust bindings compile when mapping reads no event field
relations:
- serves: vision:O2
- informed_by: story:served-store-and-entry
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests
revision: 3
---
## Outcome

A fully determined binding whose target command has no input fields, or whose mapping needs no event field, emits warning-clean Rust and remains executable.

## Fit review

1. Need: a NoteAdded event invokes an inputless Answer command in another served component. The validated brand-free notebook reduction emits answer_on_note(event: &NoteAdded) returning Answer {}, and strict Rust compilation fails on the unused event parameter. No new syntax is requested.
2. Class: generated-code defect. The repository requires generated native behavior to compile under strict warnings. Measured target/backlog-input/318-reach-progress.log reports unused variable event at generated notebook-system/src/lib.rs:74 and exits101 (14 other cases passed).
3. Existing expression: the admitted binding already expresses the behavior. Adding a fake command input solely to consume the event changes the model and is not the repair. The318 reachability regression now uses a real mapped identity for its separate purpose; this record preserves the original defect.
4. Fit: fix the transformation emitter in crates/generate/ess-synth/src/rust/system.rs:343-430, retaining its public function signature types and execution meaning. Check inputless, constant-only and omitted-Optional-only mappings, where event may also be unused, and a field-reading control. Selection mappings and Go behavior remain unchanged unless a sibling reproduces the same defect.
5. Second adopter: an audit event triggers an inputless cache refresh. The event is the cause even when no payload enters the command.
6. Cost: generated parameter spelling or local unused binding only; no authored keyword, format, runtime port or dependency change is intended.
7. Alternatives: suppress warnings globally (hides defects); require a dummy input (changes consumer contract); chosen local emitter handling of a deliberately unread event, with strict compilation and binding execution regressions.

## Decisions

Accept as a separate bounded emitter defect discovered while implementing318. Keep exact failing generated output and log as evidence. Do not claim it was fixed merely because the318 fixture now maps a real identity.

## Acceptance

- inputless_binding_compiles_and_invokes: synthesize valid inputless target; generated Rust compiles with -D warnings and executes the binding.
- literal_only_and_optional_only_mapping_compile: equivalent no-event-read siblings remain warning-clean and return declared values/presence.
- event_mapping_still_reads_payload: a nonempty event-field mapping retains the expected invocation value; changing it fails the test.
- No new ignored case, generated format or broad warning suppression.
