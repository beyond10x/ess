---
format: aep.planning-md/3
id: review-result:nested-increment-implementation-20261003-r1
kind: review-result
status: active
title: Nested increment first full independent implementation review
relations:
- reviews: story:nested-increment-previous-location
revision: 1
---
needs-revision

## Test-only diff

The prospective review patch changes only `crates/generate/ess-synth/tests/declared_behaviour.rs` (208 insertions, no deletions; diff SHA256 `7fb1ef8faf131445af096b85dd627b4c9a74f9b2201129b9ac2161091cea39f1`). It adds actual generated Rust/Go compile controls for the failing optional name-collision case and passing required-parent and creation cases. No production or author test file was edited.

## Findings

1. `crates/generate/ess-synth/src/go/behaviour.rs:2371` and `crates/generate/ess-synth/src/rust/behaviour.rs:1670` eagerly apply optional-parent previous-value handling to every recursive struct mapping. An update with a same-named optional stored field and output-only event field reaches the defect even though event payloads cannot contain increment. Generated Go fails `go build ./...` with `undefined: before`; generated Rust fails its warnings-denied build because the newly registered `undeclared` helper is unused. Required-parent and creation payload controls build in both languages.

2. `crates/verify/ess-conformance/src/ts/runtime.test.ts:377` uses a parameter property forbidden by the repository's `erasableSyntaxOnly` configuration. The configured, non-skipped repository TypeScript typecheck exits 101 with TS1294. All 239 runtime cases execute and pass, so the failure is isolated to the required compilation gate.

```findings
[
  {
    "file": "crates/generate/ess-synth/src/go/behaviour.rs",
    "line": 2371,
    "category": "correctness",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "recursive optional-struct previous-value handling runs for an output-only event payload that consumes no previous state; a same-named optional stored field makes generated Go reference an undefined `before` and makes generated Rust register an unused `undeclared` helper, so both generated targets fail their actual compile gates"
  },
  {
    "file": "crates/verify/ess-conformance/src/ts/runtime.test.ts",
    "line": 377,
    "category": "build",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the new `NestedIncrementTarget` parameter property is forbidden by the repository's `erasableSyntaxOnly` configuration; the non-skipped repository TypeScript typecheck exits 101 with TS1294"
  }
]
```

## Evidence

Fresh focused execution passed 8 domain tests, 1 compiler ancestry test, and 74 native/history/synthesis tests. The required and creation generated-payload controls compiled in Rust and Go. The prospective optional name-collision test independently failed actual generated Rust and Go compilation. TypeScript executed all 239 runtime cases, while the separately configured source typecheck failed without a skip. Scoped strict Rust lint, scoped formatting, and whitespace checks passed.

The source/caller inspection found no second ancestry representation: validation preserves root and leaf authority, native execution keeps target location separate from history provenance, synthesis walks the complete location, and recursive generated `sets:` paths use that same ancestry. The failure arises because the generated payload caller shares the recursive renderer and eagerly materializes optional previous-value machinery before a descendant asks for it.

The author-reported broad generated matrix was not repeated after the candidate was confirmed red. Full release checks and live browser ordinary/coverage acceptance remain outside this review.
