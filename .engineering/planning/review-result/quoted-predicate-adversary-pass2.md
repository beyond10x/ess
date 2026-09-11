---
format: aep.planning-md/1
id: review-result:quoted-predicate-adversary-pass2
kind: review-result
status: active
title: Review lossless predicate reader correction
relations:
- reviews: story:refuse-misparsed-predicate-disjunctions
revision: 1
---
unit: story:refuse-misparsed-predicate-disjunctions — correction review2
verdict: red
cases: one new deciding case executed, one failed; existing green cases not rerun
origin: introduced structured-operand regression; related compact-parser behavior pre-existing
wrote-outside-worktree: local-evidence/runtime-gaps/gap10/review2; Go tool cache; own managed review lease
needs-coordinator: yes — record this report before correction
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 4b5280a49df6e428255bd077ad1e7443cddf706ef4a07e903b172fd4ca378d9b, retained as local-evidence:runtime-gaps/publication-replay/snapshots/4b5280a49df6e428255bd077ad1e7443cddf706ef4a07e903b172fd4ca378d9b.md. Source creation recorded at 2026-09-11T05:53:55Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 a0686c12cea14cdfeb020ea8d6cb270cf319df07cf8ed34d133ab085db4dfe22, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/a0686c12cea14cdfeb020ea8d6cb270cf319df07cf8ed34d133ab085db4dfe22-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

Review changes: one new test-only fixture, crates/verify/ess-conformance/tests/fixtures/adversary-quoted-specials.go. The six inherited dirty central-routing files were present before this review and remained untouched. No production, Git or AEP writes.

Candidate source-only commit09a16599b1e85532869fbcb837dd37cb9c5f9ad6 plus retained temporary routing, as identified by format-candidate-sha256.txt. Source hashes for the exact Go helper bytes exercised are retained in probe-source.sha256. The test package concatenates current runtime.go, reading.go and coordinate.go exactly in emitter order, copies predicate.go unchanged, and supplies an unused suiteJSON constant; it does not claim a new actual Rust emitter execution.

| Finding | Verdict | Origin | Concrete consequence |
|---|---|---|---|
| F1: Structured valid text NaN becomes a non-finite number in the corrected Go reader | CONFIRMED | introduced on structured comparison path | predicate.go:311-312 now passes every structured string to parseOperand; parseLiteral:400-401 accepts strconv.ParseFloat NaN. Rust facts.rs:243-244 rejects non-finite numbers, keeping the same text. Matching text facts now evaluate false, and Go's old-version guard refuses an operand Rust preserves unchanged. |

What was measured: TestStructuredSpecialTextPreservesRustLiteral first proves the ordinary Ready scalar control, then parses original JSON `{"to":{"eq":"NaN"}}`. It observes float64 NaN instead of text, an old-header refusal from admitPredicateVersion(...,7), and false evaluation against actual fact `to = "NaN"`. Exit1, one executed failing test,0.001s. This probe invokes the real structured parser and version guard, not a reimplementation.

What reaches it: persisted ExpectView/EventuallyView Satisfies predicates call admission admitPredicateVersion and evaluator parsePredicate → parseConstraint. Structured comparison scalar strings are admitted grammar, including NaN as ordinary text in Rust. Suite8/9 accepts this grammar but the changed evaluator misinterprets it; older Go admission now disagrees with Rust. No particular adopter occurrence is claimed. The Rust canonical spelling inferred directly from current writer/parser is `"to == NaN"`: Operand display emits the undotted nonempty text, and Rust parse_decimal keeps NaN as text. This canonical spelling was source-checked, not newly emitted by a Rust command in this review.

Origin detail: the base parseConstraint at1fd6ba62 stored a structured string directly as literal, so this exact structured-input regression is introduced by the correction. Go parseLiteral's permissive special-number behavior and its use by the historical compact form predate this unit; do not claim the whole compact-reader defect was newly introduced.

Smallest correction: align Go parseLiteral's numeric acceptance with Rust's finite decimal syntax, preserving text for non-finite and non-decimal special spellings. The same helper must drive parseOperand and old-header detection so the version guard does not reject unchanged text. Add boundary coverage for NaN, infinities, and Go-only hexadecimal float syntax, with finite decimal and ordinary text controls. Do not paper over this by blindly quoting every value or widening the canonical format detector.

Other bounded review observations: the original independent quoted-roundtrip regression is unchanged; the compact writer falls back only after actual Rust parse-equality, preserving legacy bytes when possible. Typed and original-byte version traversal cover nested Boolean/quantified predicates and admitted selection first predicates. No Unicode formatting approximation appears in the detector. These are source observations, not a claim that the full change is correct.

Command: GOMAXPROCS=2 go test -C local-evidence/runtime-gaps/gap10/review2/go-probe -run TestStructuredSpecialTextPreservesRustLiteral -count=1 -v

```text
=== RUN   TestStructuredSpecialTextPreservesRustLiteral
    probe_test.go:10: structured scalar changed: got NaN (float64), want text NaN
    probe_test.go:13: old-header disagreement: Rust preserves text NaN, Go refuses: normalized structured comparison operands require suite/8 or /9
    probe_test.go:14: matching text failed: 1
--- FAIL: TestStructuredSpecialTextPreservesRustLiteral (0.00s)
FAIL
exit status 1
FAIL	example.invalid/predicate-review	0.001s
```

Private full-path inventory: private-inventory.txt. Public aliases above deliberately replace local paths; source hash manifest and original test output are unchanged. Compiler slot released after terminal result. No suite/full-gate or unchanged successful rerun.

```findings
- file: crates/verify/ess-conformance/src/go/predicate.go
  line: 311
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Structured text NaN becomes a nonfinite Go number and the original-byte version guard disagrees with Rust literal preservation.
```