---
format: aep.planning-md/1
id: verification-report:coverage-cli-leaf-adaptation
kind: verification-report
status: draft
title: Coverage CLI leaf inventory adaptation
relations:
- verifies: story:review-conformance-coverage
revision: 1
---
# Coverage CLI leaf inventory adaptation

This records a bounded coordinator decision under the standing remediation implementation
authorization. It is not an operator approval invented to satisfy a lifecycle gate.

The accepted transport binding explicitly adds conform select, with the area path and retained
flat alias. The first complete implementation package run retained 641 passed, one failed,
zero ignored across 60 Rust summaries, with command exit 101. Its sole failure was
main.rs::tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling: actual grouped
leaf count 52 versus the inherited AREA_LEAVES literal 51. The diagnostic enumerates the new
verify/conform/select leaf beside the existing commands.

Root independently read the Select variant, the complete grouped/flat reachability boundary,
the accepted transport binding at lines 103–107 and the original failure. Root authorized only
AREA_LEAVES 51 to 52. All reachability, alias, visibility and argument-preservation assertions
remain unchanged; the flat count remains derived as AREA_LEAVES minus one. The implementor must
retain exact before/after bytes and the failed run, then verify the unchanged assertions.

Original raw evidence is in the managed ess-conformance-coverage-writer tree at
target/review-boundaries-11/coverage-writer/logs/final-package.log, SHA256
af15fc92b730c69c39bb74b1e0d6645a0499611bd9d8c30cfb057438e384f41c.
The .exit file records 101. A passing correction run, frozen source review, complete integration
gate and actual producer correspondence remain separate evidence.

This supplements the earlier future-suite literal /5 to /6 decision. It does not authorize any
other inherited assertion change. The new M37 9-to-8 arithmetic correction is separately grounded
in Billing's four lifecycle states (Cancel excludes two, Issue three, Pay three) and affects only
a newly introduced test.
