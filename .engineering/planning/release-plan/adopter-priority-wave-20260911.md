---
format: aep.planning-md/1
id: release-plan:adopter-priority-wave-20260911
kind: release-plan
status: active
title: Integrate adopter priority fixes and release ESS
relations:
- serves: vision:O2
revision: 7
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
              "path": "babelconnect-specs"
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

Choose the next unoccupied version after refreshing main and tags; bounded accessors add capability, so a minor release is the initial candidate. Reconcile concurrent release work before bumping. Source release completes only after the exact tag is on main, required checks pass, GitHub Release is published and all four native archives plus checksums are verified. Documentation publication is asynchronous and remains pending unless verified. The four Babelconnect binding rows require their own post-release exact-pin adoption evidence; a source release alone does not close that acceptance.

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
