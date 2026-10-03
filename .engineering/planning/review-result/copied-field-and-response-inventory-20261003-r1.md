---
format: aep.planning-md/3
id: review-result:copied-field-and-response-inventory-20261003-r1
kind: review-result
status: active
title: Independent review of copied-field execution and parser inventory controls
relations:
- reviews: story:feature-request-307
- reviews: story:feature-request-312
revision: 1
---
approve

# Independent review: two remaining full-run controls, pass 1 of 2

Candidate `37f600cdf579d462c0009c17aa76e7d51ce72355` correctly updates the two stale integration
controls. This verdict applies only to the two-file correction and its exact 11 tests.

## Findings

```json
{
  "findings": []
}
```

The patch changes only `subject_guard_copied_field.rs` and `underscore_field_names.rs`; its SHA-256
is `fbe9afaa4c2ed770d10ddef34a2f7323cb1ce1e2429ce2194f7fd626aba4801e`. Source hashes match the
frozen handoff.

The native-interpreter control now requires the exact promoted, rolled-back, and finished outcome
scenarios to be present and individually Passed, and it rejects any non-passing scenario in the
whole report. The honest target, `CopiesNothing`, Optional true/false/absent witnesses, and all nine
copy/branch/state mutant controls remain unchanged.

The field-name audit remains an exact equality. Independent inventory found 13 current field-name
patterns plus 4 current fact-path patterns, exactly 17. The two added entries are the Go and
TypeScript nested-response parsers, where the pattern validates roots, declarations, mapping
sources, and target segments. Independent stale scanning found zero old field-name patterns and zero
old fact-path patterns.

Independent execution of the exact supplied binaries passed 8/8 and 3/3 tests, with zero failed,
ignored, measured, or filtered cases; both binaries exited 0. The reviewer performed no fresh
compilation. The raw execution log has SHA-256
`fe8b7fe8c0296367a8b904c00f26c841d9c60b1636a2050dfbbd088029bfdc57`.
Author evidence also records these exact two targets, strict scoped lint, exact-file formatting, and
diff checks as green.
