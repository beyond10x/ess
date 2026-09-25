---
format: aep.planning-md/2
id: review-result:consumer-accounting-entry-authority-pass-1
kind: review-result
status: active
title: Independent finite entry authority audit for accounting v2
relations:
- reviews: task:consumer-accounting-v2-mechanism
revision: 1
---
Read-only entry-authority audit verdict: PASS for all 122 proposed OwnedHelper classifications as semantic classifications; FAIL for adopting the 122 additions alone as a complete current classification map, because 11 pre-existing enforce classifications are now stale.

Inputs/digests verified:
- entry-classifications.additions.proposed.json = d24fe612719c0033eb2e21a1ca865275af4707901743929fd8824a6f4ab1641a (matches brief)
- new-entry-classification-identities.json = 9d18e0469f75f5d9f905ee5a5ce6175805b7e9e17cd3e321b6d2704df1b769b5 (matches brief)
- current entry-classifications.json = 3db108040f822879e720439e27aa39337345c8479c9b5afc4764276499422907
- initial-baseline.json = 3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de (required immutable digest)
- checkout HEAD = 41da2281e99402602c25d8faf219fc75b1954d04.

122-set checks:
- Proposal object has 122 keys; identity array has 122 unique strings; exact set equality; zero overlap with current classification keys; every row has class OwnedHelper, the same nonempty reason, and null macro guard; there are no macro identities.
- Static current-source enumeration under consumer.rs rules gives exactly: account 13, aggregate 36, enforce 16, executor 2, consumer_coverage root 1, native 2, proposal 1, reconciliation 21, rust 1, scenario_acquisition 28, wire 1 = 122. Source anchors: account.rs:8,42-98; aggregate.rs:7-588 (production declarations, excluding cfg(test) test_validate); enforce.rs:84-119,375-421,611,825,921,946; executor.rs:99,108; mod.rs:418; native.rs:281,348; proposal.rs:25; reconciliation.rs:7-318 (excluding cfg(test) validate); rust.rs:59; scenario_acquisition.rs:7-333 (excluding cfg(test) test_proof); wire.rs:15. Proposal group starts are lines 2,67,247,327,337,342,352,357,462,467,607 respectively.
- No proposed key is in profiles.json entrypoints. bind_profiles requires BoundEntry iff an identity is a profile entrypoint (proposal.rs:181-233); proposal overlap is zero. Therefore none of these OwnedHelper labels suppresses a declared model consumer or the eight separately governed ScenarioAcquisition profiles. The ScenarioAcquisition variant at proposal.rs:25 merely enables the closed profile class; the eight exact rows remain inventoried and mandatory under scenario_acquisition.rs:101-141,144-247,273-333.
- Aggregate/reconciliation/enforcement entries operate only the evidence/accounting gate. They do not consume an ESS model at a product boundary, and current code retains exact authority/case checks: reconciliation.rs:148-318; aggregate.rs:94-153,178-233,339-588; enforce.rs:421-608,825-1009; native.rs:281-425. Historical witness readers at rust.rs:59 and wire.rs:15 support the exact aggregate structural proof and grant no model behavior.
- The common reason is consistent for all 122: each is v2 accounting, finite reconciliation, acquisition, aggregate provenance, or historical-witness infrastructure; its explicit no-model-behavior/executed-case limit matches the accepted designs (consumer-accounting-reconciliation.md:23-57,67-88; consumer-accounting-applicability.md:26-59,68-112,114-147) and existing adjacent OwnedHelper precedent.

Blocking finite-set finding — remove these 11 OLD classifications when adopting additions:
1. enforce::struct::Baseline — classification map line 37607; now cfg(test) at enforce.rs:7.
2. enforce::struct::Group — map 37622; now cfg(test) at enforce.rs:22.
3. enforce::struct::PlannedCell — map 37627; now cfg(test) at enforce.rs:49.
4. enforce::struct::ExecutionPlan — map 37617; now cfg(test) at enforce.rs:155.
5. enforce::fn::plan_with_metadata — map 37577; now cfg(test) at enforce.rs:189.
6. enforce::fn::baseline_cells — map 37562; now cfg(test) at enforce.rs:294.
7. enforce::fn::insert — map 37572; now cfg(test) at enforce.rs:650.
8. enforce::fn::validate_header — map 37597; now cfg(test) at enforce.rs:664.
9. enforce::fn::read_plan — map 37592; now cfg(test) at enforce.rs:693.
10. enforce::fn::qualify_cells — map 37587; now cfg(test) at enforce.rs:785.
11. enforce::fn::qualify — map 37582; removed from current source (replaced by qualify_v2 at enforce.rs:946).
consumer.rs excludes cfg(test) items from entries (lines 172-193,225-314,425-466). classify rejects every classification whose identity is absent (proposal.rs:489-513, especially 498-500). The first unclassified-entry refusal masks this stale-key refusal until all additions are present.

I found no missing proposed production declaration, duplicate, inconsistent reason, wrong class, or authority suppression among the 122. Current modified-source diff adds no other production inventory identity outside this set; test modules/functions are correctly excluded. The only current set correction visible is the 11 stale removals above.

Limits: no build/extraction/fixture use per brief. Thus I could not establish compiler-backed exact completeness across Cargo metadata/targets, active cfgs, syn parsing, generated declaration hashes, or the provider/source checkpoint. Static source plus exact-diff enumeration establishes the 122 current additions and 11 current stale keys; root’s eventual real extraction/gate must confirm there is no further inventory drift. I read the managed AGENTS.md; both accepted design docs; both proposal files; current classifications/baseline/profiles; consumer.rs inventory semantics; proposal.rs classification/binding logic; mod.rs; and the adjacent account/aggregate/enforce/executor/native/reconciliation/rust/scenario_acquisition/wire sources and diffs. No files edited, no builds, no fixture content used.
