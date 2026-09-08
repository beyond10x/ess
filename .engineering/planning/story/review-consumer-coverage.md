---
format: aep.planning-md/1
id: story:review-consumer-coverage
kind: story
status: active
title: Require explicit consumer coverage for model extensions
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-semantic-diff-coverage
scope:
- confidence: inferred
  path: .github/workflows/ci.yml
- confidence: inferred
  path: Cargo.lock
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask
- confidence: inferred
  path: crates/verify/ess-diff/tests/consumer_coverage_f01.rs
- confidence: inferred
  path: docs/design/review-consumer-coverage.md
revision: 18
---
## Finding and source

F16 (P1) from `docs/reviews/2026-09-05-architecture-review.md:522`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `Taskfile.yml:138`, `crates/edge/ess-xtask/src/main.rs:54`, `docs/reviews/2026-09-05-architecture-review.md:532`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

The extension gate fails when a new semantic construct or field has no tested support or explicit refusal for an inventoried consumer.

## Implementation boundary

Maintain a typed consumer matrix for validation, IR, references, diff/impact, projections, synthesis and conformance, with links to executable cases or explicit unsupported/refusal evidence. Add a Rust xtask gate that checks the matrix against the actual authoritative model surface; avoid a parallel hand-maintained list silently omitting fields. A coverage record is not a substitute for running its behavioral test.

## Validation

Mutation proof: add a representative semantic field/consumer obligation without coverage and observe failure, then supply supported/refused behavior evidence and pass. Include concrete F01 rows. The existing fuzz story owns general document fuzzing rather than this matrix.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Do not rewrite passing gates or claim every target supports every construct; unknown is a valid visible matrix entry with owned follow-up work.

## Scope

Derived 2026-09-07 by `aep-drive:story-scoper` 0.8.1 against ESS `a0cf3ca8681ce06f6fbdbc988d457b23f2136c04` and draft story revision 9 — cited.

- **Primary write reservation:** `crates/edge/ess-xtask` — cited; the story names this Rust gate owner, whose dispatcher and actual compiled RawSpecFile schema provider remain the implementation home.
- **Gate write reservation:** `Taskfile.yml` — cited; add the separately named consumer lane while preserving every existing check and its order, including support-check.
- **Dependency write reservation:** `Cargo.lock` — inferred; a measured package-local extractor dependency change may require updated resolution.
- **Binding write reservation:** `docs/design/review-consumer-coverage.md` — inferred; the intended internal design home is still absent, and the reviewed v3 candidate remains unaccepted preparation.
- **Model boundary:** RawSpecFile, Specification, EssIr, private EssIrParts, EssSemanticRef and SemanticDependencyGraph, their reachable production declarations, and the separate compiled RawSpecFile Draft 7 wire inventory — cited; the three authoritative ESS model package trees are byte-unchanged from the preceding a45b4081 source checkpoint.
- **First-stage boundary:** implement closed source/profile/model/wire/consumer extraction, exact candidate case attribution and finite unaccepted baseline output, then stop at a root checkpoint before any BaselineUnknown admission — cited; the preserved v3 policy and finite first-stage work order explicitly require this sequence.
- **Authored acquisition refresh:** classify specification, authored-scenario and coverage acquisition separately across immediate manifest selection, legacy directory policies, direct files and omitted scenarios; retain original input identities and bytes, inactive-role boundaries and refusal-before-output behavior — cited; the newly integrated shared acquisition owner changes these consumer profiles without adding another semantic model root.
- **Model-caller refresh:** include release report qualification and observed-binding comparison among the explicit callers of shared specification loading; distinguish admitted model/selection consistency, declared realization identity and qualified infrastructure comparison from execution or producer authentication — cited.
- **Other integrated consumer refresh:** retain the previous coverage, cache and support classifications; refresh current legacy and paired browser playback, realization v1/v2 alternatives, namespace collection entry points and release-action/build inputs — cited; these source and case identities changed after a45b4081 and cannot inherit an old package-level claim.
- **Behavioral boundary:** Supported or Refused requires exact attributed assertions and actual case execution under the bound profile; browser unknown-state presentation, release consistency checking, fixture presence and an aggregate green suite do not establish stronger semantic behavior — cited.
- **Conditional additional writes:** propose an exact owner-test reservation only when a mandatory behavioral cell cannot be established by existing meaningful assertions and observable execution — inferred; this refresh selects no additional test owner or consumer repair.
- **Validation boundary:** retain package checks, the complete repository gate and site-build for the validation-workflow change; retain mandatory F01 attribution and same-source causal mutation requirements — cited.
- **Confidence:** medium — inferred; current write owners and relevant source deltas are established, while complete extracted inventories, exact case/profile attribution and finite baseline eligibility remain first-stage outputs.
- **Would collide with:** edits to the xtask package, Taskfile validation sequence, dependency lockfile or exact internal binding document — inferred; model, consumer and case owners remain read dependencies whose changes require refreshed inventory and evidence checkpoints.

## Candidate binding review

The coordinator accepted the four refreshed write reservations on 2026-09-06 and applied only
this Scope replacement; the story remains draft. The independent report is
target/review-boundaries-11/next-scope/consumer-coverage-report.md, SHA256
2194fea88d194d9c54b22e38f5990d6e2b00b64ed633ac039ebf833ca29abd61. Root independently
verified all 53 inputs and 49 opening Git blobs. Two document-only reviews are recorded as
review-result:consumer-coverage-binding-pass1 and review-result:consumer-coverage-binding-pass2;
verification-report:consumer-coverage-binding-wording records the narrow correction to actual
Draft7 definitions/#/definitions/ vocabulary. The current candidate is
target/review-boundaries-11/next-scope/consumer-binding-draft-v3.md, SHA256
63d6075781dc7f348fff838962c4392df3c8e9335eaf7b5d06f8a690f630812c. Its two-stage baseline
selection, exact wire/model inventory and actual attributed case execution remain requirements,
not measured gate results. Before implementation selection, refresh integrated coverage source
and case/profile identities, accept the binding and select the concrete first-stage work order.
No new story selection or baseline-unknown eligibility approval is implied by this scope update.

## Coverage-integrated scope refresh

The original independent report is retained at
`target/review-boundaries-12/preparation/scopers/consumer-report.md`, SHA256
`76b42814f61e3d7603e5e934b38f82ed9dda518478d60e6b1d46b636caccf7b1`. Root verified all 65
read inputs and 59 frozen Git blobs before applying this Scope. The same four machine
reservations remain. Its proposed first-stage profiles include coverage construction, original
input and lineage admission, paired replay, actual Rust/Go execution and coverage impact.
This source inspection does not accept the binding or any BaselineUnknown eligibility.
The story remains draft and is not selected by wave 12. Later selection must refresh any
intervening model, consumer, test or profile changes, including the pending cache implementation.

## Selected implementation policy and Stage 1

On 2026-09-07 root accepts the unchanged reviewed v3 policy under the standing implementation
approval and selects the finite first stage of wave 18. The accepted home is
`docs/design/review-consumer-coverage.md`; its current assignment names the a0cf3ca source,
production model/default-feature profile, frozen Rust authority and the mandatory later root
eligibility checkpoint. The complete Stage 1 work order is retained in
`docs/plan/2026-09-07-review-boundaries-18.md`, with the current acquisition, browser, release,
observed-binding, realization and namespace classifications added to its prior input inventory.

The independent scoper returned the same four reservations. Root verified all 44 input hashes
and 38 current Git blobs; receipt SHA256
`6f1aea6b8d7cbe240acbc8630b449d33223c14c66aad2c94c75effbab438f73a` at
`target/review-boundaries-18/preparation/scoper-root-readback.json`.
The complete prior acceptance, validation, compatibility and review history are preserved.
The source/output checkpoint must be measured and reviewed before any baseline unknown is admitted;
no such eligibility, runtime case result or story completion is claimed by this selection.

## Stage 1 checkpoint and selected enforcement scope

Root inspected the complete Stage 1 source/output checkpoint at local bot commit
7a6d76855e28d280138d2d439eb6f1e7c15a58d9. Package verification passed 71 cases,
strict Clippy and formatting; two final same-provider extractions match all 13 files.
Root independently checked all 157,122 cells, 54 mandatory exclusions, closed 148-declaration
Rust graph, complete source/provider stamps, preserved native images and the complete
3,623-entry target/TMP census. Full Stage 1 report SHA256
19ac6808b91b660d91b6bebc121cb05b3b0021e440309a0e780e69e2d071d459;
seal2 SHA256 3da17b0a507016832c41b3116f12e30900970120b5cab1a5a4a8f5fc80341ffc;
root final readback SHA256 05d78f21c2d8c1c16ccdcdb936b9d862670a29c895c36c7e4f6c30d6b1d25985.
This is an extractor checkpoint, not behavioral support or story completion.

Root accepts only the finite 157,068 initial unknown pairs in
crates/edge/ess-xtask/src/consumer_coverage/initial-baseline.json, SHA256
e005a2e74e067ad51315e594151643b702381dc174bb9af5c355c3eeeb17ad51.
Each exact group has its package owner, profile/model shape pins and bounded unproven
statement. The independently owned epic:qualify-initial-consumer-baseline remains draft;
its qualification work is outside the original 31 remediation stories. The current gate
story cannot own its own unknown follow-up. The known Go panic remains broken and its
concrete repair stays with story:fuzz-the-specification-surface.

The final conditional owner-test reservation is now selected and recorded as inferred:
crates/verify/ess-diff/tests/consumer_coverage_f01.rs. It contains only the five isolated
relation kind/target/carrier and view-parameter order/type comparison witnesses. Root's
prepared six authored fixtures passed actual retained-baseline CLI validation; the Rust
cases remain uncompiled and unexecuted preparation. The existing xtask, Taskfile, lockfile
and binding reservations remain. No model or production consumer repair is selected.
Cargo.lock scope is now confirmed by five package-local dependency edges with no version
or package additions; binding ownership is confirmed by its actual reviewed file.

Stage 2 implements closed accounting and exact actual-case execution, completes those
mandatory witnesses and the production causal mutations, then returns for independent
source adversary review and the entire repository gate plus site-build. Root owns shared
Taskfile/design/planning edits, accepted eligibility, Git, integration and publication.
Stage 1 alone does not satisfy the acceptance statement or move this story to implemented.

## Enforcement workflow profile reservation

Coordinator source inspection on 2026-09-07 found that
crates/edge/ess-xtask/src/consumer_coverage/mod.rs::validate_build admits the selected
Rust/Cargo 1.98.1 x86_64 Linux debug profile, while .github/workflows/ci.yml:44
currently provisions the moving stable toolchain before task check at line 67.
Root therefore adds the exact .github/workflows/ci.yml write reservation as inferred:
its existing Rust setup step must provision 1.98.1 explicitly. This is an integration
requirement derived from the accepted profile, not a change to the coverage policy.
The existing action identity, components, targets, permissions, triggers and gate
command remain intact. Root owns the edit; the implementor's source reservation is
unchanged. The separately named Taskfile consumer-check lane will declare the selected
build settings and all existing gate lanes retain their relative order. No downstream
publication, deployment or release authority is added.

## Source review correction 1

The first source attack is recorded verbatim as
review-result:consumer-coverage-source-pass1 against b37572e410a7b4a4d18e2abb4fd99ed0db5401f5
plus four additive tests. It found two introduced NEEDS-CHANGE defects: loss of absolute
external qualification in model type/import resolution, and omitted associated consumer
constants/types. All four cases failed on first actual execution; the package then ran
92 cases, with its original 88 passing and the four added cases failing.

Root independently read the complete report, original five command streams/direct results,
complete current 1,213-file source archive and 5,822-entry native census (8,233,562,641 regular
bytes). Report SHA256 6564e6d04d8194a98fdf09d895596e4fac8f5f4c3f566e3fb12963f133ce0263;
root readback SHA256 cd3ea02bcdc4881d06fe7a47daef4e67fc3b5b9d9cad5788f82cc65ba337d5f8.

Both defects return to the existing implementor within the current xtask reservation.
Preserve every original/adversarial assertion; explicitly account associated declarations
and keep absolute external authority separate from local shadowing. The associated-output
case is a constructed future bound profile, not an observed transfer of a currently
eligible production pair. Correct that declared-contract mechanism without claiming a
current baseline incident. Exact newly discovered member classifications require reading
their actual owners. Initial baseline e005 remains read-only and may not expand or be
regenerated. Record any actual current model/profile identity change before proceeding.

Package/Clippy/format verification and matching actual 22-case consumer execution must be
renewed on the corrected source. The original downstream causal probes remain retained
witnesses for unchanged semantics; no mechanical rerun is selected. At most one additional
full source attack remains. Root owns shared files, all AEP/Git operations, integration,
publication and cleanup. This is correction of the original story, not another story.

### Source pass1 correction and final attack handoff — 2026-09-08

The two introduced findings in review-result:consumer-coverage-source-pass1 were addressed
by bot commit c375e35def175b51a61c259e73b3b00749399539. The original four first-red cases
remain unchanged and each now passes. Seven additive correction cases cover external-owner
resolution, associated declarations/contracts and unresolved nested Self projections. No
assertion was dropped; root independently checked the original complete tests as an unchanged
prefix. The final affected package executed99 cases with99passed/0failed/0ignored, and strict
Clippy and formatting returned0. The original intermediate control failures remain retained.

The literal task consumer-check returned0 in592.2704340390628seconds:72 direct commands,
22 exact attributed cases,1806models,87behavioral profiles,54Supported/0Refused/157068
BaselineUnknown. All93 existing profiles, all157122 cell records and exact Rust/wire/provider
schema JSON equal the prior accepted extraction. The baseline remains e005a2e74e067ad51315e594151643b702381dc174bb9af5c355c3eeeb17ad51; no unknown was silently qualified.

The additional concrete classification inventory is125 associated declarations plus10 extractor
helpers. Their finite reviewed classifications are114OwnedHelper,7DiagnosticSurface,
8FixtureRealization,6ForeignContext. The removed conditions_fn helper is the sole stale row
retired. The implementor read43 non-xtask and2 xtask owner files. These are extractor coverage
changes inside the assigned surface, with no product model or dependency changes.

Implementor report: unit target/review-boundaries-18/consumer-correction-1/report.md, SHA256
0b55a51528c2d791dadf4757ad1cdb3c55c893579e423f028deb861d95b318e7. Final seal SHA256
4fd41234f70f42e552dcd77dd35ad69a7070bad2a84483b357b167cd6ba1bdad. Root readback:
coordinator preparation/consumer-correction1-readback/readback.json, SHA256
c74974609e6953fde944a9509e1ab98e61228ee3056bcf5d45a0bd766906ea99, actual0 in
19.951905607944354seconds. It checks all6720native entries/9338655895regular bytes,49 final
payload pins, every complete source archive file, all raw command streams and exact cell sets.
Agent judgement remains a review; it is not independent verifier evidence.

The second and final source attack is now assigned to the same adversary under
preparation/consumer-source-pass-2-work-order.md, SHA256
b49624de2c70ea5ffe29eefcc9cc1bdc9bd9fe131bd863044fc23f0a8a104146, and fresh execution
grant33e3dddd93cd14ab6c0f92bee088c834904399bcad5156f8d8d8c26868c36375. Its subject is
the complete unit diff against published a0cf3ca8681ce06f6fbdbc988d457b23f2136c04. No
second-pass result or final integration success is claimed yet. Story remains active; source
publication, required CI and owned cleanup remain owed. No release or downstream delivery is selected.
