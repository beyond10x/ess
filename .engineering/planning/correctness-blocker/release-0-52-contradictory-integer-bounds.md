---
format: aep.planning-md/3
id: correctness-blocker:release-0-52-contradictory-integer-bounds
kind: correctness-blocker
status: open
title: PR402 weakens contradictory equality invariants in generated JSON Schema
relations:
- blocks: task:release-0-52-0-20261003
withholds: test_result
revision: 1
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
