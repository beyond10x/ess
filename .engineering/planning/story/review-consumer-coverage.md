---
format: aep.planning-md/1
id: story:review-consumer-coverage
kind: story
status: implemented
title: Require explicit consumer coverage for model extensions
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-semantic-diff-coverage
scope:
- confidence: cited
  path: .github/workflows/ci.yml
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask
- confidence: cited
  path: crates/verify/ess-diff/tests/consumer_coverage_f01.rs
- confidence: cited
  path: docs/design/review-consumer-coverage.md
revision: 24
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

Confirmed implementation scope for wave18. The earlier scope derivation and staged decisions
remain in this story's history and journal. Root read back the final correction and merged
planning evidence against integration source 49732ec39da306e277eaf4f1d820e246e08797cf.

- **Rust checker owner — cited and confirmed:** `crates/edge/ess-xtask`; actual Stage1/Stage2 source
  and both corrections implement closed source/model/wire/consumer extraction, finite classifications,
  explicit accounting and execution of exactly attributed native owner cases. Source: Stage1 report
  scope confirmations, Stage2 report sections1–2, and both correction reports/readbacks.
- **Existing gate sequence — cited and confirmed:** `Taskfile.yml`; root adds exactly one
  consumer-check lane after support-check, preserving every existing gate lane and relative order.
  Its command pins the selected locked/offline1.98.1 provider profile. Root source comparison and
  Task-profile readiness records establish the change; The complete integration result is recorded separately below.
- **Workflow profile — formerly inferred, confirmed:** `.github/workflows/ci.yml`; provision1.98.1
  in the existing Rust setup so the actual provider meets its declared profile. Existing permissions,
  triggers, action identity, components, targets and task-check command are unchanged.
- **Dependency resolution — formerly inferred, confirmed:** `Cargo.lock`; Stage1 adds only five
  ess-xtask dependency edges, with no package/version additions. Stage2 and corrections add none.
  Source: Stage1 report scope confirmations and retained before/after resolution comparison.
- **Accepted internal binding — formerly inferred, confirmed:** `docs/design/review-consumer-coverage.md`;
  root accepted and synchronized the reviewed v3 policy. Historical prose calling this home absent
  describes the earlier proposal state and no longer describes current source.
- **Exact F01 owner cases — formerly conditional/inferred, selected and confirmed:**
  `crates/verify/ess-diff/tests/consumer_coverage_f01.rs`; five cases isolate relation kind, target
  and carrier and view-parameter order/type, each with unchanged control, exact typed delta and
  current-format roundtrip. Source: Stage2 report sections1–2 and original direct owner-case results.
- **Read dependencies — cited:** RawSpecFile, Specification, EssIr/private EssIrParts,
  EssSemanticRef/SemanticDependencyGraph, reachable authoritative declarations and separate compiled
  RawSpecFile Draft7 wire graph; current inventoried consumers and attributed tests. No production
  model or consumer semantic repair was selected by this checker story.
- **Finite initial unknowns — selected and measured:** exactly157068 initial model/profile pairs
  remain pinned by baseline e005a2e74e067ad51315e594151643b702381dc174bb9af5c355c3eeeb17ad51.
  Qualification belongs to separate draft epic:qualify-initial-consumer-baseline. Existing unknowns
  do not establish support/refusal; new identities require explicit reviewed behavior evidence.
- **Validation boundary — cited:** both independent source attacks, original and corrected cases,
  preserved same-source production mutations/restored green, exact attributed consumer execution,
  and every repository gate lane plus site-build. The root verifies correction2 after the second
  attack; no third attack is assigned. Full integration status comes from its own actual records.
- **Collisions and exclusions — coordinator inference grounded in actual writes:** reserve the six
  machine scope entries above. General fuzzing and the known Go panic remain with their existing
  story; broad initial qualification, downstream delivery and source release are outside this wave.

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

### Source pass2 routed to correction2 — 2026-09-08

The immutable review-result:consumer-coverage-source-pass2 records two introduced NEEDS-CHANGE
findings: concrete default trait callable inventory and opaque signature type-macro associated
contract traversal. Both cases failed on first exact execution. Affected package executed103,
101passed/2failed/0ignored; all99 prior cases and both new controls passed. Complete report
SHA25613d2815e847af96c566c02a88d234b841a63a278f553b75ac59fd28c32296e76 and final seal
SHA2562ac8c947b12bf462587145ebecb8fa1930b9ce9be43287d2fb13cf54ba17b65b remain unchanged.
Root independently verified6906native entries/9650245277regular bytes, complete1214-file
source archive and every original test stream; reader actual0 in17.280772335943766seconds,
preparation/consumer-source-pass2-readback/readback.json SHA256
 de91a6ad56d5db78a60a3a1a7caf08ef5a6155e8cd2b920ebf5f1c937cdd813f.

AEP's actual comparison is0carried/2new/2resolved, with the full original output below. Findings
remain2→2 while all first-pass signatures are resolved. The same implementor receives this new
ground under preparation/consumer-correction-2-work-order.md; root will verify the correction
and assertions. No third full source attack is assigned. No second-pass outcome is marked fixed
before the correction lands. Source/integration/publication remain held until green; source
eligibility and the e005 initial baseline remain fixed, and no current production incident is
claimed from the constructed macro fixture. Agent judgement remains a review.

```json
{
  "artifact": "story:review-consumer-coverage",
  "reviews": 12,
  "from": "review-result:consumer-coverage-source-pass1",
  "from_reviewer": "unattributed",
  "to": "review-result:consumer-coverage-source-pass2",
  "to_reviewer": "unattributed",
  "carried": [],
  "new": [
    {
      "file": "crates/edge/ess-xtask/src/consumer_coverage/consumer.rs",
      "line": 525,
      "category": "acceptance",
      "severity": "blocker",
      "verdict": "NEEDS-CHANGE",
      "origin": "introduced",
      "message": "A newly added public default trait method keeps the old finite consumer classification set because trait methods are skipped as concrete API entries."
    },
    {
      "file": "crates/edge/ess-xtask/src/consumer_coverage/consumer.rs",
      "line": 587,
      "category": "contract-drift",
      "severity": "blocker",
      "verdict": "NEEDS-CHANGE",
      "origin": "introduced",
      "message": "An admitted type macro in a callable signature hides its Self-associated dependency, so changing that associated type can preserve the callable declaration fingerprint."
    }
  ],
  "resolved": [
    {
      "file": "crates/edge/ess-xtask/src/consumer_coverage/rust.rs",
      "line": 445,
      "category": "contract-drift",
      "severity": "blocker",
      "verdict": "NEEDS-CHANGE",
      "origin": "introduced",
      "message": "The model resolver discards absolute path qualification in types and imports, so a local module can replace the actual external type owner in the authoritative model graph."
    },
    {
      "file": "crates/edge/ess-xtask/src/consumer_coverage/consumer.rs",
      "line": 471,
      "category": "acceptance",
      "severity": "blocker",
      "verdict": "NEEDS-CHANGE",
      "origin": "introduced",
      "message": "Implementation scanning omits associated constants and types, allowing a new public constant to retain old finite classifications and an associated result type to escape its bound callable fingerprint."
    }
  ]
}
```

## Correction2 bounded verification and preserved historical records — 2026-09-08

Root read the complete correction2 source diff, all four original pass2 cases and seven new controls,
all42 finite classification decisions, and the five initial correction-control failures. Independent
readback verified all1214 current source files, eight actual lanes and retained native images,
all103 prior assertions (one repeated blank line removed by rustfmt), and all7671 old classifications.
It reconstructed all93 profile fingerprints from both actual inventories, compared55 selected
entries, nine parent trait hashes, Rust/wire/provider schema and the e005 baseline. No profile or
baseline eligibility changed. Receipt preparation/consumer-correction2-bounded-root-readback.json
SHA256 b21e577f34e4eea3f30ebb7a6faeac75e137730da429b542ed6a41880d4986eb; actual0.

The original two new red cases each now pass alone,1passed/0failed/0ignored/97filtered.
Seven additional controls first ran2passed/5failed, then7passed/0failed/98filtered.
The affected package ran110passed/0failed/0ignored (105main+5layout), strictClippy and fmt returned0.
The one production discovery returned1 for the named unclassified Identity::entity_identity trait
callable before finite classification. Final delta:40 concrete callable entries across nine existing
traits plus trait_associated_contract and AssociatedNames::visit_macro;42additions,zero removals.
The implementation retains parent-trait hashes, required/default/cfg boundaries, selected same-trait
associated contracts and explicit opaque-macro refusal in signatures, headers and selected declarations.
Ordinary bodies and unselected associated declarations stay outside callable eligibility identity.

After all assigned producers held, root archived, independently verified and retired only the four
historical coordinator target/review-boundaries-{7,8,9,10} record roots. All12 actual commands returned0.
Original successes, failures, raw streams, source snapshots and native metadata/payloads remain in
/home/timo/.cache/ess-review/2026-09-06-resume/waves7-10-historical-records-retirement.
Complete receipt SHA256 d71c6c53f60a0c5d5a1241da577eddf610aabae18339b1ccdd4ddf92d8c443ad.
No managed source checkout, current unit target/TMP, frozen tool or Git branch was removed.

One literal Task consumer-check is selected under the separate grant
preparation/consumer-correction2-full-checker-grant-01.json, SHA256
e6d1a1182d6b749cfac4f42014567312116a224c098ac4aca11696aba90c22a4.
Measured aggregate9808551936bytes; cap10882293760 (+1GiB including completion records),
free7618129920bytes,6GiBreserve; one heavy producer/two Rust jobs. Actual complete checker,
final source/native seal, root final verification, source commit, integration and publication remain
unclaimed at this bounded checkpoint. No third source attack or downstream work is selected.

### Final correction2 source acceptance — 2026-09-08

Root accepts bot commit af8ae3b95317ef351e8e4251d883d818c6e71b0f after both source attacks
and independent correction verification. It changes only consumer.rs, tests.rs and finite
entry-classifications.json. Both author and committer are the organization bot. All1214
measured source bytes are unchanged by the commit; the unit is clean and remains unpublished.

The literal /usr/local/bin/task consumer-check returned0 in589.4778127779718seconds:
72successful direct commands,22exact nonignored cases,1806models,87behavioral profiles,
54Supported/0Refused/157068BaselineUnknown. All157122 cell records equal the preceding
accepted actual output; all93profiles,55selected entries, nine parent traits and Rust/wire/schema
inventories remain unchanged. The original two source-pass2 cases and all seven correction
controls now pass, with every prior assertion preserved. Package110, strictClippy and fmt are green.

Report unit target/review-boundaries-18/consumer-correction-2/report.md is865577bytes,
SHA256 e47ac9271830ee2610096c5a166e09d551c853a3a3b30de02e4951db7396f69e.
Final seal SHA256 3e4c5eec9bf7758f8bc8638865f254e4f0d7ba777295b1cd468c22332fe92b18;
source manifest SHA256 9dd9226b883067655511071402ed2c39d2467b6cfea614bcd3d06728c2d79c99;
complete source archive SHA256 143ea90b7fa38052846f1b430ffa7c130e01c09ddcec463cd652207888a6a64a.
Native census/readback both0:7676entries,10741693120regular bytes. The implementor released
its own lease after quiescence; root leases remain. Final allocation10609344512 was below cap.

Root read all report prose, the complete source diff, original/new test assertions and exact42
classification decisions. Independent reader actual0 in23.021481973002665seconds verified every
current native payload/recorded metadata/path, all final pins, the unique1214-file archive,
all72 actual checker streams/cases and all157122 cell records. Receipt
preparation/consumer-correction2-readback/readback.json SHA256
48c9963408791c3c068e0245251b8ab692d7d6a545558e584d7f54672ca1bb23.
No assertion was removed or relaxed; the only prior-test text change is one repeated blank line
removed by rustfmt. There is no third source attack.

Two bookkeeping discrepancies remain explicit. A preflight Task-image copy returned1 for an
absent scratch directory, and the already-authorized checker launched before that result was
inspected. The exact still-live Task image was independently captured50.68899154663086seconds
after launch, matching the pinned bytes; no checker rerun occurred. A later read-only forecast
incorrectly counted retained copies twice and returned1, while actual allocation remained within
cap. Its original tool output was truncated and the complete traceback is unavailable; no full
forecast-stream claim is made. All product-command streams remain complete and actual green.

Source pass2 outcome is now fixed by the exact accepted source commit and root verification.
Full combined integration, site-build, publication and worktree cleanup remain owed. The prepared
warm runner v3 SHA256319bbdc52cca2627835bdbf4d4c7aa023332e2cd57ad682a3c038ba098eb5b9d
adds exact Task retention before gate lanes and preserves the v2 source. No branch handoff or gate
has run. Its8GiBreserve is unchanged. Agent cost counters were not supplied by this harness;
no token/tool-use values are invented. No tag, release, deployment or downstream delivery is selected.

## Complete consumer integration — 2026-09-08

All ten `task check` lanes, `task site-build` and planning validation passed at
`49732ec39da306e277eaf4f1d820e246e08797cf`. Full test: 2,311 passed, zero failed/ignored, 199 summaries;
all 30 browser cases and 40 synthesis feasibility cases included. Consumer-check ran 72 successful direct
commands and 22 exact native cases: 54 Supported, 0 Refused and 157,068 BaselineUnknown, with all 157,122
cell records identical to accepted correction2. Root reverified all 1,215 source files, original
streams, exact case results and seven native checker images with separate retained root copies.

[Complete integration record](../../../docs/reviews/2026-09-08-consumer-coverage-integration.md)
retains every lane's actual result, both corrected source attacks, all interrupted or refused
runner attempts, the prospective capacity revision, exact archive receipts and the limits of
these claims. Root readback SHA256 `6c62998a3c951d26d6df63efdb6837d803bde1cad2240ce3f239c47a543fbfa5`. Original reports and initial eligibility remain
unchanged. The final Scope confirms all six write reservations, including formerly inferred paths.
Publication and cleanup follow under the standing approval; no release or downstream work is selected.
