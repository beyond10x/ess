# Review boundaries wave 1 — 2026-09-05

**Implementation and integrated verification complete; publication-backed cleanup pending.** Both stories are implemented with recorded evidence. All eight offline gate steps exited 0, with 1,438 tests passed, no failures and no ignored tests; the site build also exited 0. `resource-blocker:review-wave-storage-capacity` preserves the earlier interruption and clearance. The rest of the remediation epic remains active.

Skill version 0.7.0. Integration branch: `wave/review-boundaries-1`, managed record `wt-752828a285ba`; base `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Both units serve `vision:O2`.

## Approval and selection

The operator said: “i approve you handling all wave impls your own, start”. `approval-record:review-remediation-standing-implementation` records standing implementation approval. It covers the opening planning commit, the two units, their integration merges, closing evidence/store commit and the merge to local main. Each unit has an implementation commit followed by an adversarial-test commit so the immutable reviews' original commit references remain reachable; the resource interruption also required an evidence checkpoint. These commits are confined to the approved units and their governed records. Publication and release remain outside this grant.

The [saved proposal](2026-09-05-review-remediation.md) contains the complete draft and exact proposed-set wave computations, including all collisions, unassessed ids and cycles. Its selected set has two cited, high-confidence, disjoint crate scopes and no dependencies or blockers. The approved stories are now active. Other P0 candidates require broader shared surfaces and will be independently scoped at the next replan.

## Launch evidence

Primary ESS is clean on main at the base, equal to remote main. Before launch, `git branch --list 'wave/*' 'impl/*'` printed no branches. The managed inventory contains separate feature, release and review records; none is recorded as a preceding coordinated wave. This is a provenance inference from inventory and branch evidence, not a claim that other sessions are abandoned. Their trees remain intact. The coordinator's prior plan changes are the requested plan, isolated from primary main.

Latest measured free disk: 23,002,062,848 bytes, above the 21,474,836,480-byte floor. Available RAM: 48,901,672,960 bytes. Prior measured Kubernetes and conformance package targets: 78,296 and 284,228 KiB respectively. N=2 fits these measurements; reassess space before every dispatch and return. Each unit uses its own target with `/usr/bin/sccache`, no incremental compilation and no debug information. Numeric model budget was requested during planning and not supplied; N remains below the skill default and within the harness concurrency cap.

Atlas authority is revalidated immediately before bot operations. The initial authority `a8fb936ddcb35c8971311610e5c63cc86d612fab` became stale during launch when remote main advanced to `6035d6e1209686ca474a3f43975fde7d8621ba48`; provisioning an exact managed authority precedes the opening commit.

Native named agent types are unavailable. Dispatch uses the exact file charters `aep-drive:implementor` and `aep-drive:adversary` through the collaboration harness, inheriting the session model. This is the same explicit adapter limitation recorded in the proposal.

## Unit records

Paths use `<ess-managed-root>` for the managed ESS trees directory. The ignored launch record under the coordinator's `target/review-boundaries-1/` resolves absolute machine paths. Unit scratch is inside that unit's target so build, probe and report files share the same managed lifecycle.

| Unit | Branch | Head | Worktree | Build | Scratch | Stage |
| --- | --- | --- | --- | --- | --- | --- |
| `story:review-secret-sanitization` | `impl/review-secret-sanitization` | `7130468ce5baa8178407bc02df557f27a7cae7ee` | `<ess-managed-root>/review-secret-sanitization` | worktree `target/` | worktree `target/review-boundaries-1/review-secret-sanitization/` | implemented; merged at `c332e7236cf2568e08ef2210bbabbdd7f7ccc6e8`; awaiting publication-backed cleanup |
| `story:review-report-reader-validation` | `impl/review-report-reader-validation` | `6b887663736d088307b4f8957de8488740110e6a` | `<ess-managed-root>/review-report-reader-validation` | worktree `target/` | worktree `target/review-boundaries-1/review-report-reader-validation/` | implemented; merged at `aefdc330089fd9a96baf39179857a84686efa944`; awaiting publication-backed cleanup |

Only the coordinator writes planning artifacts or commits. Implementors establish red acceptance cases, run their package tests, formatter and Clippy, then return uncommitted source/test changes. Each green result receives an independent test-only adversary pass. Closing requires every offline gate step's own exit code, applicable site build, scope confirmation and evidence before terminal story moves.

## Execution

Opening commit: `3d8d6c6b287ce1c462cc50ea74f1ba5c171b827b`, with both author and committer verified as `b10x-bot[bot]`. The opening `fmt-check`, `example-check`, `projection-check`, `release-check` and `action-check` each exited 0; planning validation exited 0 with the six previously recorded empty-findings advisories. Free disk after these checks: 22,953,893,888 bytes. Both managed unit trees were created at this opening commit; the exact triples above now exist, and each is at implementor dispatch. File-backed briefs and absolute launch records are under their assigned scratch roots.

Storage exception: before either package built, free space dropped to 11,868,954,624 bytes; both new unit targets together were only 168 KiB. Builds paused at the declared floor. Finished-tree assessment and size inventory found no useful ESS reclamation; no cleanup was applied. Disk readings then stabilized. The coordinator revised the execution plan to one Cargo unit at a time and an 8,589,934,592-byte hard reserve, approximately four times the existing 2,119,576-KiB integration target. This changes a coordinator estimate, not a repository gate. Separate targets and compact builds remain mandatory, with disk measured after commands and new builds stopped if consumption resumes. Each brief has a file-backed resource supplement.

The Secret implementor's test runner executed 2→5 cases; the new malformed-response corpus failed before the change, and the required sanitizer mutation failed the existing named redaction test. Restored package tests, formatter and strict Clippy each exited 0. The [portable implementation report](../reviews/2026-09-05-review-boundaries-1-secret-implementation.md) preserves those observations and scope confirmation. Target size after package gates: 125,552 KiB; free bytes: 11,682,316,288.

The report-reader implementor's test runner executed 229→238 cases. Six new assertions first failed because all four public read routes accepted invalid claims. Final package tests, formatter and strict Clippy each exited 0; target size 309,280 KiB and free bytes 11,155,525,632. The [portable implementation report](../reviews/2026-09-05-review-boundaries-1-report-implementation.md) preserves exact observations and scope confirmation. No story is terminal before the integrated gate.

Read-only preparation for the next replan independently scoped output containment and semantic diff. Containment must validate before site map collection and cover the additional CLI sinks. Semantic diff also affects dependency-closed generated contract digests and persisted change vocabulary; its compatibility decision is unresolved and recorded in the story. Neither future unit has been dispatched for implementation.

Logs and reports will be read and recorded before any managed cleanup. A local-only commit cannot satisfy the worktree manager's publication requirement; no removal bypass is authorized.

## Adversarial results and routing

- `review-result:review-boundaries-1-secret-adversary-pass-1` preserves the returned pass verbatim. It executed 5→8 package cases, with one red; formatter and Clippy exited 0. The added malformed JSON and late malformed annotation cases passed. The failing subprocess-stderr case also reproduced in a newly built opening-baseline binary, with an empty package source diff against `3d8d6c6b287ce1c462cc50ea74f1ba5c171b827b`. Coordinator classification is therefore pre-existing, independent of the pass's recorded undecided origin. It is owned by `story:review-kubectl-diagnostic-sanitization`; the original story records review outcome no-op because this finding was filed separately. The original red assertion remains in the unit tree and in the [saved test patch](../reviews/2026-09-05-review-boundaries-1-secret-adversary-tests.patch).
- `review-result:review-boundaries-1-report-adversary-pass-1` preserves the returned resource-limited pass verbatim. Its four new tests executed directly from the already-built binary: three passed, one red. Full package after-count and Clippy are unavailable; standalone formatting passed. The YAML null-identity case is INFEASIBLE/undecided: no repository YAML-report caller was established. Coordinator contract assessment: the story promises report/version/count/status validation and preserves opaque identity strings; no evidence establishes that YAML scalar coercion to the string `null` violates that acceptance. This finding does not justify changing the implementation. The original case is preserved in its unit tree and [saved test patch](../reviews/2026-09-05-review-boundaries-1-report-adversary-tests.patch); review outcome no-op records the assessment without rewriting the immutable pass.

Coordinator resolution after storage recovered: the Secret case now explicitly characterizes the single known failure-diagnostic disclosure, retains refusal/destination/stdout checks, names its new P0 story and directs that story to restore the preserved no-disclosure assertion when fixed. The YAML case still refuses JSON nulls and missing/unknown fields; it explicitly checks YAML's string `null` only for the two unconstrained identity fields. These are documented expectation changes under the wave's pre-existing/wrong-case routing rows; no test was ignored or deleted. The original patches preserve exactly what the adversaries tested. Complete package tests then executed 8 and 242 cases with no failures, and both formatter and strict Clippy checks exited 0. A Clippy warning in the new report test was corrected by adding numeric separators without changing its timestamp value. No second attack was needed because implementation code did not change after the first attacks.

## Resource incident and resumption

During adversarial testing free space dropped below the revised reserve to 5,913,124,864 bytes, then to 910,450,688. All our Cargo/rustc processes were already stopped. Read-only process I/O observed compilation in other checkouts, including a managed Substrate tree. The coordinator reclaimed only its separate baseline measurement target with `cargo clean --target-dir target/report-baseline`: 470 files, 281.6 MiB. Raw measurement logs were outside that directory and remain intact. Another reading was already 288,198,656 free bytes. A subsequent rebound to 10,156,445,696 confirms changing shared usage, not stable capacity. No other worktree, source, cache or process was removed or stopped.

Resume after the resource blocker is cleared from fresh stable capacity evidence. Reuse the existing branches, heads, test additions, reports and briefs; do not redispatch the implementations from scratch. No final workspace step has run on a merged implementation, so the prior 1,419-test baseline cannot be claimed as this wave's validation. Complete test routing, package checks, serial merges, all eight `task check` steps, applicable site build, evidence and status transitions. Only then merge local main and address publication-backed managed cleanup.

Checkpoint incident: copying the planning validation record and staging Git objects both returned `No space left on device`. The commit attempt produced no commit because staging had failed. The coordinator then removed only its generated rustdoc output with `cargo clean --doc` (2,170 files, 56.3 MiB) to preserve the handoff; this output must be regenerated by the eventual full gate. The original raw planning log remained intact. The [validation output](../reviews/2026-09-05-review-boundaries-1-planning-validation.txt) records 79 valid artifacts and the same six empty-findings advisories from planning.

The storage blocker was cleared from fresh readings of 54,895,435,776 and 53,885,259,776 free bytes. Resumption reused all existing unit state and kept one Cargo unit at a time. The integrated source is `aefdc330089fd9a96baf39179857a84686efa944`; free bytes at full gate launch were 30,672,695,296.

## Closing evidence

[`verification-report:review-boundaries-1-integrated`](../../.engineering/planning/verification-report/review-boundaries-1-integrated.md) records the final package and integrated checks, the tested source commit, test routing and limitations. Exact gate step statuses:

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

Workspace runner totals: 1,438 passed, 0 failed, 0 ignored. `task site-build` exited 0 after WASM/browser checks and Docusaurus generation. Its baseline npm dependency advisories remain outside these two fixes. [Final planning validation](../reviews/2026-09-05-review-boundaries-1-final-planning-validation.txt) is preserved verbatim: 80 artifacts, the six previously explained empty-findings advisories, `valid`. Both implemented stories have final scope confirmation and evidence against the integrated source commit. No source or test changed after these gate runs; closing edits record evidence and lifecycle state.

Raw unit scratch reports/logs were copied to the coordinator's ignored `target/review-boundaries-1/archive/<unit>/` before any retirement attempt. Original adversarial patches and portable reports are committed engineering evidence. The local main merge is the authorized integration step; publication is a separate grant. The worktree manager requires wanted commits to be published before finishing a unit, and the wave skill requires the previous units to be retired before the next wave. Retain the trees and branch heads until that recovery proof exists; no force removal or cleanup bypass is permitted.

Next work: the newly observed kubectl stderr disclosure is P0 and scoped to its existing package. Output containment has independent scope ready for the next replan. Semantic diff needs its newly identified persisted-vocabulary/contract-digest compatibility decision before implementation. Recompute readiness and collisions after cleanup and recheck shared disk capacity; standing implementation approval remains in force.

Agent metrics: Secret implementor measured 8m 11s from its first persisted timestamp to source review; its full turn duration was not exposed. Report implementor, both adversaries and both read-only scopers have no exposed full-turn duration. The collaboration harness exposes no per-agent token/tool-count totals, so those figures are unavailable. No tag, release or publication is part of this wave.

Final base refresh: published `main` advanced by `d032565fa518ac0c9c020a95c8e4f6a00cc0b136`, a bot-authored update of exactly the documentation bundle and redirect-facade workflow pins. It was merged at `a298885227f89fd7639a54b564699a8e91e55329`. A zero-exit diff confirmed unchanged Cargo manifests/lock, Taskfile, AGENTS, crates, examples, generated outputs, schemas, website, release actions and both release workflows relative to the tested source. The action check also exited 0 after the refresh; the existing Rust and site results therefore cover identical tested inputs. The two adopted workflow pins match the published main bytes.

After verification the shared filesystem filled again. All unit scratch was compared byte-for-byte with the coordinator archive before `cargo clean` reclaimed the two units' disposable targets. Their source worktrees and branches remain intact, clean and pending publication-backed retirement. The scratch paths in the table describe the original allocation; preserved records now reside under the coordinator archive and in the committed evidence linked above.
