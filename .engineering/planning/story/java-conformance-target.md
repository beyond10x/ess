---
format: aep.planning-md/1
id: story:java-conformance-target
kind: story
status: draft
title: A conformance suite can be emitted as a Java test package
summary: ess verify conform synthesize --target offers ir and go; an adopter in Java reaches no runner
revision: 1
---
## What this is

`ess verify conform synthesize --target` offers `ir` and `go`
(`crates/edge/ess-cli/src/main.rs:403`). This story adds `java`: a JUnit test package emitted from
the same suite document, so an implementation in Java can be held to a specification by `mvn test`.

## Why the suite is not the work

The suite is already language-neutral. `--target ir` writes the canonical `ess-conformance/3`
document, and the Go emission writes that same document beside the runner as `suite.json`. A target
re-implements the **runner**, never the suite, so this story adds nothing to the model, nothing to
the scenario vocabulary and nothing to the compiler.

Measured against the Go target, verified 2026-09-04 in this tree:

| part | file | size |
| --- | --- | --- |
| the emitter | `crates/verify/ess-conformance/src/go/mod.rs` | 218 lines of Rust |
| the runner | `crates/verify/ess-conformance/src/go/runtime.go` | 52,087 bytes, hand-written Go |
| the predicate evaluator | `crates/verify/ess-conformance/src/go/predicate.go` | 14,399 bytes, hand-written Go |

Both `.go` files are embedded verbatim with `include_str!` at `src/go/mod.rs:56-57` and copied out
unchanged. So a Java target is one emitter module plus roughly 66 KB of hand-written Java, and the
Rust side of it is small.

## What an adopter has today, by language

| language | how it reaches a suite |
| --- | --- |
| Rust | links `ess-conformance` as a crate and implements `pub trait ConformanceTarget`, `crates/verify/ess-conformance/src/target.rs:79`; driven by `runner::run`, `src/runner.rs:286` |
| Go | `--target go` writes `essconform/` — `runtime.go`, `predicate.go`, `suite.go`, `suite.json`, `README.md` |
| Java | nothing |

An adopter in Java can read the JSON and write their own runner, which is the thing this project
exists to stop: a suite no runner can reach is a document, and two hand-written runners that agree
only on the day the second was written are the duplication ESS removes everywhere else.

## Scope

In:

- `--target java` on `ess verify conform synthesize`, writing a Maven- or Gradle-consumable package.
- A `ConformanceTarget` interface in Java carrying the same semantics as the Go one, including the
  `ErrUnsupported` answer — `src/go/runtime.go:100-104` — so an implementation that does not expose
  a semantic is reported inconclusive rather than failed.
- The predicate evaluator, covering the assertion forms the suite document can carry:
  `contains`, `excludes`, `counts`, `ranked`, `at`, `satisfies`, `ExpectNoEvent`, and the `elapsed:`
  windows added in 0.16.0.
- Byte-identical emission across two runs, as the Go target has.

Out, and deliberately:

- `ess generate synthesize --target java` — implementation scaffolding rather than a test runner.
  It is a larger piece of work and it is worth less first: scaffolding nothing can check is the
  weaker half of the pair. File it separately if it is wanted.

## Acceptance

`ess verify conform synthesize --target java --out <dir>` writes a package that compiles, and a
deliberately wrong implementation of the billing example fails the emitted suite at the scenario
that names the defect. A second run over the same specification writes identical bytes.

## Where this came from

Asked for by the operator on 2026-09-04, while settling what `acd/specs` should generate for its
67 authored scenarios.
