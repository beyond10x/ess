---
format: aep.planning-md/3
id: review-result:consumer-wrapped-invariant-design-v2
kind: review-result
status: active
title: 'Wrapped invariant design review: portable budget and incomplete inventory gaps'
relations:
- reviews: story:optional-value-invariant-observation
revision: 1
---
needs-revision

Independent source-only design review of frozen wrapped-value-invariant-design-v2.md SHA256 ca56df7e4687bd8e4402afe8380cff3c647a91b5c785e355652ac13d128c63c3 against runtime c2c4f01c6cfe99c6a16db5670669fb9774ab6bb9 and browser source checkpoint 1e1b97f3b240f862edc741d89d374300895e0451a1953967890a30b89d7291ab. Reviewer recover_caller executed no compiler, test, browser or target and changed no repository source. Root read the complete private report wrapped-value-invariant-design-review-caller-v2.md SHA256 3a340e254dd47c78d0673817d33c2ec4304a11b01b42f07d2b8a76d6dedd05a9. These are gaps in an unimplemented design, not measured failures of a shipped observer.

Three corrections are required before adoption: define semantic canonical numeric work charges rather than unavailable original tokens; define one logical charging schedule independent of physical traversal passes; prevent incomplete inventory loss through generic serialization and every producer reconstruction. The finite recursive graph, occurrence witness forms and lexical typed-fact direction had no further concrete contradiction in this bounded review. That assessment does not prove actual arrangement, resource or runtime acceptance.

The browser integration boundary is now explicit: web_execution/bundle.rs admits selected ordinary/coverage input and parents, recompiles sources and compares provenance. It does not authenticate every expectation declaration closure against the IR. Shared Runner and generic typed presentation need no new ABI opcode or JavaScript expectation evaluator. Reuse the actual emitted host and independent Installation in the existing browser_response_conformance.rs and fixtures/browser-target/src/lib.rs; a separate test binary would require a scoped helper extraction. Strengthening model-aware closure validation, if intended, must be an explicit additional design choice with a forged-closure regression. No format reservation or implementation authority follows from this review.

```findings
[
{"file":"wrapped-value-invariant-design-v2.md","line":342,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"WVI-D2-1: Numeric-token work charges are not portable. Native Number discards original lexical spelling and exact_text normalizes it, while Go json.Number and TS JsonNumber retain raw spelling. Define a typed canonical semantic numeric work codec independently of transport bytes and historical serialization. Pin equivalent spelling and exact-value N-1/N/N+1 controls. Source: facts.rs175-208,269; go/runtime.go1134,3208-3223; ts/runtime.ts134-150."},
{"file":"wrapped-value-invariant-design-v2.md","line":334,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"WVI-D2-2: Per-pass work debit depends on implementation organization. Fused versus separate shape/site/fact traversals can charge different totals. Define canonical logical event order and multiplicity independent of physical fusion, caching and rescans, including scalar parse/comparison charges and fact reuse across invariants. Require identical debit traces and boundary results across runtimes."},
{"file":"wrapped-value-invariant-design-v2.md","line":371,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"WVI-D2-3: A merely nonserialized incomplete-inventory marker can be lost through existing generic Serialize/Deserialize round trips. Require Serialize itself to refuse incomplete suites, including to_value, and audit every field-copy/reconstruction and checked coverage Document writer. Test clone/scoping, canonical/compact/pretty writers, emitters, browser bundles and round-trip attempts while preserving complete historical bytes. Source: scenario.rs101-117,242-263; admission.rs118-156; coverage.rs889-925; coverage_build.rs144-179."}
]
```
