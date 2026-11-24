---
format: aep.planning-md/3
id: story:generated-codecs-and-conformance-write-and-read-a-flat-union-in-the-layout-its-model-declares
kind: story
status: draft
title: Generated codecs and conformance write and read a flat union in the layout its model declares
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object
revision: 1
---
## Need

After `sl05-union-flat-wire` a union may declare `layout: flat`, and the generated targets refuse
such a model by name. Each target writes and reads a union through `union_content_key`
(`crates/generate/ess-gen/src/schema.rs:76-77`):

- Rust: `crates/generate/ess-synth/src/rust/wire.rs:167-168`, `:270`;
- Go: `crates/generate/ess-synth/src/go/http.rs:635-636`, `:761`;
- web: `crates/generate/ess-synth/src/web/catalog.rs:486`;
- conformance: `crates/verify/ess-conformance/src/input.rs:1753`, `accessor.rs:242-250`,
  `binary64_values.rs:420`, `one_time_response.rs:402`.

## Outcome

Every target that reads `union_content_key` reads the union's layout instead. A flat value is
written as the tag member followed by the struct's members in declaration order, and read by the
tag first and then the members. An unknown tag value is refused as today. Conformance writes flat
inputs and reads flat answers. A model without a flat union generates the same bytes in every
target.

## Acceptance

- Red first, in `crates/generate/ess-synth/tests/`: a generated Rust server and a generated Go
  server over the `sl05-union-flat-wire` reproduction decode `{"kind": "parcel", "name": "north"}`
  and `{"kind": "none"}`, and encode them back byte for byte. Each refuses the adjacent form as
  malformed. Before the change both refuse the model by name.
- The web catalogue writes the flat form.
- A synthesized suite over a command taking the union sends the flat form, and the interpreted
  target and the Rust runner pass it.
- A suite for a model without a flat union is byte-identical before and after. If a suite carrying
  a flat value cannot be read correctly by an older runner, the suite moves to the next major pair
  and older readers refuse it by version. The implementor decides which, with a test for either
  outcome.
- The committed example trees under `generated/` are unchanged (`ess-synth`'s whole test set, per
  the package-gate table in `AGENTS.md`).

## Fit review

Second report, G5, unit 2 of 3. The fit review is in `sl05-union-flat-wire`. Every target handles
the layout or refuses it by name (skill, question 4).

## Decisions

None beyond SL-D13.

## Scope

- cited: the files listed under Need
- inferred: `crates/generate/ess-gen/src/schema.rs` (`union_content_key` becomes a layout-aware
  accessor), `crates/verify/ess-conformance/src/go/` and `ts/` runtimes where they decode a union;
  run `ess-conformance` one `--test` at a time
- Size: M, about 560 changed lines.

## Depends on

`sl05-union-flat-wire`.

## Source

A downstream application, 2026-11-24, second report, G5.
