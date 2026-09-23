---
format: aep.planning-md/1
id: task:consumer-accounting-v2-mechanism
kind: task
status: implemented
title: Implement finite accounting v2 and separately executed acquisition obligations
relations:
- derived_from: story:consumer-accounting-baseline-never-extended
- informed_by: review-result:consumer-accounting-applicability-pass-1
- serves: vision:O2
revision: 6
---
## Outcome

Implement the reviewed accounting/v2 mechanism and all eight separately executed acquisition
obligations, preserving exact frozen eligibility and truthful incomplete-matrix reporting.
This is a decomposed implementation slice of story:consumer-accounting-baseline-never-extended.
The parent still owns every remaining behavioral cell, final qualification and full ESS gates.

## Accepted authority

docs/design/consumer-accounting-reconciliation.md and consumer-accounting-applicability.md fix the
finite decisions, closed formats, eight literal profiles, five aggregate identities and two proof
modes. Independent design review consumer-accounting-applicability-pass-1 approved with no findings.
The source scope was independently read and adopted in
local-evidence:ess-evolution/waves/0004-ess-accounting/implementation-scope-result.md.
Existing Profile, Cell, Claim, PlannedCell and Baseline types remain the typed homes; no new
product entity or ESS dependency on AEP is introduced.

## Acceptance

- Keep historical accounting/v1 readers genuinely closed and preserve their meanings; add decisive
  old-reader refusals before v2 envelopes or new dispositions. Introduce typed closed v2 candidate,
  execution-plan and qualified paths without silently extending shared v1 enums.
- Implement finite reconciliation/1 consumed exactly once over exact frozen stale tuples. Preserve
  unchanged unknowns and independently count retirements; replacements require exact current
  behavior or a qualified permitted aggregate. Unknown/metadata cannot silently replace a cell.
- Implement the eight literal ScenarioAcquisition rows and guards, separate from the model matrix
  but retained in source inventory. Execute all eight direct owner-bound cases through the normal
  exact native-case boundary with precise identity/bytes/origin/refusal/omission assertions.
- Preserve old test-target identities and native evidence meanings. Binary-unit selection belongs
  only to new acquisition/v2 authority; bind package/kind/name, Cargo target source and actual
  case-module AST independently, preventing cross-target reuse or unobserved nested execution.
- Implement private fresh aggregate proof and exact same-consumer child qualification for both
  reviewed modes, with canonical old witness validation, reference closure, complete finite
  partitions, residual provenance and no caller-deserialized current proof. Old witnesses must
  reproduce the five frozen hashes; unsupported/incomplete authority remains a concrete refusal.
- Produce exact proposed authority data and execution evidence for root review. Do not auto-approve
  generated candidates or make the gate rewrite its reviewed authority files. Preserve the frozen
  initial-baseline.json SHA256 3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de.
- Demonstrate decisive initial failures, restored causal mutations, complete affected-package
  suites, strict Clippy and formatting. Candidate and partial outputs claim no qualification;
  parent acceptance still requires zero missing/stale cells, actual task consumer-check, default
  task check and task site-build. No gate skip or smaller matrix closes the parent or initiative.

## Scope

Cited primary surface: crates/edge/ess-xtask/src/consumer_coverage/{account,enforce,proposal,mod,
executor,native,rust,wire}.rs and existing tests/authority inputs in that directory.
Cited acquisition owner: crates/edge/ess-cli/src/input_discovery.rs.
Inferred additions: reconciliation.rs, scenario_acquisition.rs, aggregate.rs, accounting_v2_tests.rs,
reviewed-reconciliation.json, reviewed-scenario-acquisition.json and reviewed-aggregate-closures.json
under consumer_coverage/, plus crates/edge/ess-cli/src/input_discovery_accounting_tests.rs.
Inferred preservation hook: consumer_coverage/preservation.rs; retain its closed input checks.
Old witness fixtures belong to a bounded consumer_coverage/fixtures/ directory and are read as
reviewed historical material, never as current-provider evidence. Root reviews their exact hashes.
The machine-readable scope is on the parent story. There is one implementation writer for these
shared files; remaining behavioral groups must not edit them concurrently.

## Dependencies and limits

No external runtime dependency blocks this accounting slice. Required historical witness material
is locally available at the baseline source commit but remains subject to canonical comparison.
All 3,482 changed-profile rows and other missing/changed matrix cells remain behavioral work under
the parent; this task must not discharge them by classification or a compatibility-unknown bridge.
Source changes remain unintegrated until the complete applicable repository gate succeeds.

## Acquisition production-source binding clarification

The coordinator inspection review-result:consumer-accounting-acquisition-source-inspection found
that consumer.rs declaration hashes include function signatures but omit bodies. Exact current
provider/source freshness alone therefore cannot authorize a reviewed acquisition implementation.
No scenario authority was adopted before this finding.

Keep entrypoint_sha256 and all existing profile hashes unchanged in meaning. The selected closed
row now additionally binds entrypoint_source (the exact production input_discovery.rs path),
entrypoint_ast_sha256 (its actual authored source_item_sha256 including the body), and
entrypoint_source_file_sha256 (the production module's raw source bytes including local helpers).
Retain separate owner-case AST/source hashes and normal source/tool/native checks around execution.
Candidate extraction obtains the actual production fields; full candidate/reviewed-row equality
authorizes only the reviewed body and module. No global hashing change or filesystem abstraction.

Because these are new persisted fields, the acquisition authority/candidate/plan/proof/qualified
family is version2. Refuse old/1 data instead of reinterpreting it. Preserve a closed old-reader
rejection test. The unadopted signature-only/1 proposal remains historical evidence; no unrelated
ESS format, baseline eligibility or parent accounting/v1 meaning changes. Accounting/2 names the
explicit nested acquisition format.

The correction is proved with an actual authored body containing a discarded default-directory
probe, processed by the production AST extractor and candidate builder. Its signature must remain
equal: show the old guard's acceptance gap, then corrected guard refusal. Also retain an on-disk
mutation through compiled extraction and normal guard entry, restore source, and execute all eight
normal exact cases. Editing a hash string to a made-up mismatch is not the required source control.
Root reviews and adopts only the regenerated complete eight-row authority. No incomplete matrix,
retirement-only proposal or partial execution closes the parent or qualifies missing model behavior.

## Bounded implementation result

Local bot submission a9b1fb015774b08a27a460b187cc05da0f25f324 contains the completed mechanism and
both review corrections. Two independent code examinations are recorded. Final findings ledger:
zero carried, one new and two resolved; the final new manifest-binding defect was then corrected,
its exact regression passed and root inspected the two-line fix and preserved assertions under
the post-second-pass verification rule. No third examination was opened.

Complete affected xtask execution passed218tests with three pre-existing ignored; strict package
Clippy and formatting passed. The CLI/acquisition evidence and protected hashes are retained in
the previous implementation/correction records. Full-workspace formatting has existing generated
drift; no unrelated source was rewritten. Normal consumer-check previously executed and qualified
all eight acquisition obligations, then refused at missing complete reconciliation. This result
completes this bounded implementation task, not parent coverage or repository integration.
All8380replacement behaviors, complete reconciliation/aggregate adoption, actual full default
consumer-check/task check/site-build and integration remain the parent's required work.
