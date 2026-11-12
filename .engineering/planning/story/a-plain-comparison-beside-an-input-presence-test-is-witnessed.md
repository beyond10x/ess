---
format: aep.planning-md/3
id: story:a-plain-comparison-beside-an-input-presence-test-is-witnessed
kind: story
status: draft
title: A plain comparison beside an input presence test is witnessed
summary: 'Follow-up of story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field: a related-row guard any(not defined(input.tenant), tenant != input.tenant) leaves CreateChild/created refused (ESS-SYNTH-003); pre-existing adversary finding, backlog'
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

A natural tenant guard refuses when no tenant is sent or a different one is:
`{any: ['not defined(input.tenant)', tenant != input.tenant]}` on a related row (a folder) reached
through `when_related via: input.folder_id`. Synthesis refuses `CreateChild/created` as
ESS-SYNTH-003 ("no candidate of the 1 tried satisfies a row of demo.d.Folder … on which no branch
answering before created is selected"), though a folder created under the tenant sent, with that
tenant sent, witnesses it.

Found by the second adversary pass over
`story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field`; pre-existing (base
`ccefa367b7` refuses the same). The cause is not verified: likely `leaves_to` decides
`not defined(input.tenant)` with no input before the plain comparison is steered.

## Acceptance

1. `adversary2_a_plain_comparison_beside_an_input_presence_test_is_a_plain_comparison` in
   `crates/verify/ess-conformance/tests/parent_row_guard.rs` loses its
   `#[ignore = "backlog: presence-tested-plain-guard ..."]` and passes with its assertions
   unchanged.
2. No suite-bytes row moves except for a model guarding a related row on an input presence test
   beside a plain comparison.
