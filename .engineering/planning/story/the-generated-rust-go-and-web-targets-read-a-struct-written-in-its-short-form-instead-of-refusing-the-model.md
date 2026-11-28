---
format: aep.planning-md/3
id: story:the-generated-rust-go-and-web-targets-read-a-struct-written-in-its-short-form-instead-of-refusing-the-model
kind: story
status: draft
title: The generated Rust, Go and web targets read a struct written in its short form instead of refusing the model
summary: 'All three server targets refuse a struct with shorthand: by name, although the format and the published schema admit it (gap report, 2026-11-24)'
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-number-shorthand-struct-decodes-in-generated-rust
- informed_by: story:a-null-when-absent-field-beside-a-shorthand-is-realized-or-refused
revision: 2
---
## Need

A downstream specification declares a struct with `shorthand: <field>` (`ess/27`) and asks `ess generate synthesize` for a Rust
crate. The generator refuses the whole workspace by name, so the specification drops the shorthand and every document is
written in the long form. The format admits the construct, the published schema and OpenAPI describe it as `oneOf [field,
object]`, and `ess generate types` reads it on Rust, Go and TypeScript, but the synthesized servers and the web app do not.

The refusal is a recorded deviation with no story behind it. The 0.63.0 CHANGELOG entry for `ess/27` says "Synthesis of a Rust or
Go server or the web app refuses such a struct by name, since their codecs decode it from its object only"
(`crates/generate/ess-synth/src/failure.rs:369-402`, `shorthand`; called from `lib.rs:491`, `rust/mod.rs:196`, `go/mod.rs:324`,
`web/mod.rs:304`). Its doc comment says the short-form decoder is "owed rather than silently missing". No story in the store owns it:
`story:a-number-shorthand-struct-decodes-in-generated-rust` is the `ess generate types` Rust target only, and its reason (serde's
`arbitrary_precision` hands a number to a visitor as a map, `crates/generate/schema-contract/src/realize/rust.rs:502-504`) does not
apply to synthesis, whose Rust codecs are generated functions over the crate's own `json::Value`
(`crates/generate/ess-synth/src/rust/wire.rs:221-363`, `struct_body`).

## Reproduction

Brand-free, `format: ess/37`, domain `demo.lim`: `types: [{name: demo.lim.Bound, kind: struct, shorthand: max, fields: [{name: max,
type: Integer}, {name: min, type: Optional<Integer>}]}]`, an entity holding a `Bound` and a creating command taking one. Installed
0.98.0:

| surface | result |
|---|---|
| `ess specify validate` | valid |
| `ess generate synthesize --target rust` | `rust target cannot emit this workspace`, `types.demo.lim.Bound.shorthand: ... decodes the struct from its object only, not from the field's value alone` |
| `--target go`, `--target web` | the same refusal |
| `--target clap` | exit 0 (the command-line target keeps its output, `failure.rs:376-378`) |
| `ess generate types --target go` | exit 0 |
| `ess generate types --target rust` | refused `rust_shorthand_number` (an `Integer` shorthand; a `String` shorthand realizes) |
| `ess verify conform author` with `input: {bound: 5}` | `ESS-AUTHOR-015 ... expected demo.lim.Bound as a mapping, found a number` (story:conformance-authors-sends-and-observes-a-struct-written-in-its-short-form-and-stops-withdrawing-the-scenarios) |

## Findings covered

- Gap 1 of the thirteenth report: the Rust target refuses `shorthand:`; the Go target and the web app refuse it the same way.

## Fit review

1. Need: a server generated from a specification reads every value the specification admits, the short form of a struct included.
   Domain fact; the requester's workaround (drop the shorthand) is not a design.
2. Class: defect. ESS contradicts its own published contract: `docs/design` and `website/docs/reference/formats.md` publish the
   `oneOf`, and a generated server that refuses the short form would answer a conforming request as malformed. The refusal's own
   comment records the decoder as owed.
3. Expressible today: no. The only idiom is to remove the shorthand, which moves the figure the reader sees out of the document.
4. Fit: grids unchanged. Decoders only; every writer the model generates writes the long form (`ess/27` row of
   `website/docs/reference/spec-versions.md`), so no encoder, projection or IR changes. The short form is unambiguous on the wire
   because the shorthand field is a scalar and never an object (`crates/specify/ess-domain/src/shorthand.rs`, "a scalar on the wire").
   The web app reuses the Rust decoders (`web/bridge.rs` imports `rust::wire`), so it follows from the Rust change. Targets:
   Rust, Go, web read it; Clap keeps its output; Entity Runtime and the interpreter never receive a short form (authoring refuses
   it, story:conformance-authors-sends-and-observes-a-struct-written-in-its-short-form-and-stops-withdrawing-the-scenarios).
5. Second adopter: any specification that publishes a protocol whose request member is a scalar or an object (a filter given as `"x"`
   or as `{op: x, value: y}`).
6. Cost: no format, no IR change, no new diagnostic. Two decoder generators and the removal of one refusal; changed bytes only for
   a model that declares a shorthand. No committed tree under `generated/` or `examples/` declares a model shorthand (the one hit
   of `grep -rl shorthand` there is a UI document), so no byte pin moves.
7. Alternatives: keep the refusal and say so in the reference (the reporter's present state); decode only in Rust and leave Go
   refusing (a disparity between targets for one construct, refused by the red flag "works in one construct and leaves a sibling").

## Decisions

Accept as proposed. FM-D1, what a generated server reads of a struct with `shorthand:`. Decided: A, 2026-11-24 16:36 CET (the coordinator, under the owner's delegation): the Rust, Go and web decoders read the short form and the long form, and writers write the long form. Reason: the specification declares the short form and its schema already publishes it. Cost: two decoder generators and the refusal removed; no format, IR or projection change. B (keep the refusal and say so in the reference) and C (Rust and web only, Go keeps refusing) not taken.

## Acceptance

- shorthand_model_synthesizes_on_every_server_target: `ess generate synthesize` of the reproduction exits 0 for `rust` (both layouts),
  `go` and `web`; the refusal test in `crates/generate/ess-synth/tests/refusal_policy/shorthand.rs` flips to assert it, and `clap` is
  unchanged.
- rust_decoder_reads_the_short_form: the generated Rust decoder of `demo.lim.Bound` reads `5` as `{max: 5}` with `min` absent and
  `{max: 5, min: 1}` as before; a string, an array and `null` are refused with the decoder's error naming the path and what is
  expected ("an object, or the `max` value"); an object member the struct does not declare is still refused unless the struct ignores
  undeclared fields.
- go_decoder_reads_the_short_form: the same four cases through the generated Go decoder, in the same test.
- invariants_hold_for_both_forms: a struct with an invariant and a shorthand refuses a broken value written in either form, naming
  the invariant.
- shorthand_field_kinds: the short form is read for a shorthand field of each kind the compiler admits: `String`, `Integer`, `Boolean`,
  `Binary64` (exact token kept, `-0` keeps its sign), an enum and a newtype over one of them.
- web_bridge_accepts_a_short_form_request: a bridge request carrying the short form is decoded, not answered `undecodable`.
- writers_write_the_long_form: every encoder still writes `{max: 5}`; a round trip of `5` through decode and encode is `{max: 5}`.
- the_test_step_stays_in_budget: each new test runs under 60 s; the step under 120 s.

## Scope

- cited: `crates/generate/ess-synth/src/failure.rs` (`shorthand`, 369-402; remove, or keep for a target that cannot decode)
- cited: `crates/generate/ess-synth/src/lib.rs:491`, `rust/mod.rs:196`, `go/mod.rs:324`, `web/mod.rs:304` (the four call sites)
- cited: `crates/generate/ess-synth/src/rust/wire.rs` (`struct_body`, 318-363; the short-form branch before `closure`)
- cited: `crates/generate/ess-synth/src/go/http.rs` (struct decoder, 738-760)
- cited: `crates/generate/ess-synth/tests/refusal_policy/shorthand.rs` (flips), a new case beside it
- inferred: `CHANGELOG.md`; the reference sentence that names the refusal, if one is left
- Size: 8 files, none of them a format, an IR or a projection file.
