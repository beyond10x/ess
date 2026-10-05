---
format: aep.planning-md/3
id: story:feature-request-427
kind: story
status: active
title: 'Conformance: an authored scenario cannot assert how many times an event is published'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#427
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/count_json.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/event_multiplicity.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/event-multiplicity.yaml
- confidence: inferred
  path: website/docs/reference/formats.md
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:21Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:22Z", actor: "human:timo", revision: 11}
---
## Outcome
Resolve beyond10x/ess#427: Conformance: an authored scenario cannot assert how many times an event is published.

## Origin
beyond10x/ess#427, filed 2026-10-05; a downstream specification (ess/20, ess 0.52.0) whose batch command publishes one event per record it changes, guarded outside the suite with an `ESS-LIMIT` marker.

## Fit review
1. Need: one command answer publishes the same declared event N times, and the suite must fail a target that publishes it fewer times. Today N identical claims are met by a single occurrence. Rust `expect_event` picks the first occurrence by name and never consumes it (`crates/verify/ess-conformance/src/runner.rs:1957-1962`). Go does the same (`src/go/runtime.go:2450-2465`, "the first occurrence ... as ess_conformance::runner's expect_event selects it"). The defect reaches synthesized suites as well as authored ones. Minimal reproduction `<fit-review scratch>/probe-427/` (fresh, brand-free) is an outcome with `emits: [demo.batch.Rewrapped, demo.batch.Rewrapped, demo.batch.Rewrapped]`. On installed ess 0.52.0 it validates (`validate.out`: "demo v1 — 2 file(s), valid"), and synthesis writes three identical `expect_event` steps (`suite.json`, `ess-conformance/4`). One published occurrence satisfies all three. Requester's proposals, theirs: multiset matching of the listed events, or `count: 3` on an event entry, or `events_exactly:`.
2. Class: defect. The model runner's documented contract is "Expected direct events are the outcome's `emits`, in order" and "Direct event names, in order" (`docs/design/mutation-audit-and-model-runner.md:560,568-569`). The suite runners drop that multiplicity. There is also a cross-runner disagreement. TypeScript searches every occurrence for one whose payload matches (`src/ts/runtime.ts:3928-3955`), while Rust and Go check only the first. So two distinct-payload occurrences of one event pass in TypeScript and fail in Rust/Go. Noted here, not filed separately.
3. Existing idiom: none. `expect_no_event`/`no_events:` assert absence only (`runner.rs:619`, `src/authored.rs:2457-2465`). A view `counts` expectation counts rows, not occurrences. Repeating the claim is already the natural spelling (`authored.rs:2454-2456` lowers each `events:` entry to its own `ExpectEvent`). It just means nothing today.
4. Fit: no new key. The step's meaning becomes: "after one command, each `expect_event` for event E claims a distinct unclaimed occurrence of E". Each claim takes the first unclaimed occurrence that carries its values. If none does, the first unclaimed occurrence is reported as before (`ESS-CF-PAYLOAD`). If none is left, it fails `ESS-CF-EVENT` and names how many were published. This keeps the runner's "wrong event vs wrong value" split (`runner.rs:1954-1956`). It also matches TypeScript's payload search, so the three runners converge. The rule applies the same way to an authored `events:` list and to a synthesized `emits:` list. `expect_event_values` (`runner.rs:1918-1940`) shares `expect_event`, so it gets the rule too. Later-observed `eventually_event` is out of scope: it reads the whole log, not the last command (`src/go/runtime.go:2515`). Every runner (Rust, Go, TypeScript) implements the rule. A reader older than the new format refuses the suite rather than ignoring repeated claims.
5. Second adopter: a bulk-close command emits `TicketClosed` once per ticket it moves. A suite claiming two closures must fail a target that closes one ticket and publishes once.
6. Cost: a suite-format pair. A suite whose single act claims one event more than once is written at `ess-conformance/44`/`/45`. That is the even/odd coverage pair (`src/admission.rs:209,267-272`), chosen by `select_fresh_format` (`src/scenario.rs:157`). Every other suite keeps its bytes, and readers through `/43` keep first-match semantics for older suites. A 0.53-era runner would silently under-check a regenerated suite, so it must refuse `/44` instead. It does today: `NEWEST_ADMITTED_SUITE_MAJOR = 43` (`src/go/mod.rs:370`). No source format, keyword or diagnostic code is added; `ESS-CF-EVENT` gains a count in its observed text.
7. Alternatives: (a) change nothing and keep `ESS-LIMIT` markers. That leaves synthesized `emits` multiplicity unchecked, which is the defect. (b) `count:` / `events_exactly:`, the requester's. That is a new authored key, an `ess-scenario` bump and two spellings of "three occurrences", which is a red flag. (c) Chosen: consuming semantics behind one suite-format pair. It adds no surface and fixes synthesized suites too. A strict positional rule (k-th claim against the k-th occurrence) was set aside because authored claims are written by hand and need not follow publication order.

## Decisions
accept, redesigned — repeated claims are counted, so no `count:` or `events_exactly:` key is needed. Each `expect_event` after one command claims a distinct occurrence, in the Rust, Go and TypeScript runners alike. Suites with a repeated claim in one act are written at `ess-conformance/44`/`/45`; older suites and readers keep their meaning. Format bump: `ess-conformance/44` and `/45`. Every other 0.54 suite-format change shares this pair: #438 and #458 depend on this story (edges recorded) and use `/44`/`/45` if they need a suite change. No `ess/23`.

## Acceptance
- repeated_emit_requires_each_occurrence: the committed fixture `crates/verify/ess-conformance/tests/fixtures/event-multiplicity.yaml` (the fit-review model: `demo.batch.Rewrap` emits `demo.batch.Rewrapped` three times) synthesizes a suite that fails a target publishing one `Rewrapped` and passes one publishing three.
- authored_repeated_event_claims_count: an authored act listing one event three times fails against one occurrence, with `ESS-CF-EVENT` naming 1 published of 3 claimed.
- distinct_payload_occurrences_agree_across_runners: two occurrences of one event with different payloads, claimed in either order, give the same verdict in Rust, Go and TypeScript.
- payload_mismatch_still_reports_payload: a single wrong-valued occurrence still fails `ESS-CF-PAYLOAD`, not `ESS-CF-EVENT`.
- single_claim_suites_keep_their_bytes: committed suites under `suites/generated/` and every fixture without a repeated claim are byte-identical, at their previous format.
- older_reader_refuses_repeated_claim_suite: a `/44` suite is refused by a reader admitting through `/43`; a `/4` suite with repeated claims keeps first-match behaviour.
- emit_count_mutant_fails: an implementation dropping one of three emissions is killed by the synthesized suite.

## Scope
- crates/verify/ess-conformance/src/runner.rs  cited — `expect_event` first-match at :1957-1962
- crates/verify/ess-conformance/src/go/runtime.go  cited — `expectEvent` first-match at :2450-2465
- crates/verify/ess-conformance/src/ts/runtime.ts  cited — `expectEvent` payload search at :3928-3955
- crates/verify/ess-conformance/src/go/mod.rs  cited — newest admitted major at :370
- crates/verify/ess-conformance/src/admission.rs  cited — suite-major admission at :209,:267-272
- crates/verify/ess-conformance/src/scenario.rs  cited — `SUPPORTED_SUITE_FORMATS` :445-448, `select_fresh_format` :157
- crates/verify/ess-conformance/src/count_json.rs  cited — suite-format list at :115
- crates/verify/ess-conformance/src/authored.rs  cited — `events:` lowering at :2454-2456, :3080-3110
- crates/verify/ess-conformance/src/synthesize.rs  inferred — `expect_event_step` (:6457) emits one step per `emits` entry
- crates/verify/ess-conformance/tests/event_multiplicity.rs  inferred — new regression test
- website/docs/reference/formats.md  inferred — the `/44`/`/45` row
- crates/edge/ess-xtask/src/docs.rs  inferred — `FORMAT_RELEASES` rows for `/44` and `/45` (the `ess-conformance` list it reads, :79-83)
- website/docs/reference/spec-versions.md  inferred — the `/44`/`/45` paragraph beside the `/42`/`/43` one (:421)
- crates/verify/ess-conformance/tests/fixtures/event-multiplicity.yaml  inferred — the fit-review model, committed
