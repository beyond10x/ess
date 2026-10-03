---
format: aep.planning-md/3
id: correctness-blocker:release-0-52-contradictory-integer-bounds
kind: correctness-blocker
status: cleared
title: PR402 weakens contradictory equality invariants in generated JSON Schema
relations:
- blocks: task:release-0-52-0-20261003
withholds: test_result
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-03T01:57:48Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
---
## Verified failure

PR402 candidate 1b2ed5857ab60fd4b55ccee1a041df6b546f124d is not releasable even if its existing CI becomes green. A peer ran ten focused integer_bounds tests with the real JSON Schema validator in managed review tree ess-release-402-audit-20261003: eight passed, two failed, exit101. The release owner independently read the retained test.log and test.exit. Production source was unchanged; the audit added tests only.

Both contradictory_equalities_preserve_every_constraint_in_either_order and contradictory_newtype_equalities_reject_both_values_in_either_order fail: Eq1 plus Eq2 accepts whichever value is last, in both declaration orders. The shared apply_bound in crates/generate/ess-gen/src/types.rs overwrites const. The reviewed correction 1c4e6149a is absent from this candidate.

Controls pass: contradictory Optional constraints reject numeric values while preserving null and absence; equality/range intersections preserve the intended constraints and consistent values.

## Evidence

Retained peer evidence: serial-20261003/402-audit/test.log, test.exit and tests.patch under its task evidence cache. The full log records the two accepted counterexamples in each failing case and the result 8 passed; 2 failed; 0 ignored. The review worktree retains the tests-only patch. These references are observations, not a claim that the corrected candidate has passed.

## Clears when

The transport owner corrects the common apply_bound behavior without discarding contradictory equality constraints, preserves the reviewed required-field behavior and the new integer-newtype controls, and passes all ten exact audit cases on a frozen corrected candidate. Record that candidate and the red-to-green evidence; then require the normal affected-package and final integration/release gates on the actual delivered source. A green Gate on the old candidate, a documentation-only fix, or a status change without validator evidence does not clear this blocker.

## Ownership

Transport implementation remains with the third Claude session. The release owner makes no product-source patch or transport PR. The separate held full ess/21 carrier remains outside PR398. The existing batch-completion hold also remains open until its own condition is met.

## Correction and ownership handoff, 2026-10-03

After transport PR402 merged at 2f554561bef25125a93a1fb1d6517d50cb24ed20 and the former release owner became unavailable, the release integrator resumed the existing PR398 under the operator's release instruction. Transport development remains with its Claude owner; this bounded release correction changes only the shared schema equality intersection and its regression tests.

Correction b4b74139b1f71212677a280dddfa525ad32c6eb6 preserves the first equality and intersects contradictory values into unsatisfiable bounds. The exact ten audit cases changed from eight passed/two failed to ten passed. The affected ess-gen package changed from 301 passed/two failed to 303 passed/zero failed. Strict package Clippy and formatting passed. Independent review added permutation and integer-boundary controls in ced143ebac98bf032351a95f13ea8c3e110eeb14; twelve focused tests pass, including optional absence/null behavior. Review found no remaining issue. Its immutable report is review-result:release-equality-adversary-20261003. Private retained raw report SHA256 is b9d0f0d5b770579fe29d7a98b0260d0d004ddb66dd59af4f6cac45b02d4d447f.

Both commits are integrated through 7f99f8168 in the sole release branch release/0.52.0-20261003. The demonstrated schema defect is corrected; normal final integration and release gates remain mandatory and pending. This is not evidence that the full release gate has passed.
