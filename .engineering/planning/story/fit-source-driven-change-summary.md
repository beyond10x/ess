---
format: aep.planning-md/1
id: story:fit-source-driven-change-summary
kind: story
status: implemented
title: Fit the source-driven change summary to the publication contract
relations:
- serves: vision:O2
- informed_by: release-plan:consolidated-ess-019
scope:
- confidence: cited
  path: changes/source-driven-realization-0.19.0.yaml
revision: 7
---
## Observed publication refusal

The normalized documentation bundle for independently published ESS5201daf passes source packaging, but Atlas snapshot-source-set and the pinned Docs System1c8c316 validator reject changes/source-driven-realization-0.19.0.yaml: /summary must NOT have more than360 characters. This blocks the current catalog delivery after the larger source-driven integration. The earlier87338bd source-set Website gate passed99 tests and full build/publication verification; it must not be substituted for the current source.

## Acceptance and bounded correction

Shorten only the summary to at most360 characters while retaining complete schema-root imports/source identity, structural Go/Rust/TypeScript libraries, source-pinned normalization, Rust's library API and pending Go/TypeScript normalization, checked Rust/Web failures and explicit slice digest profile. Preserve the change ID, version, timestamp, source URL, title, kind, impact and all other fields. The exact pinned Docs System validator must accept the corrected document. Publish the source through the bot, refresh the standard complete source set, and verify its snapshot and Website delivery. This creates no release/tag or runtime implementation change.

## Scope and validation

- changes/source-driven-realization-0.19.0.yaml — cited failing public metadata, summary only.
- Original refusal is retained in Atlas coordination scratch delivery-state-final/snapshot-consolidated.stderr and direct Docs System validate output. Run the same existing validator after the edit; no new implementation-mirroring test is needed.
- The final review gate/SDK evidence belongs to87338bd. The later integrated source5201daf is separately owned; preserve its planning records and release observations.

## Verified combined-source delivery — 2026-09-06

The consolidated ESS source ba43fda29de637ad9323d96c4bb9aac10f48ae64 passed every combined gate lane at 2026-09-05T22:42:51Z, including 1710 Rust cases (zero failed/ignored), site build and planning validation. Subsequent main dcb84be861d2f906b3dd95254f03701cb264faa2 preserves that source and adds only planning records. The existing catalog's bounded post-integration correspondence report found no mismatch with the five additional source-driven formats and digest domains; its source inspection is kept distinct from executable evidence.

The exact passive ESS bundle is24a32e3d70120dde81bac029dd7117b46ad7ea20718af518dab5781302979ac2 from producer run33996739936 attempt1/artifact9978285510. Atlas publication [33997518894](https://github.com/beyond10x/atlas/actions/runs/33997518894) completed success at2026-09-05T23:14:46Z. Root deployment [33998081746](https://github.com/beyond10x/beyond10x.github.io/actions/runs/33998081746) published durable commit8c2a069c8c99c3b19c9cc5a8469f788c53fb619c with the independently pinned Websitefc4571534765c098ed861bc326da4d3da0d1df63 runtime. The source setd30cd43ccc869a9ee723e8777c4209a7a8eaac77c847bcae1f1068c3f1b0858a binds Atlas producer858420b11a2a2deaf65063483713febfbc884af1, all24 sources and ESSba43fda.

Live PROVENANCE captured at2026-09-05T23:24:49Z equals the immutable published bytes (SHA256d12e437623d1deac41f6ce7f5289b8f4d5b6168d56726833545df36470cf2986). Root independently compared its completed resolver's entire source-set and normalized source trees to the immutable published inputs; both comparisons exited0. The exact published Atlas bootstrap was retained instead of regenerated under a later authority.

At2026-09-05T23:31:54Z, npm run gate at runtimefc457153 against those exact delivered inputs passed99 tests, zero failures/skips, and the complete site build. Publication layout and independent verification exited0 at23:31:56Z, covering356 routes and1327 files. Raw results are retained in Website target/ess-catalog-delivery/gate-runtime-pin-live. Publication's own build/verify job and the earlier runtime99-test gate are separately attributed; neither was substituted for this complete candidate gate.

Atlas4911915's full fence completed at23:34:01Z with exit1:116 Rust passes/one previously reproduced relay timeout failure; AgentIDE's primary v4 collector mismatch, the stale primary Website Docs System pin and Widgets' missing Serves declaration remain recorded baseline failures. The unchanged relay source returned202 where the test expected200; its dedicated follow-up remains draft. Catalog, generated projections, markdown, brand and all52 live delivery routes passed. This is not an organization-wide green claim. The later scheduled publication33997804307 failed in resolution before any publication; it did not roll back the verified successful artifact. No failing check, legacy result or full adversarial report was removed or weakened.

All four Wave5 implementation/SDK experiment trees were removed through reviewed exact-id managed GC after publication and byte-verified evidence archives. Raw reports and standalone fixtures remain archived in the remediation cache; the reviewed disposable compilation products were removed separately. Remaining coordinator/Atlas/Website/Docs System support records retain explicit owners until their governance publication and final cleanup are completed. The source catalog and actual public delivery acceptance are now met; the separate schema-resource identity story and overall F13 follow-through remain open.
