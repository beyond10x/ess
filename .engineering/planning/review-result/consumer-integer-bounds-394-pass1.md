---
format: aep.planning-md/3
id: review-result:consumer-integer-bounds-394-pass1
kind: review-result
status: active
title: Integer bounds and native width first independent review
relations:
- reviews: story:feature-request-394
revision: 1
---
needs-revision

```json
[
  {
    "file": "crates/generate/ess-gen/src/types.rs",
    "line": 917,
    "category": "constraint-composition",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "independent-source-review",
    "message": "Required Integer equality invariants overwrite the existing const instead of intersecting. A struct with invariants version == 1 and version == 2 is admitted, but its emitted schema ends with const: 2 and accepts version: 2. Preserve contradictory equality as an impossible schema/range or report the unlowered constraint explicitly; do not claim both recognized equalities were lowered while silently retaining only the last one."
  }
]
```

Reviewed exact source commit 0cb925e997e19ce6c2cedeea05d1a546db0fd211 versus 1ff305685, all ten changed files, read-only in external tree ess-394-integer-bounds (HEAD4ab8c3aab includes planning follow-up). Reviewer executions: 0. No edits, build, external message or source-ownership takeover.

Reproduction to run in the integration tree: take tests/integer_bounds.rs MODEL, replace its single `version == 2` with two invariants `version == 1` and `version == 2`, generate the schema and validate the test's otherwise-valid object with version2. The current projection's final assignment emits const2, so a plain JSON Schema validator accepts it. Invariant contradiction is not rejected at compile time: compiler resolve.rs design-enforcement table explicitly records that contradictory invariants are checked nowhere. Reversing invariant order changes the spuriously accepted value. Existing minimum/maximum composition already retains contradictions, so equality should compose consistently. A simple remedy is to accumulate equality as both bounds as well as a const, preserving earlier restrictions; test both declaration orders and equality-versus-range contradictions.

Other source findings: nullable equality uses numeric min/max siblings rather than const, correctly leaving null/absence unaffected by numeric keywords. Strict comparisons use checked +/-1, so boundary overflow does not wrap; unrepresentable strict bounds remain only invariant obligations. Lower/upper bounds choose the tighter bound. Width inference is confined to sealed model input, with both representable bounds or a representable integer const required; bundle input follows its old Shape::Integer path. i32 removes TypeScript's precision obligation, i64 retains it; integer/range/const validation obligations remain explicit. Native mappings choose signed fixed-width Rust/Go types, and normalization preserves Integer semantics at its existing checking seam.

Limits and required integration validation: new tests inspect generated native declaration text but do not compile/run the bounded codecs or exercise contradictory equality. This reviewer does not claim runtime verification. Run the targeted ess-gen/schema-contract tests after correction, plus actual Rust/Go decode/encode boundary controls, optional/null handling, i32/i64 cutovers, integer const enforcement through declared schema validation, and unchanged bundle realization controls. Native fixed-width decoders can be stricter about integral JSON spellings such as 1.0 than JSON Schema integer; the types-only contract explicitly disclaims decoder parity, so this review records that representational limit rather than inventing a broader parity claim. The refrozen binary64_structural digest is explained by the newly emitted minimum on score but still needs execution against integrated bytes.
