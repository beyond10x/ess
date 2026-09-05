# Review boundaries wave 1 — 2026-09-05

Skill version 0.7.0. Integration branch: `wave/review-boundaries-1`, managed record `wt-752828a285ba`; base `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Both units serve `vision:O2`.

## Approval and selection

The operator said: “i approve you handling all wave impls your own, start”. `approval-record:review-remediation-standing-implementation` records standing implementation approval. It authorizes the opening planning commit, two unit commits, their integration merges, closing evidence/store commit and the merge to local main. Publication and release remain outside this grant.

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
| `story:review-secret-sanitization` | `impl/review-secret-sanitization` | pending opening commit | `<ess-managed-root>/review-secret-sanitization` | worktree `target/` | worktree `target/review-boundaries-1/review-secret-sanitization/` | awaiting opening gate |
| `story:review-report-reader-validation` | `impl/review-report-reader-validation` | pending opening commit | `<ess-managed-root>/review-report-reader-validation` | worktree `target/` | worktree `target/review-boundaries-1/review-report-reader-validation/` | awaiting opening gate |

Only the coordinator writes planning artifacts or commits. Implementors establish red acceptance cases, run their package tests, formatter and Clippy, then return uncommitted source/test changes. Each green result receives an independent test-only adversary pass. Closing requires every offline gate step's own exit code, applicable site build, scope confirmation and evidence before terminal story moves.

## Execution

Opening commit and cheap gate pending. No implementation result is claimed yet. Logs and reports will be read and recorded before any managed cleanup. A local-only commit cannot satisfy the worktree manager's publication requirement; no removal bypass is authorized.
