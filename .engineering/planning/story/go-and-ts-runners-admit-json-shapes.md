---
format: aep.planning-md/3
id: story:go-and-ts-runners-admit-json-shapes
kind: story
status: implemented
title: Generated Go and TypeScript runners admit json shapes
tags:
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-09T01:13:59Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-09T01:13:59Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-09T01:14:08Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

The generated Go and TypeScript conformance runners admit a suite whose shapes include `json`,
and run it.

## Evidence

A consumer (ess 0.56.0 and 0.55.0): a specification with a field of type `Json` validates and
synthesizes; `ess verify conform synthesize --target go` writes `"kind": "json"` for that shape,
and the generated runner refuses its own suite at admission: `suite admission:
<Command>/outcome/<name>: unknown primitive`. `admitShape` in
`crates/verify/ess-conformance/src/go/runtime.go` lists `string`, `boolean`, `integer`,
`decimal`, `timestamp`, `duration`, `uuid` and `bytes` and no `json` (the primitive switch in
`admitShape`, and a second list of the same primitives later in the file); `admitShape` in
`crates/verify/ess-conformance/src/ts/runtime.ts` the same. With `json` added in a scratch copy
the consumer's 24-scenario suite ran and reported (13 pass, 11 fail, each explained).

## Acceptance

- Both runtimes admit the `json` primitive wherever they admit the others (every primitive list
  in `runtime.go` and `runtime.ts`), and compare a `json` value as the native runner does.
- A regression test synthesizes a suite from a model with a `Json` field for Go and TypeScript and
  runs it against a target; admission succeeds and a wrong `json` value fails a scenario.
- The Rust native runner's behaviour on the same suite is unchanged.
