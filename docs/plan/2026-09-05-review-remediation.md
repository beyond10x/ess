# ESS review remediation — 2026-09-05

**Implementation authorized on 2026-09-05.** The operator granted standing approval to handle all remediation waves. Execution starts in [review-boundaries-1](2026-09-05-review-boundaries-1.md), under `approval-record:review-remediation-standing-implementation`. The proposal and measurements below preserve the earlier planning-stage evidence; current launch facts and implementation results belong to each wave page.

**Skill version 0.7.0** — the version in `.claude-plugin/plugin.json`; the stage-1 proposal quotes it.

Owner: [`epic:review-boundary-remediation`](../../.engineering/planning/epic/review-boundary-remediation.md), serving [`vision:O2`](../../.engineering/planning/vision/O2.md), decisions as data with evidence. The original decomposition had **27 new stories and 3 reused stories**; wave 1 added one P0 story for a baseline-reproduced kubectl diagnostic disclosure. The first two stories are implemented and passed the integrated 1,438-test gate plus site build. The remaining 3 legacy drafts were scoped for honest wave computation, not pulled into remediation.

## Current progress

There are 31 remediation stories: nine implemented, no active stories, and 22 remaining drafts. [Wave 1](2026-09-05-review-boundaries-1.md) is published on main at 0f80f71e7ef997e8a3c7d2ad19e9997090e8e769; both unit worktrees were removed after remote recovery proof. [Wave 2](2026-09-05-review-boundaries-2.md) has implemented diagnostic sanitization and output containment, passed all eight offline gate steps with 1,475 tests and passed the site build. It is published on main at 98ea8abeeaf80846f525b5def8b531c139ed7071, and both unit worktrees have been removed with remote recovery proof. The user has separately approved publication of green remediation waves and managed cleanup in approval-record:review-remediation-standing-publication; release tags/version bumps remain separately authorized.

[Wave 3](2026-09-05-review-boundaries-3.md) has implemented persisted delivery validation, TypeScript namespace feasibility and the conformance migration design. Clean integration source1ef817ea9d51b2e927febce18eb2d540256e8e76 passed all eight offline steps with1505 workspace tests, the site build and the explicit TypeScript compiler lane. It is published on main6cbe372ea68487f11fa6d53500f2033a95f29b79 and all three unit worktrees were removed with advertised remote recovery proof. Design completion alone does not close the dependent count/coverage implementation findings.

[Wave 4](2026-09-05-review-boundaries-4.md) has implemented semantic diff/impact coverage and infrastructure IR invariant preservation. Its clean combined source acb7859e3202ffdc1ca840dde67f7ca4da33c746 passed all eight offline steps with 1,579 workspace tests, site build and the SDK old/new generation, checker and actual compilation experiment. Atlas ADR 0036 records the version/profile decision and order. It is published on main at 0d08e219a61cd1911b913d3da20c8e4b87993b94; both implementation and both SDK experiment worktrees were removed after verified archives, clean locks and advertised recovery proof. Atlas's migration record is published at a58a98048d5adf60b96d86a00220aeb0f10f5218 and its support tree is removed. The coordinator and current clean Atlas authority remain in use for the remaining work.

## Evidence and scope of the plan

[The unchanged review](../reviews/2026-09-05-architecture-review.md) supplies F01–F17 and their classifications at ESS `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Its SHA-256 is `fecce053b6633546f10613c580eba1b5c48be3315d70c6b4369f26143020f828`. Primary ESS main was clean and equal to remote main at this revision during planning. This session copied only the detailed review as a portable source; the earlier review checkout and its unpublished outlook remain intact.

The review's probes and 1,419-test gate are attributed historical evidence; this session did not rerun those probes or claim the defects fixed. Architecture choices below are proposals grounded in cited owners, with inferred edit surfaces marked in each story.

The exact clean Atlas authority was `a8fb936ddcb35c8971311610e5c63cc86d612fab`, verified against remote main. Its `ROADMAP.md:17` defines O2. The primary Atlas checkout was dirty/stale, so authority was read in managed record `wt-1bb42c471901`, with `B10X_ATLAS_CHECKOUT` assigned to that checkout when organization tooling is used. No Atlas mutation is planned here.

## Finding-to-work traceability

Each linked story contains a single observable acceptance, reproduction/validation matrix, compatibility boundary, dependencies and typed edit scope. A finding with multiple stories closes only when all of its required outcomes have evidence.

| Finding | Priority | Owning stories | Required disposition |
| --- | --- | --- | --- |
| F01 | P0 | [`story:review-semantic-diff-coverage`](../../.engineering/planning/story/review-semantic-diff-coverage.md) | Compare all omitted semantics and propagate obligations. |
| F02 | P0 | [`story:review-persisted-delivery-validation`](../../.engineering/planning/story/review-persisted-delivery-validation.md), [`story:review-infra-ir-invariants`](../../.engineering/planning/story/review-infra-ir-invariants.md) | Validate persisted delivery graphs; protect infrastructure IR handles. |
| F03 | P0 | [`story:review-report-reader-validation`](../../.engineering/planning/story/review-report-reader-validation.md), [`story:review-conformance-format-design`](../../.engineering/planning/story/review-conformance-format-design.md), [`story:a-skipped-scenario-is-not-a-failed-one`](../../.engineering/planning/story/a-skipped-scenario-is-not-a-failed-one.md), [`story:review-conformance-coverage`](../../.engineering/planning/story/review-conformance-coverage.md) | Validate v1 now; design and migrate category/coverage/exact-suite evidence without reinterpreting v1. |
| F04 | P1 | [`story:review-openapi-semantic-accounting`](../../.engineering/planning/story/review-openapi-semantic-accounting.md) | Preserve meaning or persist a gap/refusal by dialect. |
| F05 | P0 | [`story:review-secret-sanitization`](../../.engineering/planning/story/review-secret-sanitization.md), [`story:review-kubectl-diagnostic-sanitization`](../../.engineering/planning/story/review-kubectl-diagnostic-sanitization.md) | Malformed shape refusal and subprocess diagnostic sanitization are implemented, reviewed and published. |
| F06 | P1 | [`story:review-observation-completeness`](../../.engineering/planning/story/review-observation-completeness.md) | Retain collection scope and selector uncertainty through consumers. |
| F07 | P1 | [`story:review-rust-target-feasibility`](../../.engineering/planning/story/review-rust-target-feasibility.md), [`story:review-typescript-root-collision`](../../.engineering/planning/story/review-typescript-root-collision.md) | Validate target symbol/layout feasibility and compile emitted counterexamples. |
| F08 | P1 | [`story:review-primitive-semantics`](../../.engineering/planning/story/review-primitive-semantics.md) | Use one normative value-domain contract and exact numeric migration. |
| F09 | P1 | [`story:review-expression-typechecking`](../../.engineering/planning/story/review-expression-typechecking.md) | Typecheck full expression paths before synthesis. |
| F10 | P0/P1 | [`story:review-output-containment`](../../.engineering/planning/story/review-output-containment.md), [`story:review-output-ownership`](../../.engineering/planning/story/review-output-ownership.md), [`story:scenarios-directory-compiles-nothing`](../../.engineering/planning/story/scenarios-directory-compiles-nothing.md), [`story:review-authored-discovery`](../../.engineering/planning/story/review-authored-discovery.md) | Contain all output first; add ownership/recovery and predictable input discovery. |
| F11 | P1 | [`story:review-delivery-trust-contract`](../../.engineering/planning/story/review-delivery-trust-contract.md), [`story:review-cache-origin`](../../.engineering/planning/story/review-cache-origin.md), [`story:review-execution-recovery-design`](../../.engineering/planning/story/review-execution-recovery-design.md) | Separate consistency, content origin and execution state; recovery implementation remains an explicit obligation. |
| F12 | P2 | [`story:review-glossary-boundaries`](../../.engineering/planning/story/review-glossary-boundaries.md) | Correct concepts and record interface/multiple-entrypoint ownership disposition. |
| F13 | P1/P2 | [`story:review-format-catalog`](../../.engineering/planning/story/review-format-catalog.md), [`story:review-schema-resource-identity`](../../.engineering/planning/story/review-schema-resource-identity.md) | Catalog current bytes and schema boundaries before choosing new identities. |
| F14 | P2 | [`story:review-typed-diagnostics`](../../.engineering/planning/story/review-typed-diagnostics.md) | Carry rule/source identity independently of wording. |
| F15 | P1 | [`story:review-browser-replay-fidelity`](../../.engineering/planning/story/review-browser-replay-fidelity.md) | Fix or visibly refuse replay assignment/view semantics. |
| F16 | P1 | [`story:review-consumer-coverage`](../../.engineering/planning/story/review-consumer-coverage.md), [`story:fuzz-the-specification-surface`](../../.engineering/planning/story/fuzz-the-specification-surface.md), [`story:review-public-support-claims`](../../.engineering/planning/story/review-public-support-claims.md) | Gate consumer coverage, retain fuzz ownership and ground public status in evidence. |
| F17 | P2 | [`story:review-composition-contract`](../../.engineering/planning/story/review-composition-contract.md) | Document exact surface selection and byte transport; typed-client expansion is conditional. |

Two durable obligations prevent design/documentation work from masquerading as implementation:

- [`obligation:review-execution-recovery-implementation`](../../.engineering/planning/obligation/review-execution-recovery-implementation.md) remains open until the recovery design's typed model supports a concrete decomposition and its implementation passes the fake-executor failure/retry matrix. F11 is not closed by the design alone.
- [`obligation:review-contract-rollout-coordination`](../../.engineering/planning/obligation/review-contract-rollout-coordination.md) requires exact relying-reader inventory, Atlas ADR, downstream governed work and compatibility evidence before changed formats become default writers.

No new runtime entity is guessed into the plan. Stories operate on existing Rust-owned models; any new persisted ownership or execution evidence is modeled and validated before implementation is decomposed around it. UNMAPPED identity, authority, ownership or cardinality stays unresolved in its binding design.

## Sequencing and decisions

1. **Immediate byte-preserving hardening:** F05 sanitization and the narrow F03 v1 reader are proposed below. Then replan P0 F10 output containment, F02 persisted delivery/IR validity and F01 diff/impact. The latter units touch multiple packages or shared CLI/compiler files and are deliberately outside the first single-package exercise.
2. **Coverage design and exact supported subsets:** draft the F03 compatibility design independently; migrate count vocabulary before durable coverage/exact-suite binding. F04 import accounting, F07 target feasibility and F09 typechecking follow as scoped units. F06 waits for sanitization and infrastructure invariant preservation. Prioritize remaining P0 implementation over unrelated P2 documentation at each replan.
3. **Evidence, recovery and primitives:** F11 cache/trust work waits for validated delivery input; recovery design precedes modeling and implementation. F08 begins with the primitive matrix and format inventory, then a coordinated migration; do not change f64 serialization as an incidental refactor.
4. **Interoperability and maintenance:** F12/F13 glossary and catalogs can be prepared early as separate document surfaces; serialize public-guide changes. F14 diagnostics and F15 replay are separate, with semantics aligned to whichever primitive contract has landed.
5. **Extension evidence:** F16's matrix follows the concrete F01 repairs; the planned fuzz harness will supply its no-panic property, while target-feasibility stories own emitted-program compilation. Public status follows actual release evidence, not merely Cargo version.

These are priority phases. Standing approval now covers future remediation waves, each selected afresh from the store, blockers, remote base and typed scopes after the preceding wave. Dependencies are machine-readable where one outcome requires another; shared write paths are machine-readable collisions. A compilation dependency alone is not invented into an edit collision.

F12 and F17 need an honest statement of supported behavior, not automatic multiple-entrypoint or typed-client expansion. F13 needs a catalog and an owned schema identity policy, not blanket renaming. The existing create-only refusal design, Java target and Go formatting work remain outside this epic. The scoper found the create-only story's historical “only refusal form” claim too broad; its body now preserves the historical text with a cited current-source correction.

## Proposed first wave

Name: `review-boundaries-1`. Units: **2**, each serving `vision:O2`.

| Unit | Observable fix | Scope | Status |
| --- | --- | --- | --- |
| `story:review-secret-sanitization` | Malformed Secret shapes cannot leak into observation output or diagnostics. | `crates/infra/ess-kubernetes` — cited, high confidence | proposed |
| `story:review-report-reader-validation` | Contradictory v1 reports are refused on public read boundaries, preserving valid v1 semantics/bytes. | `crates/verify/ess-conformance` — cited, high confidence | proposed |

Eight independent `aep-drive:story-scoper` dispatches established the two candidates and every initially unassessed legacy draft. The coordinator recorded their sections and typed entries serially. All other newly drafted scopes are grounded coordinator scopes, with inferred paths marked; independently re-scope those units before later dispatch.

Selection used **`aep artifact waves` from `protocol 0.54.0`**, not the fallback pairwise method. The complete draft computation is attached below. Its greedy global partitions include unrelated legacy work, so they are not the priority order: after judging implementability and priority, the two scoped candidates moved legally from draft to proposed, and a second CLI computation over proposed stories verifies the exact selected set. Neither has a dependency or blocker, and both carry a `serves:vision:O2` edge. No source story moved to active.

### Exact proposed-set computation

Command: `aep artifact waves --kind story --status proposed --format json`; exit 0.

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:review-report-reader-validation",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            }
          ]
        },
        {
          "id": "story:review-secret-sanitization",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/infra/ess-kubernetes"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [],
  "unassessed": [],
  "cycles": []
}
```

### What is deliberately left out

- Other P0 fixes remain urgent next candidates, but their package/CLI/API coupling exceeds the first-wave single-package boundary.
- The versioned F03 count/coverage work needs the shared design and downstream compatibility; v1 reader validation can land independently.
- F04, F06, F07, F08 and F09 require their declared subset/compatibility work; no padding the first wave with a new target or an unknown surface.
- Broad delivery recovery is design-first and retains an open implementation obligation.
- All public documentation edits share a conservative `website/docs` token and are scheduled separately.
- Create-only refusal has unresolved semantics; Java requires an independently provisioned execution toolchain; Go formatting is distinct existing maintenance.

## Pre-flight measurements and launch conditions

Measured 2026-09-05 in the planning checkout, on the unchanged source revision. Compiler cache was available at `/usr/bin/sccache` and explicitly wired for both measured builds. Settings: `CARGO_INCREMENTAL=0`, dev/test debug information disabled. Each measured target directory was separate; no other worktree's target was reused.

| Check | Observation | Interpretation |
| --- | --- | --- |
| ESS primary | Clean, main, HEAD = remote main = `fd06a4d61bfb7b4990617810655dc181d6a3ab00` | Baseline is current; recheck at launch. |
| Free disk initially / after builds | 74,280,751,104 / 73,851,957,248 bytes | Above proposed 20 GiB floor (21,474,836,480 bytes); floor is coordinator policy, not a repository fact. |
| Available RAM at initial measurement | 47,774,928,896 bytes | Observed using `free -b`; recheck before launch. |
| Kubernetes package | `cargo test --locked -p ess-kubernetes`: exit 0, 2 executed tests; 4.084 s; target 78,296 KiB | Baseline only; new fault cases must increase the executed count. |
| Conformance package | `cargo test --locked -p ess-conformance`: exit 0, 229 executed tests; 13.047 s; separate target 284,228 KiB | Baseline only; all 9 test binaries and doc tests completed. |
| Planning target total | 362,752 KiB after both builds | Includes both measurements and task scratch; recorded for later cleanup. |
| Existing linked ESS checkouts | 17 before this planning tree, 18 after | Existing work is preserved. Registry inventory shows 10 pre-existing active and 7 finished records; recovery/previous-wave provenance is still unresolved for launch. |
| Existing target directories | 13,777,404; 1,823,388; 1,426,252; 6,000,704; 14,059,580 KiB | From exact `du -sk` inventory below; not evidence they are abandoned. |
| Model budget | Asked during planning; no numeric balance supplied | Provisionally N=2 implementation agents plus reviewer capacity; below the skill's default 4 and the harness's 3 sub-agent concurrency limit. Operator steering may reduce N. |
| Baseline full gate | All eight tasks underlying `task check` exited 0; 1,419 tests executed | Baseline on unchanged implementation, not remediation evidence; rerun at wave integration. |
| Approval | Not granted | Stage 1 only. |

Existing build inventory, with paths written relative to the managed ESS trees root:

```text
13777404  wt-34496b6990aa/target
1823388   wt-780a9b306a76/target
1426252   wt-b2a2189b9ecc/target
6000704   wt-ce6e143e1fe5/target
14059580  wt-e46db550dce9/target
```

Launch must first inspect previous-wave records and prove the wave preflight is satisfied. Existing worktrees/builds are not cleaned by age or name. Use managed `worktree` status/reconcile/finish/GC operations, preserve uncommitted work and inspect exact recovery evidence. The planning checkout is intentionally dirty with this plan; it must be committed on the integration branch through bot tooling and cheaply gated before unit checkouts are created. The primary stays clean.

### Planned durable unit records

Notation avoids machine-specific home paths: `<ess-managed-root>` is the registry root returned by the worktree profile, and every actual path must be written into the local launch record when `worktree create --json` assigns it. The three planned branch names were checked with `git branch --list` and were unused at plan completion. Use explicit stable IDs below if available; no unit worktrees exist yet.

| Unit | Planned branch | Planned worktree | Build directory | Scratch root | Head / stage |
| --- | --- | --- | --- | --- | --- |
| coordinator | `wave/review-boundaries-1` | existing managed record `wt-752828a285ba` | `<coordinator>/target` | `<coordinator>/target/review-boundaries-1` | `fd06a4d...` / plan only |
| secret sanitization | `impl/review-secret-sanitization` | `<ess-managed-root>/review-secret-sanitization` | `<unit>/target` | `<coordinator>/target/review-boundaries-1/review-secret-sanitization` | not created |
| report reader | `impl/review-report-reader-validation` | `<ess-managed-root>/review-report-reader-validation` | `<unit>/target` | `<coordinator>/target/review-boundaries-1/review-report-reader-validation` | not created |

Before dispatch, name the actual branch heads and triples in the wave page/local launch record, create file-backed briefs, gate the opening store/page commit, then run one implementor and one adversary per unit. Implementors never mutate the planning store. Use full role names `aep-drive:implementor` and `aep-drive:adversary`.

**Dispatch limitation:** this harness has no native `subagent_type` selector. Scopers and critics are spawned with the actual plugin charter path and full role name. The critic charter's `sonnet/high` model is unavailable here; session-model execution is a reported deviation, not a claimed native pinned dispatch. The four critic roles are `aep-plan:plan-critic-acceptance`, `aep-plan:plan-critic-design`, `aep-plan:plan-critic-scope`, and `aep-plan:plan-critic-parallel-safety`. With three sub-agent slots, the fourth reads the same frozen draft after a slot opens; no critic receives another's findings.

## Approval and completion boundary

**Approval would authorize only:** one opening store/plan commit, 2 unit commits, their integration merges, one closing store/evidence commit and the gated merge to main; no push, tag, release, unrelated work or next wave. All direct commits/merges use Atlas bot tooling with verified bot author and committer.

The first-wave approval does not waive launch preflight. If the available model budget is lower, replan N before dispatch. Every subsequent wave is proposed from the then-current store. A release is a separate human stop.

The worktree skill and workspace instructions take precedence over the wave skill's raw-Git cleanup example: publish wanted commits before `worktree finish`, review `worktree gc --dry-run` and apply only exact reviewed IDs. Since wave approval grants no push, retain trees with unpublished wanted commits and report publication/cleanup pending instead of deleting them.

During implementation: first observe each new regression fail; run the relevant package gate; record each adversary report unchanged as a review-result; route findings by introduced/pre-existing/undecided origin; use at most two attacks. At integration capture each step's own exit status and skipped-step output, record the exact merge commit evidence, validate planning, then merge only the green wave to main. Do not claim the remediation epic closed while either obligation remains open.

## Critic record

Round 1: design, scope and parallel-safety approved. Acceptance returned six findings: two obligations mixed procedure with discharge criteria; fuzz acceptance mixed corpus setup with its no-panic property; three stories combined independently checkable outcomes. All six artifacts were revised through AEP, preserving their required checks in validation/procedure sections, and all six outcomes were recorded as `fixed`. The critic classified the fuzz wording as pre-existing; the other five were introduced by this draft.

Records: [acceptance round 1](../../.engineering/planning/review-result/review-remediation-acceptance-round-1.md), [design round 1](../../.engineering/planning/review-result/review-remediation-design-round-1.md), [scope round 1](../../.engineering/planning/review-result/review-remediation-scope-round-1.md), [parallel-safety round 1](../../.engineering/planning/review-result/review-remediation-parallel-safety-round-1.md).

Round 2: design, scope and parallel-safety approved again. Acceptance returned three remaining findings because the first body replacement had retained the superseded compound paragraphs beneath the new sentences. The coordinator removed those duplicate paragraphs after all four critics finished, read back each complete Acceptance section, and confirmed that the required checks remain in their validation matrices. No third critic round ran; the final acceptance verdict remains `needs-revision` as immutable history, and the three final corrections were checked locally rather than re-approved by the panel.

Records: [acceptance round 2](../../.engineering/planning/review-result/review-remediation-acceptance-round-2.md), [design round 2](../../.engineering/planning/review-result/review-remediation-design-round-2.md), [scope round 2](../../.engineering/planning/review-result/review-remediation-scope-round-2.md), [parallel-safety round 2](../../.engineering/planning/review-result/review-remediation-parallel-safety-round-2.md).

The three findings that survived into round 2 are preserved verbatim below. Each has a `fixed` outcome on that round's record after the final body revision; none is being claimed as independently re-reviewed:

- story:review-conformance-coverage — the acceptance combines preserved incompleteness with exact-suite evidence binding, which can pass independently; define one observable acceptance result and place its component checks in validation — .engineering/planning/story/review-conformance-coverage.md:33
- story:review-glossary-boundaries — the acceptance combines glossary distinctions with interface-layout and multiple-entrypoint dispositions, which can pass independently; define one observable completion result for the reference — .engineering/planning/story/review-glossary-boundaries.md:28
- story:review-public-support-claims — the acceptance combines correcting the HTML-output description with citing current release evidence, which can pass independently; define one observable completion result for the status page — .engineering/planning/story/review-public-support-claims.md:30

Those citations identify the revisions the critic saw; the final story bodies carry the corrected sections. Outcome totals: round 1 `fixed=6, no-op=0, escalated=0`; round 2 `fixed=3, no-op=0, escalated=0`. Six distinct issues were corrected across nine recorded finding outcomes. The scope critic reissued its same round-1 verdict with actual relative-store validation output to avoid persisting a personal home path; that publication correction changed no finding or reviewed artifact.

The harness exposes no reliable per-agent token/tool/wall totals for these dispatches; those values are unavailable, not estimated. Eight scoper tasks and eight critic reviews across four independent perspectives ran; each critic used the same charter for both rounds. There is no non-interactive approval bypass: the root planning session is interactive; child read-only reviews cannot authorize implementation.

## Validation and tooling observations

All tasks underlying `task check` were run sequentially with each task's own captured status:

```text
fmt-check 0
clippy 0
test 0
doc-check 0
example-check 0
projection-check 0
release-check 0
action-check 0
```

Workspace tests executed **1,419** passing cases. No gate task skipped itself. `task site-build` exited **0** in 15.795 seconds, including the WASM/browser lab and Docusaurus. npm reported 30 dependency audit findings and blocked install scripts for two packages; these are untriaged baseline dependency warnings, not a security audit or new remediation scope. The source review explicitly did not audit every dependency.

After the gates, `df -B1` reported 66,465,345,536 free bytes, still above the proposed disk floor. Planning-checkout allocations were `target` 2,118,808 KiB, `examples/billing-web/target` 2,268 KiB, `website/node_modules` 661,040 KiB and `website/build` 6,548 KiB. These paths are retained with the managed planning record; nothing was deleted. Logs are under `target/plan-scratch`; the measurements and meaningful results are preserved here before any eventual cleanup.

`worktree status --json` confirmed 18 present ESS records: the planning record plus 10 pre-existing active and 7 finished records. Registry heads differ from current Git heads on some active records; only fresh Git/recovery evidence may authorize cleanup. A finished lifecycle label alone is not a GC approval.

Installed `protocol 0.54.0` has a reporting limitation: `aep artifact validate` reports approval records with a literal fenced `findings` block containing `[]` as “states its findings as prose only”. The actual blocks are present, and validation exits 0 with no problems. Records were kept unchanged; no fake finding was invented to suppress the advisory. The final validation output follows verbatim (exit 0):

```text
74 file(s) in .engineering/planning: 74 artifact(s)
6 review(s) recorded no findings block:
  - review-result:review-remediation-design-round-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-design-round-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-parallel-safety-round-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-parallel-safety-round-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-scope-round-1 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
  - review-result:review-remediation-scope-round-2 states its findings as prose only — nothing can enumerate what it found, so                  the next review starts from nowhere
valid
```

## Complete draft-store computation

Command: `aep artifact waves --kind story --status draft --format json`; exit 0. These are all returned waves, all 144 excluded collisions, and the unassessed/cycle lists verbatim. The two proposed units are shown in their separate exact computation above. Token equality is the CLI's rule: scopes use crate-directory tokens consistently; broader unresolved legacy design scope is not treated as dispatch-ready.

<details>
<summary>Verbatim draft-store waves, collisions and unassessed stories</summary>

```json
{
  "waves": [
    {
      "wave": 1,
      "artifacts": [
        {
          "id": "story:create-only-command-cannot-refuse",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-gen"
            },
            {
              "confidence": "inferred",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "crates/verify/ess-diff"
            },
            {
              "confidence": "inferred",
              "path": "docs/design"
            }
          ]
        },
        {
          "id": "story:fuzz-the-specification-surface",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "fuzz"
            }
          ]
        },
        {
          "id": "story:review-composition-contract",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/specify/ess-composition"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:review-conformance-format-design",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-conformance-coverage.md"
            }
          ]
        },
        {
          "id": "story:review-execution-recovery-design",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-execution-recovery.md"
            }
          ]
        },
        {
          "id": "story:review-infra-ir-invariants",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-analyze"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/infra-compiler"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-project"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-spec"
            }
          ]
        },
        {
          "id": "story:review-typescript-root-collision",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/schema-contract"
            }
          ]
        }
      ]
    },
    {
      "wave": 2,
      "artifacts": [
        {
          "id": "story:java-conformance-target",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            }
          ]
        },
        {
          "id": "story:review-format-catalog",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-format-catalog.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:review-observation-completeness",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/infra/ess-kubernetes"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-analyze"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/infra/infra-domain"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-project"
            },
            {
              "confidence": "inferred",
              "path": "crates/infra/infra-spec"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-observation-completeness.md"
            }
          ]
        },
        {
          "id": "story:review-rust-target-feasibility",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth"
            }
          ]
        },
        {
          "id": "story:review-semantic-diff-coverage",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-diff"
            }
          ]
        }
      ]
    },
    {
      "wave": 3,
      "artifacts": [
        {
          "id": "story:review-browser-replay-fidelity",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-replay-subset.md"
            }
          ]
        },
        {
          "id": "story:review-consumer-coverage",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "cited",
              "path": "crates/edge/ess-xtask"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-consumer-coverage.md"
            }
          ]
        },
        {
          "id": "story:review-glossary-boundaries",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-concept-boundaries.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:review-openapi-semantic-accounting",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-openapi"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-openapi-accounting.md"
            }
          ]
        },
        {
          "id": "story:review-typed-diagnostics",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-domain"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-typed-diagnostics.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 4,
      "artifacts": [
        {
          "id": "story:review-expression-typechecking",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/specify/ess-compiler"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-domain"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            }
          ]
        },
        {
          "id": "story:review-output-containment",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen"
            }
          ]
        },
        {
          "id": "story:review-public-support-claims",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-xtask"
            },
            {
              "confidence": "cited",
              "path": "website/docs"
            }
          ]
        }
      ]
    },
    {
      "wave": 5,
      "artifacts": [
        {
          "id": "story:review-persisted-delivery-validation",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-deployment"
            }
          ]
        },
        {
          "id": "story:review-schema-resource-identity",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "docs/design/review-schema-resource-identity.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        },
        {
          "id": "story:the-generated-go-runtime-is-gofmt-clean",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "Taskfile.yml"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-xtask"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            }
          ]
        }
      ]
    },
    {
      "wave": 6,
      "artifacts": [
        {
          "id": "story:review-primitive-semantics",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/generate/ess-gen"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-synth"
            },
            {
              "confidence": "cited",
              "path": "crates/specify/ess-primitives"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-primitive-semantics.md"
            }
          ]
        },
        {
          "id": "story:scenarios-directory-compiles-nothing",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            }
          ]
        }
      ]
    },
    {
      "wave": 7,
      "artifacts": [
        {
          "id": "story:a-skipped-scenario-is-not-a-failed-one",
          "inferred": true,
          "scope": [
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            },
            {
              "confidence": "inferred",
              "path": "website/docs/guides/verify-conformance.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 8,
      "artifacts": [
        {
          "id": "story:review-authored-discovery",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-authored-discovery.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 9,
      "artifacts": [
        {
          "id": "story:review-cache-origin",
          "inferred": false,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            }
          ]
        }
      ]
    },
    {
      "wave": 10,
      "artifacts": [
        {
          "id": "story:review-delivery-trust-contract",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": ".github/actions/release-component"
            },
            {
              "confidence": "inferred",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/generate/ess-deployment"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-delivery-trust.md"
            },
            {
              "confidence": "inferred",
              "path": "website/docs"
            }
          ]
        }
      ]
    },
    {
      "wave": 11,
      "artifacts": [
        {
          "id": "story:review-output-ownership",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-output-ownership.md"
            }
          ]
        }
      ]
    },
    {
      "wave": 12,
      "artifacts": [
        {
          "id": "story:review-conformance-coverage",
          "inferred": true,
          "scope": [
            {
              "confidence": "cited",
              "path": "crates/edge/ess-cli"
            },
            {
              "confidence": "cited",
              "path": "crates/verify/ess-conformance"
            },
            {
              "confidence": "inferred",
              "path": "docs/design/review-conformance-coverage.md"
            }
          ]
        }
      ]
    }
  ],
  "collisions": [
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:create-only-command-cannot-refuse",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:create-only-command-cannot-refuse",
      "path": "docs/design",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:java-conformance-target",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:java-conformance-target",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-authored-discovery",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-browser-replay-fidelity",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-composition-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-delivery-trust-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-format-catalog",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-output-containment",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:a-skipped-scenario-is-not-a-failed-one",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:java-conformance-target",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-browser-replay-fidelity",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-expression-typechecking",
      "path": "crates/specify/ess-compiler",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-expression-typechecking",
      "path": "crates/specify/ess-domain",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-output-containment",
      "path": "crates/generate/ess-gen",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-primitive-semantics",
      "path": "crates/generate/ess-gen",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-primitive-semantics",
      "path": "crates/generate/ess-synth",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/generate/ess-synth",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-semantic-diff-coverage",
      "path": "crates/specify/ess-compiler",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-semantic-diff-coverage",
      "path": "crates/verify/ess-diff",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-compiler",
      "confidence": "cited"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-domain",
      "confidence": "inferred"
    },
    {
      "a": "story:create-only-command-cannot-refuse",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-authored-discovery",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-browser-replay-fidelity",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-output-containment",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:java-conformance-target",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-cache-origin",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-output-containment",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-authored-discovery",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:review-conformance-coverage",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-browser-replay-fidelity",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-conformance-coverage",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-output-containment",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-cache-origin",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-delivery-trust-contract",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-format-catalog",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-composition-contract",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-conformance-format-design",
      "path": "docs/design/review-conformance-coverage.md",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-delivery-trust-contract",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-expression-typechecking",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-output-containment",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-conformance-coverage",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:review-public-support-claims",
      "path": "Taskfile.yml",
      "confidence": "inferred"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:review-public-support-claims",
      "path": "crates/edge/ess-xtask",
      "confidence": "inferred"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "Taskfile.yml",
      "confidence": "cited"
    },
    {
      "a": "story:review-consumer-coverage",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/edge/ess-xtask",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-format-catalog",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-infra-ir-invariants",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-output-containment",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/generate/ess-deployment",
      "confidence": "cited"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-delivery-trust-contract",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-primitive-semantics",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-semantic-diff-coverage",
      "path": "crates/specify/ess-compiler",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-compiler",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-domain",
      "confidence": "inferred"
    },
    {
      "a": "story:review-expression-typechecking",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-format-catalog",
      "b": "story:review-glossary-boundaries",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-format-catalog",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-format-catalog",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-glossary-boundaries",
      "b": "story:review-public-support-claims",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-glossary-boundaries",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-observation-completeness",
      "path": "crates/infra/infra-analyze",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-observation-completeness",
      "path": "crates/infra/infra-compiler",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-observation-completeness",
      "path": "crates/infra/infra-project",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-observation-completeness",
      "path": "crates/infra/infra-spec",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-openapi-semantic-accounting",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-output-containment",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-infra-ir-invariants",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:review-output-containment",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-openapi-semantic-accounting",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "inferred"
    },
    {
      "a": "story:review-output-containment",
      "b": "story:review-output-ownership",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-output-containment",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-output-containment",
      "b": "story:review-primitive-semantics",
      "path": "crates/generate/ess-gen",
      "confidence": "cited"
    },
    {
      "a": "story:review-output-containment",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-output-ownership",
      "b": "story:review-persisted-delivery-validation",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-output-ownership",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-persisted-delivery-validation",
      "b": "story:scenarios-directory-compiles-nothing",
      "path": "crates/edge/ess-cli",
      "confidence": "cited"
    },
    {
      "a": "story:review-primitive-semantics",
      "b": "story:review-rust-target-feasibility",
      "path": "crates/generate/ess-synth",
      "confidence": "cited"
    },
    {
      "a": "story:review-primitive-semantics",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/verify/ess-conformance",
      "confidence": "cited"
    },
    {
      "a": "story:review-public-support-claims",
      "b": "story:review-schema-resource-identity",
      "path": "website/docs",
      "confidence": "inferred"
    },
    {
      "a": "story:review-public-support-claims",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "Taskfile.yml",
      "confidence": "inferred"
    },
    {
      "a": "story:review-public-support-claims",
      "b": "story:the-generated-go-runtime-is-gofmt-clean",
      "path": "crates/edge/ess-xtask",
      "confidence": "inferred"
    },
    {
      "a": "story:review-semantic-diff-coverage",
      "b": "story:review-typed-diagnostics",
      "path": "crates/specify/ess-compiler",
      "confidence": "cited"
    }
  ],
  "unassessed": [],
  "cycles": []
}

```

</details>
