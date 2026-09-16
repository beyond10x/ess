---
format: aep.planning-md/1
id: blocker:live-pilot-schema-metadata
kind: blocker
status: open
title: Full ESS gate is blocked by the existing code-name schema metadata mismatch
relations:
- blocks: story:compact-live-authoring
- blocks: story:callable-live-evaluator
withholds: test_result
revision: 2
---
## Observation

The required full `task check` reaches the xtask tests and fails nine metadata
accounting cases with `stale or unreviewed schema metadata row
wire:RawSpecFile#/definitions cli-binding-resolution`. The isolated native
`fresh_provider_and_exact_six_relationships_share_the_actual_wire_inventory`
test reproduces the failure without the conformance dependency graph.

`cargo run -p ess-xtask --locked -- schema --check` confirms the generated schema
is current. At the unchanged base, the committed definitions container hashes to
447a15a5bd212cd0073f98ec28c1964d190e06b46240a5aed564b73bd4766022,
while the reviewed rows pin
3c1ce721245397ae759f2f3c6fb91ce4039d0aee9c211043c996c73657e6f79a.
The diff from 068804533346a69162640687b17547a664ea1a0b to HEAD adds the code-name
property to Naming and the flattened entity naming fields, and updates Naming's
description. This is the already-committed cf627cf1 change, not a compact-live
source-model change.

## Clearing condition

Review the exact new definitions-container relationship under
docs/design/cli-schema-metadata-accounting.md, retain all descendant obligations,
and run the complete native metadata and consumer gates. Do not replace the hash
solely to obtain a green test, edit sealed baselines, or count focused pilot tests
as a passing full ESS gate. Pilot implementation can continue; landing is blocked.

## Upstream fix and current verification

The feature checkpoint was rebased onto upstream d85f083c, which includes
af569cad's reviewed schema metadata and release-accounting fixes. At candidate
65ed702a all metadata tests now pass in the complete workspace test lane. The
full task check then reaches a different refusal, recorded as
blocker:live-pilot-consumer-coverage. That consumer run executes no guards or
behavior cases, so the original clearing condition requiring completed native
consumer execution is still pending. Do not report the full gate green.
