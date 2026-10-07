---
format: aep.planning-md/3
id: review-result:aggregate-causal-design-20261003-r1
kind: review-result
status: active
title: Aggregate causal completion design review, first pass
relations:
- reviews: component-design:aggregate-observation-integration-boundary
- reviews: dependency-blocker:aggregate-binding-cut-authority
revision: 1
---
needs-revision

## Findings

1. `docs/design/binding-causal-observation.md:33-38` proposes a separate optional Rust `CausalBindingTarget`, but it does not bind how the existing runner discovers or invokes it. `Runner` is generic only over `T: ConformanceTarget` (`crates/verify/ess-conformance/src/runner.rs:346-350`, `:409-414`, and `:522-527`), so it cannot conditionally call an unrelated trait on stable Rust. Adding a second generic bound would require every adapter to implement the capability and conflicts with the promised no-capability path when relevance is empty. Bind a backward-compatible seam, such as default causal methods on `ConformanceTarget` or a default optional trait-object accessor, and specify the scenario lifecycle that opens, executes through, queries, and closes it. Preserve the required proof that unsupported old adapters never execute an operation twice (`docs/design/binding-causal-observation.md:185-188`).

2. The design assigns receipt authority to the real generated dispatcher/store (`docs/design/binding-causal-observation.md:95-102`, `:109-123`, and `:165-173`), but `.engineering/planning/component-design/aggregate-observation-integration-boundary.md:44-54` omits the Rust/Go system generators, generated stores, and capability adapter from its ownership inventory. The current Rust system has only invocation/publication logs, a cursor, and held events (`crates/generate/ess-synth/src/rust/system.rs:740-769`, `:880-904`), while its dispatch has no causal identities or terminal/commit bookkeeping (`:1193-1232`). The generated Rust and Go stores mutate/read rows without commit positions or retained snapshot receipts (`crates/generate/ess-synth/src/rust/store.rs:34-45`; `crates/generate/ess-synth/src/go/store.rs:36-70`). Add these producer and exposure surfaces to the design and bind where every receipt field is recorded, how the aligned immutable query cut is captured, and how the capability reaches native, generated-service, WASM, and full-browser execution. Otherwise the listed conformance files could only reconstruct evidence after the fact, which the proposal forbids at `docs/design/binding-causal-observation.md:125-127`.

```json
[
  {
    "file": "docs/design/binding-causal-observation.md",
    "line": 33,
    "category": "interface",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the proposed separate optional Rust CausalBindingTarget has no discovery/invocation seam from Runner's existing T: ConformanceTarget API; bind a backward-compatible default or trait-object accessor and its exact scenario lifecycle so old adapters remain supported and an observed operation cannot execute twice"
  },
  {
    "file": ".engineering/planning/component-design/aggregate-observation-integration-boundary.md",
    "line": 44,
    "category": "architecture",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the ownership inventory omits the Rust/Go generated dispatcher, store, and adapter surfaces that must produce occurrence/attempt/commit receipts and an aligned immutable row snapshot; bind those exact producer and exposure seams for native, generated-service, WASM, and browser acceptance so evidence cannot be reconstructed from post hoc logs"
  }
]
```

## Assessment

The causal model is otherwise conservative enough for #361/#362: it closes transitive relevant binding paths, treats unknown relevance and remote effects as unresolved, prevents retry completion gaps, aligns inventory and rows to one immutable cut, preserves source authority, isolates correlations, and refuses unsupported ordering/snapshot domains. Its suite/38 and inventory/39 admission rules preserve compatibility, and its private-state rules plus failure-path control state the necessary privacy outcome. The eventual API must carry stable closed reason codes rather than arbitrary adapter detail through the existing report path.

The native/generated/browser acceptance matrix is fault-sensitive and adequate after these two interface and ownership gaps are corrected. This review does not clear `dependency-blocker:aggregate-binding-cut-authority`; that still requires actual independent adapter execution.
