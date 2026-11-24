---
format: aep.planning-md/3
id: story:generated-codecs-and-conformance-write-and-read-a-flat-union-in-the-layout-its-model-declares
kind: story
status: active
title: Generated codecs and conformance write and read a flat union in the layout its model declares
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-tagged-union-declares-that-its-tag-sits-beside-its-variant-s-members-in-one-flat-object
- derived_from: story:generated-codecs-and-the-suite-use-a-unions-declared-content-key
- depends_on: story:generated-codecs-and-the-suite-use-a-unions-declared-content-key
- depends_on: story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:25:18Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-24T03:25:19Z", actor: "human:timo", revision: 7}
---
## Need

After `sl05-union-flat-wire` a union may declare `layout: flat`, and the generated targets refuse
such a model by name. `story:generated-codecs-and-the-suite-use-a-unions-declared-content-key`
(active, beyond10x/ess#637) makes every target read a union's declared content key through the
accessor that takes the resolved union, and puts the key in the suite's structural declaration
under the next `ess-conformance` pair. The targets it changes are the ones that must also read the
flat layout:

- Rust: `crates/generate/ess-synth/src/rust/wire.rs:167-168`, `:270`;
- Go: `crates/generate/ess-synth/src/go/http.rs:635-636`, `:761`;
- web: `crates/generate/ess-synth/src/web/catalog.rs:486`;
- conformance: `crates/verify/ess-conformance/src/input.rs:1753`, `accessor.rs:242-250`,
  `binary64_values.rs:420`, `one_time_response.rs:402`, and the Go and TypeScript runtimes
  (`src/go/response.go:720-723`, `src/go/one_time_execution.go:366-368`, `src/ts/response.ts`).

This unit adds the flat layout to those same paths. The adjacent form with a declared content key
is the active story's and is not restated here.

## Outcome

Every target that reads a union's encoding reads the flat layout too. A flat value is written as
the tag member followed by the struct's members in declaration order, and read by the tag first and
then the members. An unknown tag value is refused as today. Conformance writes flat inputs and reads
flat answers. The suite's union declaration carries `layout: flat` under the pair the active codecs
story opens, so the suite format moves once for the union's wire layout. A model without a flat
union generates the same bytes in every target.

## Acceptance

Suite pair (D77, 2026-11-24): this story's suite vocabulary is written at `ess-conformance/66` (ordinary) and `/67` (coverage), opened by story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet; where this text names `ess-conformance/64` and `/65`, "the new suite pair", "the next suite pair" or a pair "after `/63`" as the pair this story writes, read `/66` and `/67` (`/64` and `/65` opened in 0.93.0 and are written since 0.95.0).

- Red first, in `crates/generate/ess-synth/tests/`: a generated Rust server and a generated Go
  server over the `sl05-union-flat-wire` reproduction decode `{"kind": "parcel", "name": "north"}`
  and `{"kind": "none"}`, and encode them back byte for byte. Each refuses the adjacent form as
  malformed. Before the change both refuse the model by name.
- The web catalogue writes the flat form.
- A synthesized suite over a command taking the union sends the flat form, and the interpreted
  target and the Go and TypeScript runtimes pass it. Each runtime refuses the adjacent form of a
  flat union as an undeclared member.
- The suite's union declaration carries `layout: flat` under the `ess-conformance` pair the active
  codecs story opens, and admission refuses it below that pair. If that pair has already been
  released when this unit lands, this unit opens the next pair instead and says so in
  `CHANGELOG.md`.
- A suite for a model without a flat union is byte-identical before and after. The committed
  example trees under `generated/` are unchanged (`ess-synth`'s whole test set and the
  repository-wide byte pins, per `AGENTS.md`).

## Fit review

Second report, G5, unit 2 of 2 of the union wire half. The fit review is in
`sl05-union-flat-wire`. Every target handles the layout or refuses it by name.

## Decisions

None beyond those settled in `sl05-union-flat-wire`.

## Scope

- cited: the files listed under Need
- inferred: the layout-aware accessor in `crates/generate/ess-gen/src/schema.rs` (written by the
  projections story); run `ess-conformance` one `--test` at a time
- Size: M, about 450 changed lines.

## Depends on

- `sl05-union-flat-wire`.
- `story:generated-codecs-and-the-suite-use-a-unions-declared-content-key` (the threaded accessor in
  every target and the suite pair). This unit is derived from it and changes the same files, so it
  lands after it.

## Source

A downstream application, 2026-11-24, second report, G5.
