---
format: aep.planning-md/3
id: story:the-generated-rust-port-of-a-component-that-accepts-no-command-and-declares-no-view-builds-with-warnings
kind: story
status: active
title: The generated Rust port of a component that accepts no command and declares no view builds with warnings denied, with its constructor unchanged.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#793
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T02:12:59Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T02:13:01Z", actor: "human:timo", revision: 3}
---
# d793: a generated Rust port with no handler and no query builds without warnings

Size: S (about 50 changed lines: 10 source, 40 test).
Fixed suite bytes of committed models: none. Generated Rust bytes change only for a component with
no accepted command and no view; every committed Rust port has a reader of `behaviors` (checked:
each committed file holding "the port over the component's obligations" outside
`crates/generate/ess-synth/src` and outside Go holds `self.behaviors`).
Patch-release candidate: yes (no committed byte, no diagnostic).

## Need

beyond10x/ess#793. For a component that accepts no command and declares no view, the Rust target
writes a port struct with a private field `behaviors` that nothing reads. `cargo build` of the
generated workspace warns `field 'behaviors' is never read`, and a build with warnings denied fails.

## Reproduction

Reproduced on 0.98.0 (generation; the warning itself was read off the generated text, no `cargo`
was run). Model `repro/793/`: component `stock` accepts `depot.shelf.AddShelf` and owns a view;
component `labels` owns domain `depot.label` holding one newtype and accepts nothing.

```console
$ ess generate synthesize --path . --target rust --out gen
11 capabilities: 10 generated, 0 obligation(s), 1 refused
17 artifact(s), written to gen
$ grep -c self.behaviors gen/crates/labels/src/lib.rs gen/crates/stock/src/lib.rs
gen/crates/labels/src/lib.rs:0
gen/crates/stock/src/lib.rs:2
```

`gen/crates/labels/src/lib.rs:27-29`: `pub struct Labels<B> { behaviors: B, outbox: … }`, and no
`impl<B> Labels<B> where B: …` block follows.

## Outcome

A port whose bound list is empty holds nothing unread: the generated crate of such a component
builds under `-D warnings`. `Port::new(behaviors: B)` keeps its signature, because the generated
server entry (`crates/generate/ess-synth/src/rust/entry.rs:40`) and the web bridge
(`crates/generate/ess-synth/src/web/bridge.rs:639`) call `<Port>::new(<behavior>)` for every
component. A port with at least one handler or query is byte for byte unchanged.

## Acceptance

1. Red today: synthesizing the reproduction model for `Target::Rust`, the artifact
   `crates/labels/src/lib.rs` holds the field line `    _behaviors: B,` and no line
   `    behaviors: B,` (rustc's `dead_code` lint does not report a field named with a leading `_`).
2. `crates/stock/src/lib.rs` of the same synthesis is byte-identical to what 0.98.0 writes.
3. `Labels::new` takes one argument of type `B`, as before.
4. Every committed Rust example tree under `generated/` keeps its bytes (the existing byte
   comparisons of `ess-synth` stay green).

## Scope

- `crates/generate/ess-synth/src/rust/port.rs`, `port_struct` (lines 144-216): the struct text at
  152-168 is written before `bound_list` (line 170) is computed. Compute `bounds` first; when it is
  empty, write the field as `_behaviors: B` (and `_behaviors: behaviors,` in `new`), constructor
  signature unchanged. Chosen over `#[allow(dead_code)]` on the field: no lint attribute in
  generated code. The value stays alive for the port's lifetime, as today.
- Nothing else in the Rust target reads the field (`port.rs:264, 271, 353, 360` are inside the
  bounded `impl`, skipped when `bounds.is_empty()`, line 182).

## Owned files

- `crates/generate/ess-synth/src/rust/port.rs`
- `crates/generate/ess-synth/tests/synthesis.rs` (new `#[test]` only)
- `CHANGELOG.md`

## Not yours

- The Go target: Go does not warn on an unused struct field, and the committed Go ports without a
  reader (`generated/go/billing/components/emailservice/emailservice.go` and two more) stay.
- The one refused capability in the reproduction's count (`1 refused`): not part of this issue.

## Tests first

In `crates/generate/ess-synth/tests/synthesis.rs` (existing target), a new
`a_port_with_no_handler_and_no_query_leaves_no_field_unread`: the reproduction model inline,
synthesize in process for `Target::Rust`, assert acceptance 1-3 on the artifact text. No `cargo`
build of the generated workspace in the test step (a compile of a generated workspace belongs in
the `slow` lane, if at all). Runs in well under 5 s (bound 60 s per test, 120 s per step).

## Gate

One at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-synth --all-targets --locked -- -D warnings`
- `cargo test -p ess-synth --locked --test synthesis`
- the `ess-synth` byte pins (`AGENTS.md`, "A gate of named test targets includes the
  repository-wide byte pins"): `--test served_codec_names`, `--test go`, one at a time
- The table row "a committed example tree under `generated/` or a projection" applies only if a
  committed tree moves; acceptance 4 says none does. If one does, run `ess-synth`'s
  `feasibility`, `adversary_related_guard_behaviour_pass1` and `http_binding_rust`.
- `task test-xtask` (the unit edits `CHANGELOG.md`)
- No stamp: `ess-synth/src` is not among the sources `synthesis-sources.digest` hashes.

## Depends on

Nothing.

## Changelog

Under `Fixed`:

- `ess generate synthesize --target rust` writes the port of a component that accepts no command
  and declares no view without a field nothing reads, so the generated workspace builds with
  warnings denied (#793).

## Fit review

1. Need: a generated workspace must build clean under the strictest common lint setting; one
   component shape breaks that. No proposed syntax.
2. Class: defect. The generated crate claims to compile (`port.rs:156-158` doc: "a component that
   compiles and refuses, in the type system, everything not yet implemented"), and under
   `-D warnings` it does not.
3. Expressible today: only by giving the component a command or a view it does not have.
4. Fit: no authored surface; generated text only.
5. A second adopter: any system with a component that owns only shared types (a "kernel" or
   "shared vocabulary" component).
6. Cost: generated bytes for this shape only.
7. Considered: change nothing (rejected); drop the field and change `new` to take nothing (rejected:
   breaks the two callers above and every adopter's wiring); keep the field in a form the lint does
   not report (chosen).

## Decisions

- Decision: accept as proposed (the issue's expected behaviour, no design question).
