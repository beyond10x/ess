---
format: aep.planning-md/1
id: verification-report:review-boundaries-10-versioned-source
kind: verification-report
status: draft
title: Schema workflow final versioned source gate
relations:
- verifies: story:review-schema-resource-identity
revision: 1
---
## Final published source gate

Published ESS239996d846460aee342ce42514378c25b2be5152 contains the reviewed schema workflow and separately published0.20.0 preparationc90ca1b2a3a5db02d7580dab63be6cbc56679e0b. After reconciling that incoming version change, root reran every8declared task-check lane, the separate site-build and planning validation. All10commands returned0. Workspace tests passed2018cases,0failed/ignored,178summaries in143.793475881seconds. Every tracked source byte stayed unchanged after each lane and at completion. Source gate began2026-09-06T15:56:55.402521Z and completed2026-09-06T16:00:49.987709Z. No optional feature-specific TypeScript suite beyond the declared gate is inferred.

The previous green9e82d20 gate remains in verification-report:review-boundaries-10-integrated; its result is not relabeled as this one. The prepublication remote equality check refused before any push when c90ca1b appeared. Root preserved its complete1414-line canonical journal and replayed17semantic AEP commands producing19events, verifying163unrelated incoming artifacts and all3local artifacts byte-exact. Both public pages merged, incoming Cargo manifests/lock/design version changes are unchanged, and the complete incoming0.20.0 changelog section is preserved. Only this task's new schema entry remains Unreleased.

Both bot identities were verified on the final integration. The exact green239996d commit was then pushed and read back as advertised main; no release, tag, installation or version change was selected by this wave. The publication workflow was dispatched after the exact source bundle succeeded. Expected joint sources are ESS239996d and AEP658cf76e6371b1628f6de69548e724b52803f5c2; its actual artifact and public delivery remain separately measured outcomes.

Full raw source/toolchain/environment/argv/status/log evidence is retained under target/review-boundaries-10/gate-239996d84646. The earlier unit/review sources, first fixture failures, corrections and archive evidence retain their original identities.

## Individual gate rows

```json
[
  {
    "step": "fmt-check",
    "argv": [
      "task",
      "fmt-check"
    ],
    "started_at": "2026-09-06T15:56:55.402521+00:00",
    "finished_at": "2026-09-06T15:56:56.799066+00:00",
    "seconds": 1.396546024014242,
    "exit_code": 0,
    "free_before_bytes": 39799758848,
    "free_after_bytes": 39799750656,
    "rust_summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "e96f1cab699f6a4bb3c4dd64f88e7ce3dfb5aaaa299891416e29335e005cd893",
    "source_unchanged": true
  },
  {
    "step": "clippy",
    "argv": [
      "task",
      "clippy"
    ],
    "started_at": "2026-09-06T15:56:56.873340+00:00",
    "finished_at": "2026-09-06T15:57:18.253836+00:00",
    "seconds": 21.380495412973687,
    "exit_code": 0,
    "free_before_bytes": 39799746560,
    "free_after_bytes": 39778689024,
    "rust_summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "ccc756394548b2e1b3df1134b4a83635ec6030c34d434069613731277edd1536",
    "source_unchanged": true
  },
  {
    "step": "test",
    "argv": [
      "task",
      "test"
    ],
    "started_at": "2026-09-06T15:57:18.335431+00:00",
    "finished_at": "2026-09-06T15:59:42.128904+00:00",
    "seconds": 143.79347588098608,
    "exit_code": 0,
    "free_before_bytes": 39778689024,
    "free_after_bytes": 42318004224,
    "rust_summaries": 178,
    "passed": 2018,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "ef7ec4f23be0759f84930d2edcfe5df53fae4d8946b31b2f51212fa085e8c0a3",
    "source_unchanged": true
  },
  {
    "step": "doc-check",
    "argv": [
      "task",
      "doc-check"
    ],
    "started_at": "2026-09-06T15:59:42.202118+00:00",
    "finished_at": "2026-09-06T16:00:02.979767+00:00",
    "seconds": 20.777651241980493,
    "exit_code": 0,
    "free_before_bytes": 42318004224,
    "free_after_bytes": 42331373568,
    "rust_summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "33fda5b13d26958beeb8bf05e85eb2a7c7f5b8a08bfb6c35f01bf0600b4d54f3",
    "source_unchanged": true
  },
  {
    "step": "example-check",
    "argv": [
      "task",
      "example-check"
    ],
    "started_at": "2026-09-06T16:00:03.052728+00:00",
    "finished_at": "2026-09-06T16:00:08.848502+00:00",
    "seconds": 5.7957743590231985,
    "exit_code": 0,
    "free_before_bytes": 42331365376,
    "free_after_bytes": 42217218048,
    "rust_summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "ed589817ee6baa31cfa20701f4bdc592332a146236d0eb890353581a9e7876f5",
    "source_unchanged": true
  },
  {
    "step": "projection-check",
    "argv": [
      "task",
      "projection-check"
    ],
    "started_at": "2026-09-06T16:00:08.921444+00:00",
    "finished_at": "2026-09-06T16:00:16.000220+00:00",
    "seconds": 7.078776063979603,
    "exit_code": 0,
    "free_before_bytes": 42217218048,
    "free_after_bytes": 42139508736,
    "rust_summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "d67703f04c862e265e9b0d6276196cbaa9e98ec58d32cc62b88b08fff8e94f66",
    "source_unchanged": true
  },
  {
    "step": "release-check",
    "argv": [
      "task",
      "release-check"
    ],
    "started_at": "2026-09-06T16:00:16.073382+00:00",
    "finished_at": "2026-09-06T16:00:16.171238+00:00",
    "seconds": 0.0978581829695031,
    "exit_code": 0,
    "free_before_bytes": 42139504640,
    "free_after_bytes": 42139471872,
    "rust_summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "a0ddaec10557806a60282302c856a4ee919bbb5ff393481d2fb750a7b8fe7b10",
    "source_unchanged": true
  },
  {
    "step": "action-check",
    "argv": [
      "task",
      "action-check"
    ],
    "started_at": "2026-09-06T16:00:16.244186+00:00",
    "finished_at": "2026-09-06T16:00:16.291376+00:00",
    "seconds": 0.047190212993882596,
    "exit_code": 0,
    "free_before_bytes": 42139459584,
    "free_after_bytes": 42139455488,
    "rust_summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "ce62e1f35e7bcdfebdd7ed28735f5d7e206b8f32f6f5fc8e2172e216eac900b9",
    "source_unchanged": true
  },
  {
    "step": "site-build",
    "argv": [
      "task",
      "site-build"
    ],
    "started_at": "2026-09-06T16:00:16.364900+00:00",
    "finished_at": "2026-09-06T16:00:32.088871+00:00",
    "seconds": 15.723972088075243,
    "exit_code": 0,
    "free_before_bytes": 42138914816,
    "free_after_bytes": 42133225472,
    "rust_summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "3d94ef4dddadb131163596628b91f3a6e307df350ce29233ad7c37a1fe5736a2",
    "source_unchanged": true
  },
  {
    "step": "planning",
    "argv": [
      "/home/timo/.local/state/worktree/trees/b10x/aep/ess-conformance-v2-reader/target/debug/aep",
      "plan",
      "artifact",
      "validate"
    ],
    "started_at": "2026-09-06T16:00:32.162822+00:00",
    "finished_at": "2026-09-06T16:00:49.987709+00:00",
    "seconds": 17.824888088041916,
    "exit_code": 0,
    "free_before_bytes": 42133504000,
    "free_after_bytes": 42153021440,
    "rust_summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "log_sha256": "0576ac3c97396abcb658252dde5e7c217268ccc1568b6df07ce3d149236b6078",
    "source_unchanged": true
  }
]
```
