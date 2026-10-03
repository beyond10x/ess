---
format: aep.planning-md/3
id: review-result:consumer-nested-response-design-pass1
kind: review-result
status: active
title: 'Nested response design review: structural grammar and byte profile'
relations:
- reviews: story:nested-response-observations
revision: 1
---
needs-revision

Supplemental AEP-schema rendering of the original independent design review, preserved unchanged with SHA256 `a3fa81cf508f1c09445629c05ff39e7efa19a9e396b9f987fd4cb9ea8d69cfe0`. This is a formatting supplement, not a repeated review.

Reviewed source carrier: `046db8a6805154aa5cafc63b0a4b741bc26e954d`. Reviewed proposal SHA256: `327839ef111a6a7ee7f4f94e8ea65ba5ea8baa305c5086dd19b9f6cde6b27474`. Reviewed design candidate `docs/design/nested-response-observations.md` SHA256: `cc0c1a36ccb169c23c91fec499c0aaa9d5b0689638493b3d74c799c26f270278`. Reviewer test executions: 0; no builds or production edits. Verdict concerns design readiness only.

```findings
[
  {
    "file": "docs/design/nested-response-observations.md",
    "line": 15,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "NRO-D1: Define a closed structural name/type field grammar and reject naming/presence metadata before projection. Reusing Rust Field admits source metadata that generated Go/TypeScript response field readers reject; producer omission does not establish consistent original-byte admission. Keep historical response-value presence authority separate and compare overlap by representation only."
  },
  {
    "file": "docs/design/nested-response-observations.md",
    "line": 47,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "NRO-D2: Define one common whole-observation byte counting profile for nested authority. Native tagged-declaration serialization differs from Go's non-omitted zero declaration fields and TypeScript's Go-shaped marshalShape, so merely retaining each existing 1 MiB check yields different nested admission boundaries. Specify encoding, omission and escaping rules, test exact limits across runtimes, and preserve historical nested-less behavior."
  }
]
```

The original report retains the full source citations and rationale. These are bounded design corrections; Binary64 execution, a new suite pair, interpreter edits, and changes to legacy response semantics are not requested.
