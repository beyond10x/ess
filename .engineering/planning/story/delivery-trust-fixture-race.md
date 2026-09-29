---
format: aep.planning-md/3
id: story:delivery-trust-fixture-race
kind: story
status: draft
title: Delivery-trust tests never copy a fixture another test is rewriting
relations:
- serves: vision:O2
revision: 1
---
## Outcome

The delivery-trust tests that copy a compiled release-tool fixture never race each other: each
test copies from a fixture no other test rewrites while it runs.

## Acceptance

- `ess-cli::delivery_trust` passes under nextest's parallel runner 20 times in a row.
- A compiled fixture under `target/tmp/compiled-fixtures/` is written once (atomic rename) and
  never replaced while another test copies it.

## Origin

PR #248 CI, Test 2/4 (run 36604168638): `adversary_component_release_unit_mismatch_stops_action_before_generic_check`
failed with `/bin/cp: skipping file '…/compiled-fixtures/fake_release_component-84db7bdb65eda26a',
as it was replaced while being copied`; the re-run passed.
