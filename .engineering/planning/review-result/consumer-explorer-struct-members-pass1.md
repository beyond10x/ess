---
format: aep.planning-md/3
id: review-result:consumer-explorer-struct-members-pass1
kind: review-result
status: active
title: Explorer typed struct comparison admits extra target members
relations:
- reviews: story:feature-request-293
revision: 1
---
needs-revision

Root reviewed the in-progress #293 typed comparison in both explorer assets and executed an independent faulty target against an actual emitted TypeScript matrix package. This is a bounded review before source freeze, not the final full unit review. No compilation was needed, and no worker files were changed by the root probe.

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/ts/explore.ts",
    "line": 715,
    "category": "typed-struct-comparison",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "valueAgrees recurses through declared struct fields but never rejects undeclared keys in the actual struct. The Go exploreValueAgrees has the same omission. The previous exact equality did not permit this extra content. An actual emitted matrix target that copies its input then adds payload.box.undeclared='fabricated' executes all8steps and reaches both outcomes without a finding, just like the healthy control. Preserve closed actual-member comparison while normalizing only source-admitted Optional absence/presence, and add the same extra-member mutant to both emitted runtime controls."
  }
]
```

Actual output SHA256 3e71e929fe83583addcd9419bf9ab132d57090f0dbd4af57bb7297f4b0616340, private ess-293-root-review-20261003/extra-struct-member.log. Exact tested emitted dist/explore.js SHA256 2227e790f3abb544b5ce09d3301d053e7e613118a2e3be6ece46bcd170daeb9d, from retained ess-optional-unknown-3797665-ts-matrix. Both runs use seeds1/steps8, actual independent callbacks, no suite oracle: the healthy target echoes actual input, the mutant changes only an additional nested key. Both outputs report8executed, no exclusions/ambiguity/undetermined values and no failure. The owner confirmed the missing key-set check and is correcting both assets plus the paired mutant fixture. Correction and final emitted reruns remain pending, subject to the existing disk-start floor; this record does not claim a fix is green.
