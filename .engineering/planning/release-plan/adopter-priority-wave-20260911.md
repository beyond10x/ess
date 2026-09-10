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

- Literal docs: scoped; tree/branch/build/scratch pending creation after opening plan commit.
- Xattrs: scoped; targeted verification only; tree/branch/build/scratch pending creation.
- Bounded accessor: scoped; design must precede implementation and docs unit must be integrated before overlapping source edits; tree/branch/build/scratch pending creation.

All scratch and logs belong beneath local-evidence:ess-evolution-20260910/priority-wave/<unit>. Retain reports and original draft snapshots; never create HANDOFF files. Record actual triple and head before every implementation dispatch. Publish wanted commits before worktree finish and exact reviewed gc. Keep any tree whose recovery remains unproven.

## Release boundary

Choose the next unoccupied version after refreshing main and tags; bounded accessors add capability, so a minor release is the initial candidate. Reconcile concurrent release work before bumping. Source release completes only after the exact tag is on main, required checks pass, GitHub Release is published and all four native archives plus checksums are verified. Documentation publication is asynchronous and remains pending unless verified. The four Babelconnect binding rows require their own post-release exact-pin adoption evidence; a source release alone does not close that acceptance.

## Public import correction

The first unpublished import was refused by the coordinated private-identifier check. Its exact rejected patch is retained privately in the local wave evidence. This replacement was created through AEP from the same source snapshot with the private organization identifier generalized before any journal event was written. Source acceptance and source snapshot hashes are preserved; no scanner policy or exception changed.
