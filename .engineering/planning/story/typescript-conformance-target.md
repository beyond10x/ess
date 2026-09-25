---
format: aep.planning-md/2
id: story:typescript-conformance-target
kind: story
status: draft
title: A conformance suite can be emitted as a TypeScript test package
summary: ess verify conform synthesize --target offers ir and go; an adopter in TypeScript reaches no runner
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/specify/ess-domain/src/reading/coordinate.go
- confidence: cited
  path: crates/verify/ess-conformance
revision: 2
---
## What this is

`ess verify conform synthesize --target` offers `ir` and `go`
(`crates/edge/ess-cli/src/main.rs:600-605`). This story adds `ts`: a TypeScript test package emitted
from the same suite document, so an implementation in TypeScript can be held to a specification by
`node --test`.

Sibling of `story:java-conformance-target`, and the same argument applies: the suite is already
language-neutral, so a target re-implements the **runner**, never the suite. Nothing is added to the
model, the scenario vocabulary or the compiler.

## Why this one has a consumer waiting

Java is a language an adopter might arrive in. TypeScript is a language an adopter has already
arrived in and cannot be served.

The shape is common and will recur: a system specified in ESS whose server is Go and whose client
SDK is TypeScript. `--target go` holds the server to the specification. The client — the half an
integrator actually writes against, and frequently the half that is sold — reaches no runner at all,
so its only coverage is a hand-written fake of its own server that nothing checks against that
server. Half such a system can be held to its specification and half cannot, and the half that
cannot is the half facing outward.

One such adopter is waiting on this: 333 synthesized scenarios, a Go server held to all of them
through `--target go`, and a TypeScript SDK held to none. Its design artifact names this story as its
long pole, and it rejected a stdio bridge in order to wait for this rather than fork the evaluator.
That store is private; nothing about it is needed here beyond the fact that the gap is measured
rather than anticipated.

## Measured, 2026-09-19, in this tree

`story:java-conformance-target` carries a measurement from 2026-09-04. It is stale: `runtime.go` has
grown roughly fourfold since. Fresh numbers, and one file that story does not list:

| part | file | bytes | lines |
| --- | --- | --- | --- |
| the emitter | `crates/verify/ess-conformance/src/go/mod.rs` | — | 266 Rust |
| the runner | `src/go/runtime.go` | 210 627 | 6 741 |
| the predicate evaluator | `src/go/predicate.go` | 18 678 | 693 |
| the response decoder | `src/go/response.go` | 10 600 | 374 |
| the reading accessor | `src/go/reading.go` | 7 135 | 217 |
| **the coordinate reader** | `crates/specify/ess-domain/src/reading/coordinate.go` | 6 229 | 186 |

**The sixth row is the one to notice.** It lives outside `src/go/` and is pulled in by
`include_str!` at `src/go/mod.rs:202`, beside the other three at lines 199-201. A port that reads
only the `go/` directory will miss it and fail at the first reading coordinate.

Total hand-written Go to port: 253 269 bytes across five files. The Rust side stays small — the
emitter is 266 lines that assemble static assets and embed `suite.json`, not code emitted as strings,
and `src/ts/` must keep that shape for the same reason `src/go/` has it: every future change to the
runtime would otherwise be a change to Rust.

## What an adopter has today, by language

| language | how it reaches a suite |
| --- | --- |
| Rust | links `ess-conformance` and implements `ConformanceTarget`, `src/target.rs` |
| Go | `--target go` writes `essconform/` — runtime, evaluator, suite, README |
| Java | nothing (`story:java-conformance-target`) |
| TypeScript | nothing |

## Scope

In:

- `--target ts` on `ess verify conform synthesize`, writing an npm-consumable ESM package.
- A `Target` interface in TypeScript carrying the same semantics as the Go one, including the
  `ErrUnsupported` answer, so an implementation that does not expose a semantic is reported
  inconclusive rather than failed. The Go runtime states this at its own `ErrUnsupported`.
- The predicate evaluator, covering every assertion form the suite document can carry, and the
  reading coordinate reader from the sixth row above.
- The report writer: `ESS_REPORT_FORMAT` and `ESS_REPORT_OUT` behave as the Go runtime's do, and a
  filtered run refuses to write a report rather than writing a partial one.
- Byte-identical emission across two runs, as the Go target has.

Out, and deliberately:

- `ess generate synthesize --target ts` — implementation scaffolding rather than a test runner. It is
  the weaker half of the pair and is filed separately if wanted.
- The name `ts` must not collide with the existing `schema typescript` projector
  (`docs/design/types-only-realizations.md:140`), which emits declarations rather than a runner. If
  the collision is judged confusing, `--target typescript` is the fallback; the emitted package is
  the same either way.

## Acceptance

`ess verify conform synthesize --target ts --out <dir>` writes a package that typechecks and runs,
and a deliberately wrong implementation of the billing example fails the emitted suite at the
scenario that names the defect. A second run over the same specification writes identical bytes.
The emitted runtime and the Go runtime, run against the same suite and equivalent targets, report
**the same counts** — total, passed, failed, skipped — which is the check that the two runtimes are
siblings rather than two opinions.
