---
format: aep.planning-md/1
id: story:fit-source-driven-change-summary
kind: story
status: active
title: Fit the source-driven change summary to the publication contract
relations:
- serves: vision:O2
- informed_by: release-plan:consolidated-ess-019
scope:
- confidence: cited
  path: changes/source-driven-realization-0.19.0.yaml
revision: 4
---
## Observed publication refusal

The normalized documentation bundle for independently published ESS5201daf passes source packaging, but Atlas snapshot-source-set and the pinned Docs System1c8c316 validator reject changes/source-driven-realization-0.19.0.yaml: /summary must NOT have more than360 characters. This blocks the current catalog delivery after the larger source-driven integration. The earlier87338bd source-set Website gate passed99 tests and full build/publication verification; it must not be substituted for the current source.

## Acceptance and bounded correction

Shorten only the summary to at most360 characters while retaining complete schema-root imports/source identity, structural Go/Rust/TypeScript libraries, source-pinned normalization, Rust's library API and pending Go/TypeScript normalization, checked Rust/Web failures and explicit slice digest profile. Preserve the change ID, version, timestamp, source URL, title, kind, impact and all other fields. The exact pinned Docs System validator must accept the corrected document. Publish the source through the bot, refresh the standard complete source set, and verify its snapshot and Website delivery. This creates no release/tag or runtime implementation change.

## Scope and validation

- changes/source-driven-realization-0.19.0.yaml — cited failing public metadata, summary only.
- Original refusal is retained in Atlas coordination scratch delivery-state-final/snapshot-consolidated.stderr and direct Docs System validate output. Run the same existing validator after the edit; no new implementation-mirroring test is needed.
- The final review gate/SDK evidence belongs to87338bd. The later integrated source5201daf is separately owned; preserve its planning records and release observations.
