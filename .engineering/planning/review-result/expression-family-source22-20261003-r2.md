---
format: aep.planning-md/3
id: review-result:expression-family-source22-20261003-r2
kind: review-result
status: active
title: Expression-family design independent review pass 2
relations:
- reviews: component-design:expression-family-source22
revision: 1
---
needs-revision

# Family F expression design review — round 2 (publication-safe)

Reviewed `docs/design/expression-family-source22.md` at integration commit
`159a21d7fdfbd99903f738d655e9cf2cda52cb61`; its SHA256 is
`6b869c42cd1e0d90229477c4d86319af633441ac07eaf8967649116b7da9eb23`. This was a static,
read-only review of the whole revised contract and its actual source seams. No compiler, test,
formatter or generator was run, and no implementation or acceptance result is claimed.

Four round-1 blockers are resolved: lexical and resolved predicates are mechanically separate;
UTF-8 byte selectors are tagged while old fields and direct bindings retain their meaning; Integer
offsets use exact wider intermediates with extrema controls; and distinctness persists its key kind
with exact numeric/instant and finite-domain witnesses. The revised contract also preserves old
literal, enum/root, quoted, binder and field meanings, keeps the approved filtered-related-read
snapshot and precedence semantics, and requires actual native, generated Rust/Go and browser lanes.

One A3 blocker remains. The design requires history/2 to record the exact UTC instant used by the
actual command decision, including possible evidence for an indeterminate operation
(`docs/design/expression-family-source22.md:263-276`). The current target and interleaving completion
boundaries return only `SemanticCommandResult` or `TargetError`, and the recorder therefore has no
typed route to receive the evaluator's frozen instant; an error also discards all completion
metadata (`crates/verify/ess-conformance/src/target.rs:181-185,570-614`;
`crates/verify/ess-conformance/src/record.rs:50-80,340-368`). The recorder's monotonic ordering clock
is not UTC and the design correctly forbids using it.

Bind a typed recorded-completion receipt carrying the semantic answer-or-error and the same optional
parsed UTC instant frozen by the command edge. It must survive absent semantic answers, flow through
`Interleaved::complete` into the reserved operation, and never use a second provider or recorder
clock read. Existing clock-free targets need a compatibility default that supplies no time; actual
A3-capable adapters must supply the receipt. Returned, indeterminate, missing-evidence and retained
retry controls must distinguish the true decision receipt from a reread, wall-clock or corrupted
instant. No other blocker was found.

```findings
[
  {
    "file": "docs/design/expression-family-source22.md",
    "line": 271,
    "category": "architecture",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "history/2 requires the recorder to persist the exact UTC instant used by the actual command decision and allows that evidence on an indeterminate operation, but the current ConformanceTarget and Interleaved completion boundaries return only SemanticCommandResult or TargetError and the revised design binds no typed receipt from the evaluator to Operation; define a recorded-completion receipt carrying answer-or-error plus the same optional frozen instant, preserve it across indeterminate completion, provide a no-time compatibility default for existing targets, and add returned/indeterminate/retained-retry controls that detect a reread, recorder-wall or corrupted instant"
  }
]
```
