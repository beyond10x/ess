---
format: aep.planning-md/1
id: story:review-execution-recovery-implementation
kind: story
status: draft
title: Implement finite deployment recovery for the existing F11 obligation
summary: Implement the complete finite deployment recovery contract and offline fault matrix already owed by obligation:review-execution-recovery-implementation.
tags:
- P1
- review-2026-09-05
relations:
- delivers: obligation:review-execution-recovery-implementation
- decomposes: epic:review-boundary-remediation
- depends_on: story:review-execution-recovery-design
- serves: vision:O2
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli/Cargo.toml
- confidence: inferred
  path: crates/edge/ess-cli/src/lib.rs
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/oci_cache.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/recovery/authority.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/recovery/chart.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/recovery/journal.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/recovery/mod.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/recovery/model.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/recovery/observe.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/recovery/process.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/cache_origin.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/cache_origin_adversary_pass1.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/command_surface.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/execution_recovery.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/persisted_delivery.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/cache_origin_attack_client.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/fake_delivery.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/fake_oci.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/support/fake_recovery.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/support/recovery_driver.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/support.rs
- confidence: cited
  path: crates/infra/ess-kubernetes/Cargo.toml
- confidence: cited
  path: crates/infra/ess-kubernetes/src/lib.rs
- confidence: inferred
  path: crates/infra/ess-kubernetes/src/recovery.rs
- confidence: inferred
  path: crates/infra/ess-kubernetes/tests/recovery_adapter.rs
- confidence: cited
  path: docs/design/review-execution-recovery.md
- confidence: cited
  path: models/execution-recovery/domains/execution.yaml
- confidence: cited
  path: models/execution-recovery/system.yaml
- confidence: cited
  path: website/docs/concepts/component-delivery.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/status/limitations.md
- confidence: cited
  path: website/docs/status/where-this-stands.md
revision: 3
---
# Story: Implement finite deployment recovery for the existing F11 obligation

## Outcome

Callers of ESS deployment reconcile can distinguish desired intent, current observation and recorded process outcomes across partial failure and retries, and the executor performs only the finite mutations permitted by the accepted recovery contract.

This story implements the already-open obligation:review-execution-recovery-implementation. It adds the executable proof still owed by F11; it introduces no new remediation objective. The obligation remains open until the complete matrix passes against a recorded integrated implementation commit.

## Context and verified dependencies

Published source and declaration/binding subject: ESS 95ef5be70dbce966056d0484b66db7ed836fa406, identified as published main by root. Historical scoping at 57e242e8a0eaa721968c3970099b4bc561cb91aa, the namespace-main refresh at a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3 and preparation at d7f12b7a027018fc020fadf8f1d66ca6caac1e68 remain retained provenance. Exact Git comparison confirms runtime-source equality with integrated 3408bbf049d10215487c50f6f7b5597486b14127. F11 remains at docs/reviews/2026-09-05-architecture-review.md:394–422; its execution finding is at lines 411–415. The current obligation explicitly requires implementation after typed design, rather than treating design completion as recovery-runtime completion.

Verified store links for root to create: delivers:obligation:review-execution-recovery-implementation; decomposes:epic:review-boundary-remediation; depends_on:story:review-execution-recovery-design; serves:vision:O2. The obligation is open, the design story implemented, the epic active and vision O2 present. The original design is the sole proposed depends_on edge; no dependency on an unverified future artifact is introduced. The protocol's delivers relation explicitly includes obligation targets. No reverse dependency from the obligation to this story is proposed.

The supplied post-browser-replan graph and story lists show story:review-execution-recovery-implementation absent, obligation:review-execution-recovery-implementation open, the original design implemented, the epic active and vision O2 present (draft). Browser replay is implemented. These are the supplied graph snapshot, cross-checked against frozen 95ef store blobs; root refreshes them if the store changes before creation. This is the complete proposed body, without frontmatter or lifecycle/evidence claims. Root owns creation and scope registration after readback. Source and gate observations elsewhere do not count as this story's implemented result.

## Binding and model authority

The published binding is docs/design/review-execution-recovery.md at 95ef, SHA256 9699f5ed6c2f66fc67641d58e0cdf6b8b10f5f397761ef55cea1f1c074a7b989. Root has adopted and published the mechanically consolidated reviewed contract, including the offline qualification. The two exact model-v2 files at their target paths passed actual validate/compile on 2026-09-07T09:14:03Z. Root freshly validated and compiled the exact published declaration again on 2026-09-07T11:06:40Z: both exit 0, with the same 36,251-byte compiled output SHA256 40a9fb6e2ac3c67d27e87b6c935422ada17490637a93d3517e9c9dc28cc7cd20. The fresh receipt is target/review-boundaries-16/preparation/recovery-declaration/validation.json. This refresh verifies receipts and published-byte identity; it executes no validator. Declaration publication and the recorded structural checks are satisfied preparation prerequisites, not recovery implementation evidence or implementation selection. The published binding already applies this precedence, with later inputs overriding only the clauses they explicitly correct:

1. Historical original design constraints and all R01–R29 families in docs/design/review-execution-recovery.md before consolidation (SHA256 679482baa7e9584c78c715fed55d201f89b33d95e6488f7d9b318e5fb15e2783), preserved except where the later inputs explicitly supersede a clause.
2. Authority contract base at target/review-boundaries-14/preparation/execution-recovery-authority-candidate/report.md (SHA256 1b0d0cbc32eef5957cb2ccb6d3435a46b3e388e03cd8d528727b28f8e8e93a0e).
3. target/review-boundaries-14/preparation/execution-recovery-authority-candidate/addendum-v2.md (SHA256 fac0c4de5bcd6483d935a09324193abdf33116eadb4f373943a62f5b4ae5a87f): preparation/observation/Prepared ordering, full active-registry admission and exact Helm artifact/version/protocol.
4. target/review-boundaries-14/preparation/execution-recovery-authority-candidate/addendum-v3.md (SHA256 845cac69786aecc40de6b4a8dc324c32bc147610087090760bd4f880ecfe9248): projection-specific inventories including baseline-only retirement, valid incomplete journal prefixes, complete relevant-history admission and directory publication barriers.

The exact retained model-v2 bytes are published at models/execution-recovery/system.yaml and models/execution-recovery/domains/execution.yaml in 95ef. Their respective SHA256 pins are c981b332cc9bdb08690c3f4f44f732138dc66bf725bcef930ce3ed4836825f28 and e03ca97fc429ab20950c2c75e0c8567e6174cc4c93aca6ca9c6860338c27e294. They declare 44 named types, with no entities, commands or components, exactly as listed in the binding C11. Full-byte identity binds the published declaration to the retained and fresh root validate/compile receipts. These checks remain structural evidence only; changes to the declaration would require fresh validation before dependent implementation.

Preserve the explicit Rust obligations that the declaration does not enforce: canonical/closed reader behavior, cross-record/cardinality/uniqueness rules, original-byte verification, registry admission, filesystem publication, authenticated observations, freshness, ownership, process uncertainty and independently established quiescence. Do not add receipt fields to ess-deployment/1 or ess-release/1. A necessary new persisted noun/field/relationship must be resolved and declared before code depends on it; do not infer it to make validation green.

The coordinator accepted the offline shared-production-code driver qualification in target/review-boundaries-14/preparation/execution-recovery-authority-candidate/implementation-scope/coordinator-decision.json (SHA256 291717217326896e307f7481566186e4c6e513a04c324f811433042a7ed472a9) and the exact 34-path scope in implementation-scope/integrated-refresh/root-readback.json (SHA256 9fc6f284993f03afda945e192c637630c48d7dca96152374473426c408e721f0). That qualification is published in the consolidated binding C13. This story draft records those decisions; it makes no new acceptance or deployment-authority decision. No third candidate review is requested by the mechanical consolidation.

## Acceptance

1. Entire desired/current inputs validate before any external call, cache write or recovery write. Baseline input remains admitted intent rather than proof of application. Preserve strict duplicate-key/intrinsic validation and the early --allow-removals guard. Dry-run, including equal desired and removal preview, is explicitly local/unverified with no target/probe/cache/evidence effects. Both flat and grouped reconcile spellings retain matching behavior.

2. Normal execution requires future caller-provisioned authority for the finite SingleHostGeneratedHelm1 profile. Mechanically scan every active registry entry before selection; validate protected registry/history/host/store/kubeconfig/target/principal identities, registry generation, store roots and aliases, release ownership/incarnation, baseline/desired membership, and overlapping object-address unions. No authority or quiescence decision is fabricated by implementation. Missing, contradictory, changed or unsupported bindings refuse.

3. Admit an exact protected Helm artifact, its complete-byte digest, canonical stable v3.M.P version and Helm3Recovery1 protocol. Probe bounded version/help compatibility and use the same absolute artifact with explicit kubeconfig/context/namespace and constructed environment. Preserve restricted render/status/manifest/hooks/apply/uninstall capabilities. Refuse plugins, wrappers, proxies, extra flags, cross-namespace resources, hooks, CRDs, dependencies, post-renderers, keep/generateName effects and unsupported object kinds. Do not use create-namespace, keep-history or generic ignore-not-found as authority.

4. Consume the existing OCI manifest/blob proof through its exact original-byte verification. Bound and admit the exact generated chart against the caller-pinned RuntimeIr and all five project_helm files, then prepare private chart/values bytes. Establish complete baseline and desired inventories separately; observe their union. Before apply require desired-only addresses absent; after apply require baseline-only addresses absent. Admit retirement with baseline and no desired projection. Foreign incarnations/occupants refuse even with repair/removal flags. Direct objects/finalizers prevent absence; retained descendants/PVCs are not falsely included in a cleanup claim.

5. Use OS-random invocation identity with the caller store epoch and exclusive reservation; require directory and parent durability before claim publication. Durable no-replacement claims survive parent interruption and are never stolen by age, PID, timeout or child exit. Journal canonical immutable entries with readback, hash/sequence/scope checks and required synchronization. Distinguish valid incomplete prefixes from corrupt published entries; scan all relevant history regardless of retry_of omission. Preserve corrupt state for diagnosis and retain history for the epoch.

6. Finish artifact/render/values preparation before fresh Before observation; durably publish Observed and then Prepared referencing that operation's current-invocation Before sequence. Recheck registry, target/principal, claim and admitted Helm identity before launch within the fixed 30-second monotonic budget measured from Before.started. Expiry or changed preparation refuses. At most one admitted mutation is attempted per operation; acknowledged disposition and required fresh durable After precede later mutation. No unbounded retry/reobservation loop is introduced.

7. Process spawn refusal, started failure, timeout, lost acknowledgement and effect/evidence gaps retain distinct claims. Every started uncertain operation stops later applies/removals and retains the protective claim; direct-child reaping does not prove descendant or outstanding API-request quiescence. Fresh observations may resolve current matching/absence without inventing historical applied/removal attribution. A retained predecessor claim requires independently established caller Q before later mutation or completed recovery; without Q, readings remain point-in-time observations.

8. Equal desired inputs still receive fresh observation. Manual drift between invocations refuses implicit overwrite and needs a caller's exact newly reviewed repair_from snapshot. Apply in declared order and retire baseline-only releases in reverse baseline order. Preserve partial prefixes and removal history, refuse foreign replacement, and complete a repeated removal only from authoritative absence plus admitted ownership/history. No multi-release rollback, perpetual-convergence or application-behavior claim is made.

9. Final success requires durable Completed with complete accounting for every selected operation and its required observations. Failure to persist final evidence reports incomplete execution evidence even if all child calls returned success. Keep the accepted text/exit boundary and existing intent formats. No unknown outcome is recast as applied, absent, rolled back or successful merely to obtain a zero exit.

10. Execute the full matrix below with faithful independent process/target/storage controls, every applicable index and variant, original cache/delivery/CLI controls and secret containment. Preserve original manifest/blob byte tests and explicitly distinguish opaque cache-layer vectors from admissible generated-chart execution vectors. An earlier authority refusal may not make a cache test pass without reaching its intended boundary.

11. Update only the four reserved public pages' changed reconcile/support statements, the CLI help and the matching Explicit executors row in ess-xtask/src/support.rs. Preserve the current 20-row block, source/release distinction and dated release observation. Preserve the passive markers [ess-source-support-begin]: # and [ess-source-support-end]: # with their required blank lines. The support check invokes no deployment executor and remains a source-drift check, not recovery execution evidence.

12. Record exact vector IDs, actual executable identities, commands, exits, failures, restart evidence, counts and unchanged controls against a frozen integrated implementation. Pass affected-package checks, all task check lanes and task site-build, and retain the declaration checks. AEP's minimum test_result count is insufficient to meet this story or the parent obligation without the complete integrated proof.

## Namespace-observation compatibility

The namespace observation/IR version2 path now on main is deliberately partial. Its metadata-only namespace/node topology and typed omissions are not recovery observation authority. Preserve that existing CLI/import behavior and its refusal/coverage controls; implement the separately required finite recovery observation adapter specified by the binding. No topology observation or source-support check discharges the authenticated Before/After, ownership, freshness or full inventory obligations. The retained namespace-main contract refresh records the exact changed owners. This 95ef refresh supplies current source pins and the same 34 reservations for creation; root refreshes changed inputs and collisions again before dispatch.

## Offline execution qualification

Use real independent Rust driver processes sharing the production parser, handler, admission, reconciliation, chart, journal and process logic. Their durable synthetic target is independently controlled and survives the first driver; neither the journal nor the executor's result regenerates target state. Keep effect, launch/acknowledgement, observation and evidence-storage fault controls separate. Include parent termination while a child/descendant remains active and no-effect/effect-before-failure variants under the same failing process result.

The production profile expects administratively protected files and a nonzero executor UID. Positive administrative metadata in unprivileged fixtures is explicitly injected through test-only Rust adapters. Ship no bypass flag, environment switch, permissive profile or fixture authority. The concrete driver/Cargo linkage may be selected within the 34 reservations, but it must exercise shared production code rather than a separately implemented recovery algorithm.

Real filesystem/process adapters and the production TLS transport also receive separate positive and negative tests: regular-file/symlink and path/inode/mode checks; exact hashing and no-replacement publication; real directories/claims and restart; bounded child IO, argv/environment, timeout/kill/reap; loopback Rust TLS with CA/hostname mismatch, redirect/proxy restrictions, bounded/malformed/error responses and finite request selection. The shipped ess binary receives actual parse/help, local dry-run, invalid-input and untrusted/missing-authority controls. Injection of administrative UID metadata is not proof of root provisioning; injected crash persistence is not hardware power-loss proof; synthetic authenticated API responses are not a real-cluster identity; compatible fake Helm output is not proof of stock Helm semantics.

Do not serialize kubeconfig credentials or raw Secret data into logs, evidence, observations or fixtures. Retain metadata-only release-storage handling and secret-redaction controls. Observation proves only the covered direct-resource state at its point in time. The caller's registry completeness, unique uncloned host, cooperative writers, immutable executable installation, approved object fingerprints, storage semantics and independently established quiescence remain explicit trust inputs.

## Required R01–R29 matrix
Use a fixed fixture containing three desired releases and three baseline-only retirements, with a nontrivial rollout order. For every indexed boundary, inject at each applicable apply and removal index. Also run dedicated empty/equal, first-create, all-absent and changed-inventory fixtures. These are families with multiple concrete vectors, not a claim that 29 tests suffice (`docs/design/review-execution-recovery.md:471–536` at 95ef; original-design lines 64–100 remain historical).

| Family | Required executable vectors and assertions |
|---|---|
| **R01** | Invalid desired/current JSON and YAML, duplicate keys and invalid later releases. No external call, cache change, recovery reservation or claim. Preserve existing whole-plan controls. |
| **R02** | Wrong/unresolved target or principal, context aliases/remapping, unrelated baseline, duplicate release address and registry/store/object aliases. Reject before acquisition or mutation; an invalid unselected registry entry also rejects selection. |
| **R03** | First/equal/changed plans and removal previews, with differing synthetic target and call traps. Local unverified preview only; cache/evidence/target bytes unchanged. |
| **R04** | Missing execution evidence with unavailable observation on first run and restart. Preserve an independently populated target; never infer empty state. |
| **R05** | Acquisition-directory creation and ORAS spawn failure at every apply index. No Helm mutation at that index; settled prefix remains. |
| **R06** | Fail/interruption during manifest or each referenced blob fetch, including partial output. No chart admission or target progress; restart must reacquire/re-admit. |
| **R07** | Completed fetched output followed by interruption before bounded read, proof assembly or chart preparation. Staged residue is neither a cache hit nor application evidence. |
| **R08** | Wrong/missing/nonregular output; invalid manifest/layer cardinality; descriptor read failure; malformed chart archive or generated-member inventory. No Helm mutation; retain exact completed prefix. |
| **R09** | Fail proof staging write/readback/no-replacement publication boundaries; retain incomplete stages. Restart ignores unpublished stages and preserves existing entries. This tests current proof publication, not the removed archive/checksum pair. |
| **R10** | Corrupt warm framing, original manifest identity, descriptor size/digest, payload and read errors. Refuse without overwrite or silent repair; no Helm mutation. |
| **R11** | Valid warm original-byte proof with ORAS trapped, fresh authorized observation requiring apply. Revalidate and reuse bytes, then perform the required mutation; cache presence does not imply applied state. |
| **R12** | Values serialization/write and each pre-mutation storage failure; cut after values, after `Observed`, and after `Prepared` before launch. Verify preparation/freshness ordering and conservative restart classification. |
| **R13** | Definite Helm apply spawn refusal at every apply index. `NotLaunched` only where absence of launch is established; retain prefix and stop later work. |
| **R14** | Started nonzero, timeout and lost acknowledgement, each with no target effect and effect-before-failure variants. Same indeterminate classification; retained claim; no later apply/removal or fabricated rollback. |
| **R15** | Apply effect and successful return, then fail/cut before durable process outcome or required readback. No overall success; restart observes without automatic replay or invented acknowledgement. |
| **R16** | Cut after complete per-operation evidence and before next action/final output. After required quiescence, fresh matching observation skips duplicate mutation and permits only authorized remaining work. |
| **R17** | Partial multi-release sequence at each index, including middle operation effect then failure. Preserve earlier settled state/evidence; identify unknown release; later releases and all retirements remain untouched. |
| **R18** | Unsupported/wrongly scoped/corrupt published entries, gaps, wrong predecessor hash, duplicate terminal or entries after closure. Refuse and preserve bytes. Separately admit all specified valid incomplete prefixes and ignore unpublished partial stages. |
| **R19** | Valid historical evidence with unavailable fresh observation; monotonic expiry at launch; wrong invocation/operation/phase references. Stale evidence cannot establish no-op or authority. |
| **R20** | Equal desired/baseline bytes with manual target drift between processes. Observe despite equality, refuse implicit repair, then separately test an independently supplied exact `repair_from` revision. |
| **R21** | Effect then lost acknowledgement; restart observes exact desired match. No duplicate mutation or invented historical applied attribution. Without required Q, remain observation-only and mutation-blocked. |
| **R22** | Retirement present, desired apply present, removal flag absent. Reject before both phases and before acquisition/recovery writes that would imply execution began. |
| **R23** | Baseline-only retirement with no desired projection; uninstall spawn refusal at each reverse-order retirement index. Preserve completed prefix and leave later removals untouched. |
| **R24** | Started uninstall failure/timeout/lost acknowledgement with still-present and removed-before-failure variants. Remain uncertain; never infer absence from status or message text. |
| **R25** | Successful uninstall effect followed by evidence/readback failure or process interruption. Restart with retained target and history; no unconditional repeat uninstall or fabricated removal attribution. |
| **R26** | Explicit repeated baseline-only removal with valid owned history and authoritative absence of both Helm storage and every baseline direct object. Complete as observed already absent without another uninstall, subject to required Q. |
| **R27** | Foreign incarnation at old release address; foreign occupant at baseline-only or newly desired object address. Refuse; neither removal flag nor repair authority may adopt/delete it. |
| **R28** | Competing real driver processes, retained claim after parent death, live descendant, changed registry generation/bytes, target/principal/Helm/claim mismatch between observation and launch. No claim theft or subsequent mutation; independently supplied Q is a separate lane. |
| **R29** | Fail final `Completed` staging/readback/publication/directory synchronization after all calls succeed. No complete-success claim; retained valid prefix survives and a later invocation observes instead of replaying all calls. |

Supplement the family table with these finite, named dimensions:

- **Directory durability:** exclusive reservation creation, new-directory sync and parent publication sync fail independently; repeat for every dynamically introduced directory. No claim or mutation before the required barriers.
- **Journal grammar:** reserved directory without `Opened`; `Opened` only; `Observed` without `Prepared`; `Prepared` without disposition; acknowledged disposition without `After`; all per-operation facts without `Completed`; valid `Stopped`/`Completed`; staged partials; corrupt published entries. Omitting `retry_of` never hides relevant history.
- **Inventory changes:** baseline and desired address sets differ. Before apply, newly desired addresses must be absent; after apply, baseline-only addresses must be absent. Retirement admits baseline-only inventory. Retained direct objects/finalizers prevent absence.
- **Generated-chart refusal:** links, traversal, duplicate/extra/missing archive members, mismatched projected bytes, injected template actions, hooks, CRDs, dependencies, post-renderers, cross-namespace objects, `generateName`, keep-policy and unsupported object kinds.
- **Projection fidelity:** authored rendered fields and complete approved live projection, including unknown additional fields; distinguish UID/resourceVersion evidence from the projection digest. Cover Deployment, StatefulSet and Service.
- **Executable admission:** binary/path/hash/version/protocol mismatch, writable/linked installation, absent flags, malformed probe output, output limit and timeout, environment injection and unsupported credential/proxy configuration.
- **Secret containment:** credential or Secret-value sentinels never appear in stdout, stderr, durable evidence, temporary serialized observations or retained test reports.

The cache controls must retain both original-byte transport profiles and their malformed-input, warm-cache, concurrent-winner and private-snapshot assertions. Opaque historical chart fixtures can remain cache-layer vectors; they cannot be relabeled as admissible generated recovery charts. Add genuine projected-chart vectors for the complete recovery lane, and explicitly account for that changed test boundary. The current fake attack client hard-codes old arguments including `--create-namespace` (`support/cache_origin_attack_client.rs:22–35`), so that fixture needs deliberate adaptation.

The supported result remains direct-resource observation at a point in time. Tests must not claim application behavior, descendants, retained PVC/PV cleanup, real admission-controller policy or perpetual convergence.

## Scope

Refreshed by aep-drive:story-scoper 0.8.0 against published ESS 95ef5be70dbce966056d0484b66db7ed836fa406. These are the exact 34 coordinator-accepted write paths. The historical 57e scope and namespace-main refresh are retained. Only the two now-published model paths change confidence from inferred to cited; every other confidence is unchanged.

- crates/edge/ess-cli/src/main.rs — cited.
- crates/edge/ess-cli/src/oci_cache.rs — cited.
- crates/edge/ess-cli/src/lib.rs — inferred.
- crates/edge/ess-cli/src/recovery/mod.rs — inferred.
- crates/edge/ess-cli/src/recovery/model.rs — inferred.
- crates/edge/ess-cli/src/recovery/authority.rs — inferred.
- crates/edge/ess-cli/src/recovery/journal.rs — inferred.
- crates/edge/ess-cli/src/recovery/chart.rs — inferred.
- crates/edge/ess-cli/src/recovery/observe.rs — inferred.
- crates/edge/ess-cli/src/recovery/process.rs — inferred.
- crates/infra/ess-kubernetes/src/lib.rs — cited.
- crates/infra/ess-kubernetes/Cargo.toml — cited.
- crates/infra/ess-kubernetes/src/recovery.rs — inferred.
- crates/infra/ess-kubernetes/tests/recovery_adapter.rs — inferred.
- crates/edge/ess-cli/Cargo.toml — cited.
- Cargo.lock — cited.
- crates/edge/ess-cli/tests/persisted_delivery.rs — cited.
- crates/edge/ess-cli/tests/cache_origin.rs — cited.
- crates/edge/ess-cli/tests/cache_origin_adversary_pass1.rs — cited.
- crates/edge/ess-cli/tests/command_surface.rs — cited.
- crates/edge/ess-cli/tests/support/fake_delivery.rs — cited.
- crates/edge/ess-cli/tests/support/fake_oci.rs — cited.
- crates/edge/ess-cli/tests/support/cache_origin_attack_client.rs — cited.
- crates/edge/ess-cli/tests/execution_recovery.rs — inferred.
- crates/edge/ess-cli/tests/support/recovery_driver.rs — inferred.
- crates/edge/ess-cli/tests/support/fake_recovery.rs — inferred.
- models/execution-recovery/system.yaml — cited; inferred in the historical scope, now present with the exact validated model-v2 bytes.
- models/execution-recovery/domains/execution.yaml — cited; inferred in the historical scope, now present with the exact validated model-v2 bytes.
- docs/design/review-execution-recovery.md — cited.
- website/docs/concepts/component-delivery.md — cited.
- website/docs/reference/cli.md — cited.
- website/docs/status/where-this-stands.md — cited.
- website/docs/status/limitations.md — cited.
- crates/edge/ess-xtask/src/support.rs — cited.

- **Primary surface:** crates/edge/ess-cli — cited; the current finite executor and tests own this outcome.
- **Read-only reuse:** crates/generate/ess-deployment/src/{lib,environment,runtime}.rs; Cargo.toml; AGENTS.md; Taskfile.yml; crates/edge/ess-xtask/src/main.rs — cited; no write reservation or new dependency edge is implied.
- **Confidence:** high for the accepted finite contract and 34 reservations — cited; 51 prior/current evidence paths were compared from namespace-main a45 to 95ef: 47 are unchanged, main.rs changes only browser Web help, the reviewed consolidated binding is published and the two exact model files are added. Historical 57e marker and namespace-main findings remain retained. The supplied graph matches frozen target states and has no implementation story. Root refreshes changes before creation/dispatch if source or store moves.
- **New-module layout confidence:** medium — inferred; concrete internal linkage/dependency choices are bounded implementation work.
- **Would collide with:** CLI command/help, cache consumers/tests, Kubernetes credential edge, Cargo.lock, the four public pages and ess-xtask support.rs — cited.
- **Resource boundary:** one implementation worktree with owned TMPDIR/cache/fault fixtures and a serialized full gate; root alone mutates planning — inferred.

## Validation and completion

The published binding and exact two model files now match the adopted, declaration-validated bytes at 95ef; retain those receipt identities before runtime edits and revalidate any declaration change. During implementation run targeted meaningful vectors, then affected-package formatting/tests/strict Clippy. Integration must run every lane of task check, including support-check, and task site-build. The affected packages are ess-cli, ess-kubernetes and ess-xtask. Preserve generated-byte and dependency-boundary controls. Record the actual runner counts rather than claiming 29 tests.

```bash
cargo fmt --package ess-cli --package ess-kubernetes --package ess-xtask -- --check
cargo test --locked -p ess-cli -p ess-kubernetes -p ess-xtask
cargo clippy --locked -p ess-cli -p ess-kubernetes -p ess-xtask --all-targets -- -D warnings
task check
task site-build
```

These are required future gates, not executed results. Record integrated implementation evidence against the story and use it to discharge obligation:review-execution-recovery-implementation; neither model validation, candidate review, story creation nor an unrelated passing source gate is sufficient. Root alone performs lifecycle/evidence actions through AEP.

## Out of Scope

No new review objective or decomposition beyond this single existing obligation; no continuous controller, additional executor, live deployment, real authority provisioning, cgroup/supervisor infrastructure, automatic Q/claim theft, generic persisted property bag, new intent-format fields, cache redesign, arbitrary Helm support or perpetual-state guarantee. No third review of the unchanged candidate contract. Ordinary implementation verification/review remains required.

## Ambiguities

- **Cited — publication prerequisite satisfied:** the typed model, binding precedence and offline test qualification are published at 95ef with matching declaration receipts. Root still owns story creation, scope registration and any later implementation selection; this refresh does not reopen the accepted semantics.
- **Inferable — implementation choices:** Rust driver linkage, bounded archive/TLS dependency selection and explicit read/allocation bounds belong within the exact scope and existing MSRV/unsafe/dependency rules. Resolve them against the binding and test them; no new persisted model or external authority may be inferred.
- **Inferable — tracking relation:** use delivers for the existing obligation and depends_on only for the verified implemented design story, as declared by the pinned AEP protocol.

## Open Questions

No unresolved stakeholder input or real deployment credential is required to prepare this bounded offline unit. Any newly discovered semantic prerequisite must be named to root before extending the model or scope; it is not silently discharged by this draft.
