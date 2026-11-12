---
format: aep.planning-md/3
id: story:a-reader-beside-a-plain-related-tenant-guard-is-witnessed
kind: story
status: draft
title: A reader beside a plain related tenant guard is witnessed
summary: 'Follow-up of story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field: GetChild/found stays refused (ESS-SYNTH-001) beside a second related row compared plainly; pre-existing adversary finding, backlog'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field` creates every related
row a child's creator guards on the input under the tenant the scenario sends, and an `Optional`
tenant compared plainly (`tenant != input.tenant`) under that tenant. A reader of the child
(`GetChild`) beside a second related row compared plainly is still refused: with the full parent
guard and the folder (or folder and shelf) guarded `tenant != input.tenant`, `GetChild/found` is
refused as ESS-SYNTH-001 ("the command creating them refuses a missing related row through the
input the selector compares, and no arrangement left a related row for the rows and the send to
name"). A witness exists: create the parent and the folder under tenant T, then send T.

The same root likely leaves `GetChild/found` and `GetChild/child-not-found` refused when every row
(parent, folder, subject) is compared plainly
(`every_related_row_and_the_subject_compared_plainly_are_created_under_the_tenant_sent`, which
accepts a refusal by name today): the reader's own parent is arranged without a tenant, so the
creator must send an absent tenant and the plain folder guard reads Unknown.

Found by the second adversary pass over the parent tenant guard; pre-existing (base `ccefa367b7`
refuses with the same text).

## Acceptance

1. `adversary2_the_parent_guard_beside_a_folder_compared_plainly_leaves_found_and_deleted` and
   `adversary2_three_related_rows_two_compared_plainly_leave_every_witness` in
   `crates/verify/ess-conformance/tests/parent_row_guard.rs` lose their
   `#[ignore = "backlog: plain-related-row-reader ..."]` and pass with their assertions unchanged.
2. The every-row-plain case synthesizes `GetChild/found` and `GetChild/child-not-found` and the
   interpreted run reports 0 failed and 0 unsupported.
3. No suite-bytes row moves except for a model whose reader sits beside a plainly compared related
   row.
