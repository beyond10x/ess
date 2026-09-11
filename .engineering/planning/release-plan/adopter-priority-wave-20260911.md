---
format: aep.planning-md/1
id: release-plan:adopter-priority-wave-20260911
kind: release-plan
status: active
title: Integrate adopter priority fixes and release ESS
relations:
- serves: vision:O2
revision: 2
---
## Authorization and scope

Skill version 0.8.1 — aep-drive:wave. The operator said to review the ask, use the wave skill with sub-agents, and cut a new release after main integration; the subsequent instruction excludes crosswalk. This is explicit interactive pre-approval for the selected work, commits, merges, publication and the later source release. No repeated approval is needed for those actions. Flutter stays optional and last; the broader ESS evolution and Connectors adoption goal remains active.

This page coordinates story:docs-literal-mapping-claims-unchecked, story:binding-mapping-bounded-accessor and story:output-ownership-tolerates-platform-xattrs, all serving vision:O2. Crosswalk is excluded. The xattr implementation already landed in 391b2651; this unit reconciles tracking and measures existing targeted tests. Literal docs precede accessor implementation because their renderer and IR files collide. Accessor design can proceed concurrently in its separate design file; implementation waits for the docs merge.

Authorized commits: the opening plan/import commit, source-unit commits and corrections, merges to this integration branch, closing evidence/store commit, merge to main, then the separately requested version/release commits and annotated version tag. No deployment, Atlas/Website release or crosswalk work is authorized by this wave.

## Computed selection

The following is the unfiltered output of aep plan artifact waves --kind story --status proposed --format json. The unassessed delivery story is outside the operator-selected set and is not a candidate in this wave; no readiness is inferred for it. Scope is cited except the new accessor design path. Scopers ran read-only before selection.

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:docs-literal-mapping-claims-unchecked",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/docs.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/tests/corpus/billing/docs/interactions.md"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/tests/corpus/oracle-fixture/docs/interactions.md"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/tests/docs.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/ir.rs"
            }
          ]
        },
        {
          "id": "story:output-ownership-tolerates-platform-xattrs",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": ".engineering/planning/story/output-ownership-tolerates-platform-xattrs.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 2,
      "artifacts": [
        {
          "id": "story:binding-mapping-bounded-accessor",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "consumer"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/asyncapi.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/docs.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/graph.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/openapi.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth/src/go/system.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth/src/plan.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth/src/rust/feasibility.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth/src/rust/system.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/ir.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/resolve.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/binding.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/runner.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/scenario.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/synthesize.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-diff/src/diff.rs"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/binding-mapping-bounded-accessor.md"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [
    {
      "a": "story:binding-mapping-bounded-accessor",
      "b": "story:docs-literal-mapping-claims-unchecked",
      "path": "crates/generate/ess-gen/src/docs.rs",
      "confidence": "cited"
    },
    {
      "a": "story:binding-mapping-bounded-accessor",
      "b": "story:docs-literal-mapping-claims-unchecked",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "cited"
    }
  ],
  "unassessed": [
    "story:binding-delivery-at-most-once"
  ],
  "cycles": []
}
```

## Pre-flight and deviations

Base: exact remote main 6b666e58f2e87dd8798d27f935e9a012203296a3; fresh remote advertisement verified. Integration: feat/adopter-priority-wave, managed id wt-05b595184ac8. Worktree path is worktree-state:trees/b10x/ess/wt-05b595184ac8 under the operator's local managed-state root. Coordinator lease: ess-priority-wave-01a089ee.

Primary checkout remains untouched with this observed status:

```
 M .engineering/planning/journal.jsonl
?? .engineering/planning/story/binding-mapping-bounded-accessor.md
?? .engineering/planning/story/crosswalk-verb-external-names-held-to-declarations.md
?? .engineering/planning/story/docs-literal-mapping-claims-unchecked.md
?? .engineering/planning/story/output-ownership-tolerates-platform-xattrs.md
```

The user explicitly asked to handle these existing drafts. Their exact source snapshots and hashes are retained; imports here use the AEP CLI. This clean exact-main checkout avoids changing another session's primary. Existing ESS evolution tree wt-20021617fa84 holds wanted local-only commits behind its recorded scanner-capacity blocker, and other sessions' trees are preserved. Those are accounted-for active work, not abandoned wave output. The evolution build target was cleaned before this wave (353.4 MiB reclaimed).

Measured available storage: /tmp tmpfs 10 GiB; disk 97 GiB. Prior targeted ESS adapter build measured 353.4 MiB. Allow at most two active builds, two jobs per build, unique /tmp/ess-priority-wave/<unit>-target, debug=0 and incremental=false; preserve a 3 GiB /tmp floor and inspect before broadening. sccache 0.17.0 is available with a 10 GiB bound and 1014 MiB occupied; enable it for ordinary native builds, honor any repository-specific compiler wrapper exclusions. User goal has no token budget; harness permits root plus three active agents. No prior wave build directory is reused.

Dispatch roles: aep-drive:story-scoper, aep-drive:implementor, aep-drive:adversary. The harness has no subagent_type selector; generic collaboration agents receive the exact installed charter and a durable unit brief. This is a reported harness deviation, not a claim of plugin-native dispatch. Root owns every planning write, Git commit and publication. Read-only scopers completed for all three selected stories.

The operator's no-full/ownership-gate instruction supersedes the skill's full-gate default. No repeated unchanged tests. Current main rules require common Security and privacy, Gate, and both macOS ownership jobs. The release workflow also requires its exact-commit reusable gate, WASM check and four native archives. Do all authorized implementation and targeted verification first; do not bypass these remote requirements or report release completion without them. Connectors operation search currently refuses connector-unreachable; use previously authorized Git and standalone bot delivery fallback.

## Unit ledger

Opening commit: 59afcf8caec5230a88aadfd7c590a703e0b5500b, signed common evidence passed and feature branch published. Coordinator integration: wt-05b595184ac8 on feat/adopter-priority-wave, lease ess-priority-wave-01a089ee; planning updates remain coordinator-owned.

- Literal docs: impl/priority-literal-docs, corrected candidate eb71c85d634c8e252a7ac380843a87c26aefcba3 merged in 55c2b107, wt-99a063fb0aaf, worktree-state:trees/b10x/ess/wt-99a063fb0aaf. Base 59afcf8c. Build /tmp/ess-priority-wave/literal-target; scratch local-evidence:ess-evolution-20260910/priority-wave/literal. Implementor scope_literal ended lease after corrected 222 generator tests (40 docs), eight literal-validation tests and Clippy/format checks. Second adversary pass by scope_xattrs found nothing after eight deciding cases; both review records and outcomes are in AEP. Stage: source merged into wave and published at b3f2cb74, story implemented, managed tree removed after exact reviewed GC; build cleanup reclaimed 628.1 MiB. Adversary lease ended. No full local or ownership gate ran.
- Xattrs: impl/priority-xattr-verification, base/head 59afcf8c; wt-6f0fa901447d removed by exact reviewed worktree GC after target cleanup reclaimed638.1MiB. Report and original logs retained in local-evidence:ess-evolution-20260910/priority-wave/xattrs. Story implemented on its revised exact-label verification contract; no new source change. Branch may be removed after final wave integration/recovery audit.
- Bounded accessor: impl/priority-bounded-accessor, base59afcf8c, wt-30254233b1b4, worktree-state:trees/b10x/ess/wt-30254233b1b4. Build /tmp/ess-priority-wave/accessor-target (not yet used); scratch local-evidence:ess-evolution-20260910/priority-wave/accessor. Implementor scope_accessor owns source implementation, lease ess-priority-accessor-01a089ee. Docs source is integrated. Two design attacks found three then one issues; root verified the final correction directly and admitted design fa2d234c for implementation, with no third full design attack. Consumer feasibility review by scope_xattrs confirmed additional identity/conversion obligations; preserve those requirements and exact evidence, not invented wire fields.

The unit trees were created before observing the first import commit refusal; no agent edited them until they were advanced to the admitted opening commit. The rejected transaction remains a private recovery patch. User subsequently instructed the coordinator to unblock itself and requested another origin/main pull: fresh fetch still resolves main to6b666e58, already included here. Primary journal changes overlap incoming commits, so the primary draft state remains preserved. Current development uses exact incoming main; no stale source is used merely because primary is older.

Release authorization covers required remote merge/release checks. Continue targeted local checks; no full local or ownership gate, no scanner/protection bypass, no crosswalk.

## Release boundary

Choose the next unoccupied version after refreshing main and tags; bounded accessors add capability, so a minor release is the initial candidate. Reconcile concurrent release work before bumping. Source release completes only after the exact tag is on main, required checks pass, GitHub Release is published and all four native archives plus checksums are verified. Documentation publication is asynchronous and remains pending unless verified. The four consumer binding rows require their own post-release exact-pin adoption evidence; a source release alone does not close that acceptance.

## Public import correction

The first unpublished import was refused by the coordinated private-identifier check. Its exact rejected patch is retained privately in the local wave evidence. This replacement was created through AEP from the same source snapshot with the private organization identifier generalized before any journal event was written. Source acceptance and source snapshot hashes are preserved; no scanner policy or exception changed.

## Publication and current evidence

Opening commit 59afcf8caec5230a88aadfd7c590a703e0b5500b passed the coordinated hook and signed common scan. Signed publish succeeded to refs/heads/feat/adopter-priority-wave; fresh Git remote advertisement matches. Remote common check: https://github.com/beyond10x/ess/runs/103090084123. The existing App branch-creation exemption authorized this feature branch; no repository rule was edited. This is planning publication, not main integration or a release.

Xattr reconciliation: three existing tests passed once, 42.36-second build and 0.00-second test execution. Story moved active to implemented on the revised exact-label verification contract, retaining the missing real-labelled-filesystem witness limit. Its build was cleaned (638.1 MiB); managed tree wt-6f0fa901447d finished and reviewed eligible for exact GC. Retained unit report and logs are beneath local-evidence:ess-evolution-20260910/priority-wave/xattrs.

Latest observed published GitHub Release is 0.22.2, published 2026-09-10T20:11:55Z, with four native archives and SHA256SUMS. No new version/tag has been chosen or pushed for this wave. Main remains 6b666e58 at this observation.


## Final review routing and source progress

Literal review trend: pass one found one introduced documentation overclaim; pass two found zero. AEP findings reports carried [], new [], and resolved [docs.rs:1730 contract-drift blocker CONFIRMED introduced]. The final attack added no redundant tests and ran eight existing deciding cases once; full raw output and its digest are retained. The separate pre-existing admission exhaustion gap is tracked by story:literal-representation-walk-exhaustion and remains outside this wave's source scope. Merge 55c2b107 contains only reviewed source; this progress record adds the changelog and planning evidence.

Accessor design trend: pass one found three, pass two one. The signature ledger reports carried [], new [design:297 contract-drift blocker NEEDS-CHANGE introduced], resolved [design:61 boundary blocker, design:214 contract-drift blocker, design:123 boundary warning]. Resource bounds and suite/report routing are settled; native nested Optional assignment is specified, but observation cannot reconstruct every hidden source state. Root authorized an explicit conformance capability refusal where observation cannot determine the expected value, preserving typed native assignment. Same implementor is making that bounded correction; root reviews it directly under the two-attack rule before source work starts. Both immutable review reports remain verbatim in the store. No design test/build was represented as executed.

Required remote checks are authorized for the final candidate. Fresh GitHub reads found no open ESS PR and observed published release 0.22.2 with four uploaded archives and SHA256SUMS. These metadata reads do not independently requalify the historical release checks or downloaded artifact bytes. No new release tag or PR has been created.

Validation evidence: local-evidence:ess-evolution-20260910/priority-wave/progress-validation.log, exit 0. It reports the existing unassessed delivery scope and prose-only review warnings; the literal final report contains an explicit empty findings block, which this CLI still labels as no findings block. Do not invent findings to suppress that diagnostic. Agent aggregate token/tool/wall costs are unavailable from this harness; runner counts and observed command durations are retained instead.


## Source implementation dispatch

Final accessor design fa2d234c is integrated after published docs/evidence b3f2cb74. Its final content digest is b9e7e6f5ddf7d3775d7d79e5fd5df92c683feb6b851632f8a9d069445aaa9657. Coordinator review retained all assertions and required cases; both design review outcomes are fixed and the design approval explicitly covers implementation admission only. Story is active. Same implementor resumes the same unit with implementation-brief.md; complete native/conformance/compatibility support and exact refusals remain required. Root owns generated public schema/accounting, changelog, planning and delivery changes.

Literal publication b3f2cb74 passed signed common checks and is freshly advertised on refs/heads/feat/adopter-priority-wave. Remote check: https://github.com/beyond10x/ess/runs/103096472982. The existing bot App exemption authorized feature-branch publication; no protection or hook changed. All outgoing author and committer identities were verified. Literal target cleanup reclaimed 628.1 MiB; worktree wt-99a063fb0aaf finished, assessed eligible and removed with exact-id worktree GC. Tests/reviews/raw logs remain outside the removed tree in the recorded local evidence. Unit branches await final main integration before deletion.


## Latest operator delivery boundary

The operator clarified that the current task is to provide a remote pull request against main. Finish the selected implementation and review evidence, publish the candidate, and deliver that PR. Main merge and the previously requested new release are later milestones; this PR handback must not claim either has happened.

Consumer-related pipeline failures are explicitly acceptable for this delivery: the operator may repair them in another change, or permits disabling those consumer checks. CI already invokes task check SKIP_CONSUMER_CHECKS=true. Prefer retaining and identifying any remaining consumer-only failure on the PR rather than making speculative workflow edits. Core accessor, documentation, native generation and conformance correctness still require the focused feature evidence; no full local or ownership gate and no unchanged reruns are authorized by this clarification.

Exact previous-reader witness is retained in local-evidence:ess-evolution-20260910/priority-wave/compat-0.22.2. The released Linux archive matches both GitHub asset SHA256 5ce965299cf20d0872acf23c30aedcca1593ca68ffd49a0130de78bc2eb0e67d and the published checksum manifest. Extracted executable SHA256 88100c18992c073983feda6f31eee0f42e6c1f057f6d34375af2bdcccd1c203c reports ess 0.22.2. This establishes previous-reader identity without a duplicate baseline build; capability probes remain implementor work.

Design/admission head dcdc3343d016e4f76801123522a882f1915e7b7e is also published with signed common evidence; remote check https://github.com/beyond10x/ess/runs/103097453941. Integration primary source is clean apart from subsequent coordinator planning updates; accessor implementation remains in its isolated tree.


## Remote PR and newly prioritized consumer gaps

Draft PR https://github.com/beyond10x/ess/pull/28 targets main from feat/adopter-priority-wave. It honestly identifies completed literal documentation, existing xattr reconciliation, reviewed accessor design and in-progress accessor implementation. The current milestone is a reviewable remote PR; main merge/release have not occurred. Consumer-only pipeline failures are accepted and will be documented, while core accessor correctness remains required.

The operator subsequently prioritized task:ess-gaps-measured-in-a-consumer-specification, which contains eight gaps rather than approximately six: the active accessor plus list selection and six archived arguments. The task was imported through AEP with private identifiers generalized before the first journal event; exact primary snapshots remain retained and untouched. Seven distinct high-priority proposed stories now decompose it, with acceptance and typed cited/inferred scope. It decomposes initiative:ess-evolution. The six archived originals were not illegally reopened or silently duplicated as active history.

Read-only scopers scope_literal and design_adversary inspected current source and returned state-views.md and time-size.md under local-evidence:ess-evolution-20260910/priority-wave/gap-scoping. No builds/tests were run for that scoping. Important corrections to the submitted claims and actual consumer witnesses are in the parent task. Its recorded waves start with authored setup plus enum coverage, then compact output plus clock-design work, then periodic, list-selection and subject-state units as scopes permit. There are 41 exact-path collisions, no cycles and one unrelated unassessed delivery story. The proposed-only query does not include the live accessor unit: its active ownership remains a separate preflight constraint. No new implementation wave was dispatched.

Accessor source progress remains independent: the implementor first observed four failing accessor-admission cases with a passing legacy-flat case, then passed 11 focused domain cases and a compiler check after the shared domain DAG/resolver change. Native generators, conformance/report paths and complete feature review remain in progress. This source slice is not a completed-feature verdict. Exact previous-reader binary evidence remains available without rebuilding the old compiler.


## Consolidated publication and current ownership

Planning checkpoint 4b935f0a6d1d4e34f8ccbd53b4000fe5d0bf649b committed the new eight-gap task and seven follow-ups. The store validated at 262 artifacts, exit 0, with the existing scope/review warnings retained in local-evidence:ess-evolution-20260910/priority-wave/gap-scoping/validation.log. Signed publication of that history then refused candidate exceeds scan limit. Public Gates git.rs accumulates changed blobs from each post-baseline commit and enforces MAX_TOTAL=256 MiB; the approximately 24 MB journal recurred through the branch's eight commits. This was a capacity refusal, not permission to weaken scanners.

The exact original tree c1618ddb592b34e751b7f37d0129fac5d3f0ac53 was consolidated onto current main 6b666e58 as bot commit ac369db9a152811019bdb840e919c571b1a719f3. Both precommit tree ids matched exactly. Original branch feat/adopter-priority-wave still retains 4b935f0a locally; its earlier dcdc3343 history remains published, and verified pre-consolidation.bundle retains the eight-commit range with main as prerequisite. No source/planning bytes, hook or policy changed during consolidation; no force push was used.

Consolidated head ac369db9 passed signed common checks and is freshly advertised on refs/heads/feat/adopter-priority-review. Check: https://github.com/beyond10x/ess/runs/103101749747. Current remote review is draft PR https://github.com/beyond10x/ess/pull/29 against main, replacing draft #28. Integration remains wt-05b595184ac8, now on feat/adopter-priority-review, owned by the coordinator. Commit further planning evidence with the completed source batch to avoid unnecessary journal-history duplication.

At an explicit idle checkpoint, the accessor implementor paused all writes/builds. Root switched the same wt-30254233b1b4 normally from impl/priority-bounded-accessor at dcdc3343 to impl/priority-bounded-accessor-review at ac369db9. Source/design/manifests outside planning were verified identical between bases. All 32 dirty source files, including seven untracked files, retained exact bytes/modes; the tracked binary patch was identical. Backup patch, untracked copies and before/after inventories are retained in accessor/pre-base-switch* and post-base-switch-files.json. Root released its checkpoint lease and told the implementor to resume. No test rerun was required merely for ancestry. The old unit branch retains its already published base and no unique source commit.

Accessor implementation was still incomplete at the ancestry checkpoint: that attempted native test had not executed because its fixture had generic type-inference compile errors. Eleven domain cases and the compiler check remain the measured earlier success; native execution, observation/resource/version/report completion and adversarial implementation review remain required. The implementor owns the resumed source tree; coordinator owns final review, commits, publication and cleanup. The independent new-gap planning task is complete as a decomposition, while its eight capability outcomes remain active/proposed.


## Latest resumed verification and retained CI finding

PR #28 is now closed with a link to replacement #29; its published branch is retained. Verified #29 is OPEN, draft=true, base main, head feat/adopter-priority-review at ac369db9. No superseded PR runs remain active: common checks and documentation validation succeeded; CI completed with failure. Retained log pr28-ci-failed.log identifies the failure precisely: projection-check found generated docs/interactions.md and site/interactions.html stale after the literal wording change. This is coordinator-owned generated-output work, not a consumer-only failure covered by the user's waiver. Regenerate through cargo xtask generate on the completed integrated candidate and check the resulting projections; do not hand-edit generated files or rerun the full local gate.

After the source checkpoint, the implementor corrected the fixture and successfully executed the native witness in Rust and Go. Rust ran one passing test; Go selected its one named native test and exited 0. Cases cover absence, union availability, three-segment/newtype traversal, whole-struct copying, nested Optional identity and additional wrapping. Original failed compilation evidence remains retained. Conformance Go/admission/resource/compatibility work is still active; this is not a complete accessor verdict.

The post-publication AEP record validated with exit 0 at gap-scoping/post-publication-validation.log before this final progress paragraph. The newly requested decomposition is already published in ac369db9. These subsequent wave-progress updates remain coordinator-owned for the next source/evidence batch, limiting repeated large journal blobs in Git history.


## Planning critique and public accessor guide

The newly prioritized eight-gap decomposition has completed both bounded critique rounds. Nine acceptance findings were fixed through AEP; the other three first-round perspectives approved, and all four final perspectives approved. Exact reports and the runtime dispatch deviation are recorded in the parent task. No new gap implementation wave was dispatched. The review records and completion clarifications are retained for the next source/evidence commit so the large planning journal is not republished in an unnecessary extra history step.

Coordinator-owned public guides now describe unreleased ess/3 accessors, declared names, Optional/union availability, terminal whole-value copying, source limitations, conditional suite/6-/7 and existing report/2 requirements. Their three exact website paths were added to the accessor's AEP scope. These are uncommitted drafts awaiting comparison with the completed implementation and one required site build; no successful site validation is claimed.

Latest implementor observations extend earlier native Rust/Go witnesses: five Rust observation tests and a separate suite/6 roundtrip/downgrade admission test passed; generated Go first failed on unsupported /6, then passed suite admission and two focused observation tests covering missing/null/present, union branches, declared names and malformed payloads. Resource, compatibility/report and regression work remains active. No full local or ownership gate ran. Coordinator still owns final generated projection/schema/support and diagnostic catalog refresh on the integrated source.


## Operator-authorized complete gap delivery

The operator now explicitly instructs: "just fix all of the gaps, and provide one PR at the end" and "use multiple subagents to fix those gaps". This authorizes implementation of all eight owners already planned, their source commits, corrections and managed integration. The delivery boundary is one final ESS pull request against main, using existing draft #29; no automatic main merge, tag or release is part of this handback. Prior full-local/ownership-gate exclusions and consumer-only CI tolerance remain in force. The planning task's implemented status closes decomposition only, not these eight capability outcomes.

Skill version 0.8.1 — aep-drive:wave. The selected set is explicit user approval, so no repeated approval question is required. The previous scope computation and four-perspective review remain evidence; fresh unfiltered computation follows below. The unrelated unassessed delivery story is outside the user-selected eight. All selected stories serve O2.

Dispatch now: existing accessor implementor continues wt-30254233b1b4; separate aep-drive:implementor units take closed-enum-outcome-coverage and authored-entity-state-arrangement. Harness dispatch uses existing general collaboration agents with the exact installed charter, because no plugin-native type selector exists. Compact output, clock provenance, periodic triggers, list selection and subject-state outcomes remain selected for subsequent coordinated units, not deferred out of the final PR.

The active accessor collides by filename with these units. Apply the skill's explicit fallback: split shared files by named symbols and current base line ranges in each brief; require diff hunk headers; run git merge-tree --write-tree --merge-base=<base> before the first unit merge and inspect its result. Enum coverage owns command validation and synthesize::{supply,reach,decides,rendered} (base lines1757-1885), not accessor mapping, binding refusal types or suite-version routing. Entity arrangement owns its authored setup declarations/compilation, new setup step/target operation, setup-specific runner dispatch and setup admission arms, not accessor values, expect_invocation, Run::resolve or accessor helper functions. Shared import/export/version wiring is explicitly reconciled by root, never silently overwritten. Designs precede code; source changes use exact per-unit assignment.

Root now owns the accessor CLI main.rs/release_evidence.rs and directly affected report/preflight/target-failure tests; the accessor agent retains the coverage.rs target-closure preflight change and all conformance-library/Go implementation. This is an acknowledged ownership transfer. All units preserve old reader/legacy bytes and coordinate the unreleased vocabulary before introducing independent format numbers.

Storage preflight: 8.6 GiB available in /tmp after creating the integration build target. Each unit receives a unique /tmp/ess-priority-wave/<unit>-target, two build jobs, debug0, incremental0 and sccache. At most two builds run concurrently; the accessor holds one slot, enum receives the second after its design checkpoint, and arrangement starts with design and verifier construction until a slot is available. Keep the 3 GiB floor. Root integration target is /tmp/ess-priority-wave/integration-target and is currently idle after projection generation. User authorization for /tmp supersedes older charter defaults requiring targets inside trees.

The known CI projection failure is repaired locally by cargo xtask generate: exactly two normative outputs changed, zero removed, exit0. It is not a consumer failure. Retained command output is priority-wave/projection-generate.log. No full local gate ran. Final source validation checks these projections after integration; one final PR carries the completed gaps.

### Fresh unfiltered scheduling result

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:authored-entity-state-arrangement",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/authored.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/go/mod.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/go/runtime.go"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/input.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/runner.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/scenario.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/target.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/tests/authored.rs"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/authored-entity-state-arrangement.md"
            }
          ]
        },
        {
          "id": "story:closed-enum-outcome-coverage",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/command.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-domain/src/expression.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/synthesize.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/witness.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/tests/synthesis.rs"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/closed-enum-outcome-coverage.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 2,
      "artifacts": [
        {
          "id": "story:conformance-compact-json-output",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/src/coverage.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli/src/main.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli/tests/coverage_cli.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli/tests/coverage_lineage.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/scenario.rs"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:timestamp-clock-provenance-contract",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/types.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-compiler/src/ir.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-compiler/src/resolve.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/expression.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/types.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-primitives/src/time.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/target.rs"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/timestamp-clock-provenance.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 3,
      "artifacts": [
        {
          "id": "story:periodic-binding-trigger-contract",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-gen/src/asyncapi.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-gen/src/docs.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-gen/src/graph.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/ir.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/resolve.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/binding.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-conformance/src/go/runtime.go"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-conformance/src/runner.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/scenario.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/synthesize.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/target.rs"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/periodic-binding-triggers.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 4,
      "artifacts": [
        {
          "id": "story:binding-list-selection-contract",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-synth/src/plan.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/ir.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/resolve.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/binding.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/expression.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-conformance/src/runner.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-conformance/src/scenario.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-conformance/src/synthesize.rs"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-diff/src/diff.rs"
            },
            {
              "confidence": "cited",
              "path": "docs/design/binding-mapping-bounded-accessor.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 5,
      "artifacts": [
        {
          "id": "story:subject-state-outcome-guards",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/docs.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/http.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen/src/openapi.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth/src/plan.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/ir.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler/src/resolve.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/command.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain/src/entity.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/decision.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/input.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/src/synthesize.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance/tests/synthesis.rs"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-diff/src/diff.rs"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/subject-state-outcome-guards.md"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [
    {
      "a": "story:authored-entity-state-arrangement",
      "b": "story:binding-list-selection-contract",
      "path": "crates/verify/ess-conformance/src/runner.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:authored-entity-state-arrangement",
      "b": "story:binding-list-selection-contract",
      "path": "crates/verify/ess-conformance/src/scenario.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:authored-entity-state-arrangement",
      "b": "story:conformance-compact-json-output",
      "path": "crates/verify/ess-conformance/src/scenario.rs",
      "confidence": "cited"
    },
    {
      "a": "story:authored-entity-state-arrangement",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/verify/ess-conformance/src/go/runtime.go",
      "confidence": "inferred"
    },
    {
      "a": "story:authored-entity-state-arrangement",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/verify/ess-conformance/src/runner.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:authored-entity-state-arrangement",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/verify/ess-conformance/src/scenario.rs",
      "confidence": "cited"
    },
    {
      "a": "story:authored-entity-state-arrangement",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/verify/ess-conformance/src/target.rs",
      "confidence": "cited"
    },
    {
      "a": "story:authored-entity-state-arrangement",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/verify/ess-conformance/src/input.rs",
      "confidence": "cited"
    },
    {
      "a": "story:authored-entity-state-arrangement",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/verify/ess-conformance/src/target.rs",
      "confidence": "cited"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:closed-enum-outcome-coverage",
      "path": "crates/specify/ess-domain/src/expression.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:closed-enum-outcome-coverage",
      "path": "crates/verify/ess-conformance/src/synthesize.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:conformance-compact-json-output",
      "path": "crates/verify/ess-conformance/src/scenario.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "cited"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/specify/ess-compiler/src/resolve.rs",
      "confidence": "cited"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/specify/ess-domain/src/binding.rs",
      "confidence": "cited"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/verify/ess-conformance/src/runner.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/verify/ess-conformance/src/scenario.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/verify/ess-conformance/src/synthesize.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/generate/ess-synth/src/plan.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "cited"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/specify/ess-compiler/src/resolve.rs",
      "confidence": "cited"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/verify/ess-conformance/src/synthesize.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/verify/ess-diff/src/diff.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/specify/ess-compiler/src/resolve.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:binding-list-selection-contract",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/specify/ess-domain/src/expression.rs",
      "confidence": "cited"
    },
    {
      "a": "story:closed-enum-outcome-coverage",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/verify/ess-conformance/src/synthesize.rs",
      "confidence": "cited"
    },
    {
      "a": "story:closed-enum-outcome-coverage",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/specify/ess-domain/src/command.rs",
      "confidence": "cited"
    },
    {
      "a": "story:closed-enum-outcome-coverage",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/verify/ess-conformance/src/synthesize.rs",
      "confidence": "cited"
    },
    {
      "a": "story:closed-enum-outcome-coverage",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/verify/ess-conformance/tests/synthesis.rs",
      "confidence": "cited"
    },
    {
      "a": "story:closed-enum-outcome-coverage",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/specify/ess-domain/src/expression.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:conformance-compact-json-output",
      "b": "story:periodic-binding-trigger-contract",
      "path": "crates/verify/ess-conformance/src/scenario.rs",
      "confidence": "cited"
    },
    {
      "a": "story:periodic-binding-trigger-contract",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/generate/ess-gen/src/docs.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:periodic-binding-trigger-contract",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "cited"
    },
    {
      "a": "story:periodic-binding-trigger-contract",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/specify/ess-compiler/src/resolve.rs",
      "confidence": "cited"
    },
    {
      "a": "story:periodic-binding-trigger-contract",
      "b": "story:subject-state-outcome-guards",
      "path": "crates/verify/ess-conformance/src/synthesize.rs",
      "confidence": "cited"
    },
    {
      "a": "story:periodic-binding-trigger-contract",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:periodic-binding-trigger-contract",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/specify/ess-compiler/src/resolve.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:periodic-binding-trigger-contract",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/verify/ess-conformance/src/target.rs",
      "confidence": "cited"
    },
    {
      "a": "story:subject-state-outcome-guards",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/specify/ess-compiler/src/ir.rs",
      "confidence": "inferred"
    },
    {
      "a": "story:subject-state-outcome-guards",
      "b": "story:timestamp-clock-provenance-contract",
      "path": "crates/specify/ess-compiler/src/resolve.rs",
      "confidence": "inferred"
    }
  ],
  "unassessed": [
    "story:binding-delivery-at-most-once"
  ],
  "cycles": []
}
```

## Active parallel implementation dispatch

Operator explicitly authorized implementation of all eight gaps using multiple subagents into one final PR. Accessor remains active in wt-30254233b1b4; existing source work is retained.

Closed enum: story:closed-enum-outcome-coverage; branch impl/gap-closed-enum; base 4c03e4fc89153323d79a7639a08679b15c246d28; worktree wt-2420def710d2; build /tmp/ess-priority-wave/enum-target; scratch local-evidence:ess-evolution-20260910/priority-wave/enum; agent scope_literal, implementor charter.

Authored setup: story:authored-entity-state-arrangement; branch impl/gap-entity-arrangement; same base; worktree wt-fa580dc62e63; build /tmp/ess-priority-wave/arrangement-target; scratch local-evidence:ess-evolution-20260910/priority-wave/arrangement; agent design_adversary, implementor charter.

The two implementation-brief.md files define exact symbol/range ownership in shared conformance files. Agents return actual diff hunk headers; coordinator must inspect merge-tree output before the first merge. Root owns all planning mutations, central version wiring, public documentation, CLI edges, generated outputs and publication. At most two builds, two jobs each, unique targets and a 3 GiB tmpfs floor. Arrangement starts design/verifiers pending a build slot. No full local or ownership gate and no unchanged test reruns, per operator instruction. Remaining five gaps stay in delivery scope and are dispatched as dependencies and agent slots permit.

## Verifier and storage progress

Accessor CLI deciding cases now exist: three actual generation/report/lineage cases and one early target-failure envelope case. Baseline runs executed 3 and 1 cases respectively, all failed on the existing unsupported source/path boundary; logs retained in the accessor scratch directory. New report-before-target and release-qualification verifiers are written but not yet executed. These are incomplete implementation checks, not passing receipts.

Enum initial deciding runner executed 9 cases, 3 passed/6 failed; first implementation run passed 9/9. Expanded boundary checks and regression/lint remain pending. Authored setup deciding run executed 2 cases, both failed on the unsupported setup field with a valid no-creator model. Agents retain exact red/green logs; no implemented move follows from these partial observations.

Both enum and accessor then hit tmpfs EDQUOT before finishing their next checks. The host exposes a per-user quota despite 6.3 GiB globally free. Coordinator confirmed compiler processes stopped and relocated exactly this wave's four disposable compiler target directories to local-evidence:ess-evolution-20260910/priority-wave/build-targets/{accessor,enum,arrangement,integration}. Copies/moves completed successfully; /tmp now has 10 GiB free and the cache filesystem 83 GiB. Source trees, branches, leases and evidence are preserved. Test temporary files remain on /tmp; maximum two builds/two jobs each remains. Unit brief build assignments are superseded by build-storage-correction.md. Only unexecuted or failed checks resume.

Read-only consumer evidence for actual polling and clock authority is retained in gap-scoping/time-authority-source-check.md: the two-second ticker is conditional and session-scoped, refresh inputs are host-owned, another worker can cause refresh, and producer timezone/comparability cannot be inferred from a literal Z. Historical compact fixture pair located in its original consumer projection checkout; no consumer source changed.

## Standing priority for new intake

Operator additionally authorized checking for newly arrived tasks, assigning relevant additions high priority, and handling them with subagents in the same delivery. Recheck fresh intake at each wave boundary and before final publication; preserve existing dependencies, scope assessment and per-unit verification. Existing crosswalk exclusion remains.

Fresh ESS check on 2026-09-11 compared the integration store and primary store, then fetched origin. Local Connector search refused connector-unreachable/Connection refused; that gap was reported before the authorized Git fallback. Origin/main remains6b666e58f2e87dd8798d27f935e9a012203296a3; PR branch remote remains592c6fce01dd0bf86fd5d4c95a27cf4a97a1757b. No newly arrived ESS task or story was identified beyond the known eight-gap intake and previously reviewed priority items. Primary's existing staged/untracked planning input was preserved. No unrelated historical task was silently added to this ESS delivery.


## Implementation checkpoint and renewed intake — 2026-09-11

Remote fetch still names origin/main6b666e58; primary task list remains the same six tasks (latest-primary-tasks.json). No new ESS task was found. The operator's standing instruction remains: new ESS intake is high priority and delegated. Preserve primary staged planning. Connector search returned connector-unreachable/Connection refused, so this session used its already-authorized Git fallback.

Accessor candidate87326740 and first source adversary are retained. Immutable review-result:bounded-accessor-implementation-adversary-1 records two introduced findings. Root correctiond513145de085777160ca2b9d2e848a2b50bc53be fixes nullable-newtype payload shape and canonical Unicode-byte admission. Actual bounded_accessor12 passed and scoped strict Clippy completed; correction logs are under accessor/implementation-adversary. A second/final attack is in progress; this is not yet downstream adoption.

Enum candidatea66279d5 has immutable review-result:closed-enum-implementation-adversary-1 (74/76 scoped cases passed). Origin of the invariant-excluded witness remains undecided: no exact-base run is claimed. Coordinator keeps the correction inside this high-priority unit because executable branch witnesses are explicit acceptance and both finite/default candidates must not contain invalid typed values. Correction will reuse arrangement's existing recursive type/invariant validator, preserving candidate ordering and conservative named unreachable refusals. No duplicate solver.

Arrangement candidate259a00cf710b61ac56765b034e020685ca6801c7 has310 package cases green plus strict Clippy/package formatting; original report is arrangement/implementation-report.md. First implementation adversary is active. Root CLI verifier entity_setup_cli executed1 red case against pre-integration6084655c: source setup unknown, exit101 (arrangement/cli-red.log). Fresh ordinary6/coverage7 and explicit report2 routing remain coordinator work.

Compact writer story is active priority-high in wt-5210f914aeea, branch impl/gap-compact-conformance, base6084655c. Scope additions: conformance coverage.rs typed document serializer and mechanical author_suite call argument. Agent owns new compact methods/fresh writer and dedicated tests; root owns shared version routing, report preflight, public docs and generated output. All machine waves/collisions/unassessed output is retained verbatim in compact/waves.json. Known file collisions are handled under the already-authorized symbol assignment, exact returned hunks and inspected merge-tree requirement. No extra approval or full local gate.

The historical consumer managed tree has been retired by its owner since measurement. Exact commit9e8b5c482b30b6d1941dc2aeabe12f45ef052627 remains in Git; root exported only consumer with git archive into task-owned compact/consumer-source. Agent measures both fresh encodings from this identical source. Consumer primary has advanced to7f03d513 and has another session's dirty code; leave it alone. Build paths remain the disk-backed per-unit paths in build-storage-correction.md, two concurrent builds maximum.


## Unpublished privacy recovery

Common Gates refused the first checkpoint commit for74 personal-paths matches in two review artifacts and their journal records. No rejected commit or private report was published. The original complete AEP snapshot, immutable reviews and journal are preserved under local-evidence:ess-evolution-20260910/priority-wave/planning-private-recovery; journal SHA25677c65c457dde37b2211cebc3d0f0f35db22fb3a3013bae4273fa80b0d4fbbf70. Coordinator restored only its unpublished planning changes to committed6084655c, then replayed desired state through AEP. Public review variants explicitly disclose path redaction and original report digests; findings remain unchanged. No hook or policy bypass.


## Final source integration and PR boundary

All eight selected source implementations have local commits and are combined in the integration checkout: bounded event accessors, closed enum coverage, authored entity arrangement, compact suites, subject-state outcomes, periodic bindings, clock provenance and list selection. The later operator instruction sets one remote PR against main as this delivery boundary. No main merge or source release is claimed here.

Parallel agents resolved generator and conformance overlaps, corrected the recorded subject-state move-source finding, and checked the merged native selection paths. The coordinator resolved compiler/source cause composition and typed diff format routing. New cross-feature regressions reject periodic selection without event authority and refuse selection inputs whose reading contracts cannot be preserved. Source validation nine cases, native/conformance selection sixteen cases, and reading-selection two cases passed. The second subject-state correction passed three cases. Every confirmed implementation finding has an immutable review and a separate fixed outcome.

The public documentation build passed, including WASM browser checks. The full local gate and ownership gate were not run, as instructed. Exact retained focused logs are under local-evidence:priority-wave/gap-scoping and the per-unit evidence directories. Enum coverage and compact output are implemented in AEP; six stories stay active for actual adopter evidence and their explicitly named host obligations.

Publication uses the clean managed tree wt-cabd48f19a37 based on the current remote PR head 592c6fce. Source changes are consolidated there and planning events replayed only through AEP, preserving original unpublished history privately. This prevents new private consumer identifiers from entering public commits. Unit trees remain retained until their wanted content has verified publication and managed recovery permits cleanup.


## PR 29 CI correction

The Gate at public candidate cd930cc5 failed in the coverage producer refusal control: crates/edge/ess-cli/tests/support/coverage_producer.rs expected the former suite/5-only message after report pairing expanded to versions 5, 6 and 7. The real CLI correctly refused before execution. Correct the stale assertion and include stderr in assertion failures, preserving the nonzero exit, empty stdout and absent report checks. Source scope is that existing test helper; no production semantics or workflow checks are weakened.

Evidence: https://github.com/beyond10x/ess/actions/runs/34561516524/job/103145036051 . Documentation, common security/privacy and both macOS ownership jobs passed on that candidate. Verify the failed control only plus scoped lint/format, then publish the fix to this same PR and inspect its replacement CI run. The operator prohibition on a full local gate remains in force.


Publication constraint: the common Gates candidate scanner refused the combined source/planning commit with `candidate exceeds scan limit`. Its cumulative scan from the enrolled adoption baseline is bounded at 256 MiB and rereads changed blobs for each commit. A new version of the large journal crosses that boundary. The source-only correction is published separately through unchanged security checks; these validated AEP updates remain local and must be published after a governed capacity/baseline resolution. Do not disable scanning or alter enrolled policy to evade the refusal.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 42e2fb1980966313564c476e23be8abadbc6a6efe15900d4db6800753dc3dcb2, retained as local-evidence:runtime-gaps/publication-replay/snapshots/42e2fb1980966313564c476e23be8abadbc6a6efe15900d4db6800753dc3dcb2.md. Source creation recorded at 2026-09-10T23:37:06Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
