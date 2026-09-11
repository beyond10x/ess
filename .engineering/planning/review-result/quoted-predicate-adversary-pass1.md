---
format: aep.planning-md/1
id: review-result:quoted-predicate-adversary-pass1
kind: review-result
status: active
title: Check predicate quote admission against canonical round trips
relations:
- reviews: story:refuse-misparsed-predicate-disjunctions
revision: 1
---
unit: gap10 adversary1
verdict: red
cases: 1 new deciding test executed, 0 passed / 1 failed; no previous successful suite rerun
origin: n/a
wrote-outside-worktree: assigned review1 scratch, assigned build-targets/gap10, managed lease/tool caches
needs-coordinator: yes — record immutable finding before implementation correction
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 7af85689c4d3124667cfd1a9dc22e8f74eac2cc28764c25443471227bd738b63, retained as local-evidence:runtime-gaps/publication-replay/snapshots/7af85689c4d3124667cfd1a9dc22e8f74eac2cc28764c25443471227bd738b63.md. Source creation recorded at 2026-09-11T05:18:44Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 2eb2c24659b967733c687a9d300a62ee4df69a9c8c347b6939e5e83d725671ab, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/2eb2c24659b967733c687a9d300a62ee4df69a9c8c347b6939e5e83d725671ab-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

Reviewed candidate at base1fd6ba62c497c1c0fd4354745e2f60e26859c235 in managed wt-25bf1c564b6d. Reviewed all8 reported files, including actual Rust parser/canonical writer, Go original-byte admission and evaluator, browser admission, and each new fixture. No production edits, AEP mutation or Git commit. Added only crates/specify/ess-primitives/tests/adversary_quoted_roundtrip.rs.

P1 — admitted literal data now serializes into bytes refused by the same reader.

Reproduction: construct typed Node from {"to":{"eq":"\"busy\" status"}}. Predicate::from_node admits the structured scalar and the test verifies its exact FactValue::Text is `"busy" status`. Actual Predicate::to_node emits Text("to == \"busy\" status"). Predicate::from_node then fails with “tokens after a quoted operand are unsupported; use structured any/all/not”. The test also contains the equivalent valid single-quoted compact source as a second control, but the first structured case fails before that second iteration executes; no passing claim is made for it.

Cause: new validate_quoted_operand (predicate.rs around1248) tightens leaf admission while Operand::Display (around272) still renders text without dots or empty contents as raw text, and Predicate::to_node (around1180) serializes comparison leaves through Display. Predicate's Serialize implementation uses that same writer. The candidate's structured_composition_and_literal_data_remain_admitted test checks initial admission only and misses emitted bytes. This is a new refusal regression: previously the emitted RHS lacks a matching final quote, so FactValue::parse_literal preserves the entire text verbatim; the added guard now rejects it.

Impact: an otherwise valid structured source predicate can compile/serialize into a generated predicate/suite that Rust refuses to re-admit. The matching new Go/browser quoted guards also reject those emitted compact bytes by inspection; this review did not execute a duplicate Go/Node attack because the actual Rust canonical writer/reader counterexample already decides the acceptance failure. Initial admission parity across all three does not establish validity of generated output.

Smallest correction direction: keep the strict malformed compact-input refusal, and make canonical emission of these otherwise unrepresentable literal values use an existing typed structured comparison form. Do not decode/rewrite the literal bytes, and do not silently reject structured data that the accepted contract preserves. Root must approve the bounded canonical-byte consequence; this does not require a new infix language or arbitrary value model. Add durable roundtrip/generated-suite coverage for the admitted value and retain relevant Rust/Go/browser reader parity. No generic parser redesign is requested.

Executed once:
`cargo test --offline --locked -p ess-primitives --test adversary_quoted_roundtrip`
Brief environment: Rust1.98.1, two jobs, assigned gap10 target, debug/incremental disabled, lld/sccache, encoded rustflags unset.
Result exit101; build0.87s, test0.00s. Exact evidence roundtrip-red.log and deciding-test.patch. Reviewed production patch retained as reviewed-production.patch.

No other confirmed finding. Quote-boundary scanning otherwise matches across the three implementations for the inspected literal cases. Compiler slot1 is released. Own review lease ended at handback.

```findings
- file: crates/specify/ess-primitives/src/predicate.rs
  line: 1248
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: A valid structured quoted literal is serialized into compact bytes rejected by the new reader; use an existing structured comparison when compact spelling cannot preserve meaning.
```