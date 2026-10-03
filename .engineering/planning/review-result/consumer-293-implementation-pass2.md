---
format: aep.planning-md/3
id: review-result:consumer-293-implementation-pass2
kind: review-result
status: active
title: Optional and missing-instance explorer implementation review
relations:
- reviews: story:feature-request-293
revision: 1
---
approve

Root independently reviewed the frozen nine-file patch over 6727e07363877925aa6a0f2bb1cb47b08d2348e3, SHA256 93245998c5d554653c53ef40b6c11c0f98fb7b536ad7f2189010d7e6b09b2304. The current working diff reproduces that hash. The implementor authored the changes; root reviewed both complete production diffs and the Rust-driven tests and independent target fixtures. No new compiler or runtime execution was started during this final review because release verification has priority.

The production changes keep typed absence separate from unavailable model knowledge, preserve field presence policy and closed struct members, draw known/fresh subject identities with bounded retries, resolve missing-instance outcomes after the source-selected branch, and retain the external arrangement separately from the expected terminal outcome. Replay retains reference provenance and checks intentional freshness; shrinking preserves the affected command, arrangement and failure. Concurrent recording continues to use the native history checker rather than introducing a local verdict engine.

Root verified the retained evidence manifest (all listed hashes matched), final serial 14/0, concurrent 2/0 with 40 native-judged histories, strict TypeScript, repository formatting and strict Clippy terminal receipts. Earlier neighbor receipts cover 30 serial and 22 concurrent tests; they precede the narrow member-closure correction and are not relabeled as final-source executions. The final dedicated matrix includes that correction. The focused fixtures execute actual Go/TypeScript packages and independent targets; the native process boundary comes from CARGO_BIN_EXE_ess. Healthy callbacks and negative controls prevent exclusion-only green claims.

The preceding review found a real false green: the typed struct comparison accepted undeclared actual members. Both assets now reject them. Root's independent emitted-TypeScript probe first accepted the extra-member mutant and then rejected it after correction, while retaining the healthy control. Red log SHA256 3e71e929fe83583addcd9419bf9ab132d57090f0dbd4af57bb7297f4b0616340; corrected log SHA256 e4dc215158f3087e8c4551e260b9879c2745f3a7408b28049a6fb0306fe379c4. Both generated-runtime matrices now carry this fault. The finding in review-result:consumer-explorer-struct-members-pass1 is fixed.

Approval is for this bounded source unit and local transfer into the held integration bundle. It does not claim complete touched-crate, feature-off, workspace, remote or release gates. Existing unsupported inner types and finite witness boundaries remain explicit separate obligations. The constrained Optional newtype fixture's ValueInvariantUnwitnessed refusal is an independently reproduced fixed-suite observation gap, tracked separately; the passing explorer/history tests do not discharge it. No new source/IR/suite/history/report format or public target API is introduced.

Implementation report SHA256 f710ae6c741fe628d53d188c20d8ed4ecde28c89e6509b313d21669161ebe530; evidence manifest SHA256 40fe301a9fd850ca0dfbd7e9231508e2e0e4c0cc2cc36edda9cf3fc61e0ecc67. Retained private evidence is named ess-293-implementation-20261003 and ess-293-root-review-20261003. Publication remains with the single held ess/21 integration carrier; no independent PR.

```findings
[]
```
