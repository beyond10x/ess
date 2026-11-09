---
format: aep.planning-md/3
id: story:a-value-outside-its-declared-type-is-answered-malformed
kind: story
status: draft
title: A read or command sent a value outside its declared type is answered malformed
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#750
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-answer-takes-its-protocols-list-shape
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/malformed.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/malformed_request.rs
- confidence: inferred
  path: docs/design/malformed-requests.md
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 4
---
## Need

A read sent a parameter outside its declared type is refused before it runs: a page size `0` or
`201` against a size newtype bounded `1..200`, or a continuation the target never minted. The same
holds of a command input outside its declared type. The type states the bound
(`example.log.Limit`, invariants `value >= 1`, `value <= 200`, validates on 0.92.0, `run.txt`
step 1), the generated Rust and Go decoders refuse a value that breaks it with `400`
(`story:generated-decoders-enforce-declared-value-constraints`; CHANGELOG 0.91.0, "the generated
Go server's decoder of a newtype enforces what the newtype declares"), and `ess-http/1` already
names the standard refusal `malformed` beside `not_granted` and `unfinished`
(`docs/design/http-presentation-binding.md`, key `refusals.<key>`). But the semantic target
protocol has no `malformed` answer (`crates/verify/ess-conformance/src/target.rs:1259-1290`: only
`Unsupported`, `Unavailable`, `NotGranted`), so no suite sends such a value: the suite of the
reproduction holds no scenario for an out-of-range size (`run.txt` step 5).

## Acceptance

- `TargetError` gains `Malformed { path, constraint }`, the standard refusal of a request whose
  value breaks its declared type, answered before any branch runs and changing nothing, as
  `NotGranted` is answered before the command runs.
- A new step `expect_malformed` follows a `query_view` or an `execute_command` and requires that
  answer; every other assertion finds no declared outcome after it, and an `expect_view` after it
  finds the rows unchanged.
- `ess verify conform synthesize` on the reproduction model writes `example.log.Recent/malformed/limit`,
  which reads the view with `limit: 0` and with `limit: 201` and expects `malformed` for each; on a
  view declaring `cursor: before` it writes `…/malformed/before`, sending a continuation no page
  answered. For a command input of a constrained type it writes `<command>/malformed/<input>` with
  one value just outside each declared bound (the boundary ladder of `conf/witness.rs`), unless a
  declared outcome's guard already claims that value, in which case the outcome scenario stands
  and no malformed scenario is written.
- The suite is written at the new `ess-conformance` pair (the one `story:conformance-follows-a-views-continuation-to-its-last-page`
  opens, if it lands first, else its own); a model with no constrained parameter or input keeps
  its suite bytes.
- `ess verify conform run --target interpreted` passes the scenarios: the interpreted target
  checks each sent value against its declared type and answers `Malformed`. A target that accepts
  `limit: 201` fails `example.log.Recent/malformed/limit`.

## Scope

- cited: crates/verify/ess-conformance/src/target.rs
- inferred: crates/verify/ess-conformance/src/scenario.rs
- inferred: crates/verify/ess-conformance/src/synthesize/malformed.rs
- cited: crates/verify/ess-conformance/src/witness.rs
- cited: crates/verify/ess-conformance/src/interpret/views.rs
- inferred: crates/verify/ess-conformance/src/interpret/execute.rs
- inferred: crates/verify/ess-conformance/tests/malformed_request.rs
- inferred: docs/design/malformed-requests.md
- inferred: website/docs/reference/formats.md
confidence: low

## Fit review

1. **Need.** A value outside its declared type is refused, and the suite checks it, for a view
   parameter and a command input alike. Requester's syntax (theirs): "view query parameters with
   types and bounds, refused with `400`", and "out-of-range `limit` → `400`" among the scenarios.
2. **Class.** Gap. Typed, bounded view parameters exist (`run.txt` step 1); the refusal is enforced
   in generated decoders but has no answer in the target protocol (`target.rs:1259-1290`) and no
   scenario.
3. **Already expressible?** The bound, yes: a newtype with invariants on the parameter. The
   refusal, not for a view: a view has no outcomes. A command can declare a guarded refusal
   outcome, which synthesis already witnesses (`.engineering/planning/story/synthesis-offers-an-input-that-breaks-an-entity-invariant.md`,
   Need), so for commands the malformed scenario is written only where no outcome claims the value.
4. **Fit.** Grid R column "view param" and the input column gain one observation, the standard
   refusal, for every constrained type rather than for paging sizes alone; it mirrors `NotGranted`
   and `expect_not_granted` (`target.rs`, `NotGranted` doc comment) and the binding's existing
   `refusals.malformed`. The HTTP binding maps it to its declared `malformed` status, so the `400`
   is the binding's. Generated Go and TypeScript runtimes:
   `story:the-go-and-typescript-runtimes-follow-continuations-and-check-malformed-answers`. Entity
   Runtime lowering already enforces input type invariants or refuses them by name
   (`story:entity-runtime-lowering-enforces-input-type-invariants`).
5. **Second adopter.** A search view with `q` of a String newtype `length: {max: 100}`; a payment
   command whose `currency` newtype has a three-letter alphabet.
6. **Cost.** A suite format pair; one `TargetError` variant (additive for a Rust test adapter, but
   a target that never answers it fails the new scenarios); one step; a design page. No source
   format.
7. **Alternatives.** (a) Change nothing: the bound is enforced by generated code and checked by
   nobody. (b) A view-only `refusals:` key on views: refused, a second refusal mechanism beside the
   binding's. (c) The standard `malformed` refusal in the target protocol, for reads and commands:
   chosen.

## Decisions

accept, redesigned. The request's "refused with 400" becomes the semantic standard refusal
`malformed` for every request value outside its declared type, on reads and commands, which the
HTTP binding maps to its declared status. A design page states it first.

## Source

github:beyond10x/ess#750
