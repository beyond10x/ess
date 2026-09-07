---
format: aep.planning-md/1
id: verification-report:consumer-coverage-stage1-checkpoint
kind: verification-report
status: draft
title: Consumer coverage extractor and finite eligibility checkpoint
relations:
- verifies: story:review-consumer-coverage
- informed_by: epic:qualify-initial-consumer-baseline
revision: 1
---
# Consumer coverage Stage 1 checkpoint

Root inspected the complete Stage 1 source/output checkpoint at local bot commit
7a6d76855e28d280138d2d439eb6f1e7c15a58d9. Package verification passed 71 cases,
strict Clippy and formatting; two final same-provider extractions match all 13 files.
Root independently checked all 157,122 cells, 54 mandatory exclusions, closed 148-declaration
Rust graph, complete source/provider stamps, preserved native images and the complete
3,623-entry target/TMP census. Full Stage 1 report SHA256
19ac6808b91b660d91b6bebc121cb05b3b0021e440309a0e780e69e2d071d459;
seal2 SHA256 3da17b0a507016832c41b3116f12e30900970120b5cab1a5a4a8f5fc80341ffc;
root final readback SHA256 05d78f21c2d8c1c16ccdcdb936b9d862670a29c895c36c7e4f6c30d6b1d25985.
This is an extractor checkpoint, not behavioral support or story completion.

Root accepts only the finite 157,068 initial unknown pairs in
crates/edge/ess-xtask/src/consumer_coverage/initial-baseline.json, SHA256
e005a2e74e067ad51315e594151643b702381dc174bb9af5c355c3eeeb17ad51.
Each exact group has its package owner, profile/model shape pins and bounded unproven
statement. The independently owned epic:qualify-initial-consumer-baseline remains draft;
its qualification work is outside the original 31 remediation stories. The current gate
story cannot own its own unknown follow-up. The known Go panic remains broken and its
concrete repair stays with story:fuzz-the-specification-surface.

The final conditional owner-test reservation is now selected and recorded as inferred:
crates/verify/ess-diff/tests/consumer_coverage_f01.rs. It contains only the five isolated
relation kind/target/carrier and view-parameter order/type comparison witnesses. Root's
prepared six authored fixtures passed actual retained-baseline CLI validation; the Rust
cases remain uncompiled and unexecuted preparation. The existing xtask, Taskfile, lockfile
and binding reservations remain. No model or production consumer repair is selected.
Cargo.lock scope is now confirmed by five package-local dependency edges with no version
or package additions; binding ownership is confirmed by its actual reviewed file.

Stage 2 implements closed accounting and exact actual-case execution, completes those
mandatory witnesses and the production causal mutations, then returns for independent
source adversary review and the entire repository gate plus site-build. Root owns shared
Taskfile/design/planning edits, accepted eligibility, Git, integration and publication.
Stage 1 alone does not satisfy the acceptance statement or move this story to implemented.
