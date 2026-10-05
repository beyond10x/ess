# ESS 0.54.0 — waves 1–3 (2026-10-05)

AEP implementing skill version 0.19.2, wave mode. Coordinator: one session on integration branch
`integrate/ess-054`, managed worktree `ess-054-plan` (`<worktrees>/ess/ess-054-plan`), forked from
`batch/ui-live-apps-complete-20261003` at `087935463f` (the 0.53.0 bundle, PR #431). Release plan:
`release-plan:ess-054`.

## Authority

On 2026-10-05 the operator asked to reconcile the ESS issues filed that day, plan them with the AEP
planning store, dispatch them in several waves and ship them as 0.54.0. The coordinator read that as
pre-approval of the stage-1 stop for every wave in this page: the stop is waived and the page is still
written. The commits this authorises: the opening planning commit, one bot commit per unit on its unit
branch, the merges into `integrate/ess-054`, review-correction commits, and one closing planning commit
per wave. Not a push, not a tag, not a release. 0.54.0 is cut through the repository's own release
procedure after the waves close and after 0.53.0 has landed on `main`.

## Planning record

- 36 stories tagged `ess-0.54.0`: `story:feature-request-<N>` for #426–#430 and #432–#459,
  `feature-request-426a`, `feature-request-426b`, and `story:ess-generate-check` (split out of #435).
- Fit reviews: 9 read-only reviewers, one per issue family (`general-purpose`), bodies written by the
  coordinator through `aep plan artifact new --from`.
- Plan critics, two rounds, agent types `aep:plan-critic-acceptance`, `aep:plan-critic-design`,
  `aep:plan-critic-scope`, `aep:plan-critic-parallel-safety`. Round 1: 4 × `needs-revision`, 55 findings,
  55 fixed. Round 2: 4 × `needs-revision`, 28 findings, 28 fixed. Records `review-result:ess-054-plan-*-r1`
  and `-r2`; one `review_outcome` per finding. Revisions were drafted by a `general-purpose` agent and
  written by the coordinator. No third round.
- Two critic reports carried unquoted YAML in their findings blocks; the coordinator single-quoted each
  `message:` value so the store could read them (content unchanged).

## Pre-flight

| check | value |
|---|---|
| free on `/` | 31G (shared with other sessions; fell to 0 twice during planning) |
| `/dev/shm` used | 1G of a per-user quota near 26G |
| build dirs | `/dev/shm/ess-054/<unit>`, one per unit, `CARGO_INCREMENTAL=0`, debug info off |
| concurrent agents | 4 (no model budget stated; skill default) |
| shared invariants | `<scratch>/invariants.md`; one brief per unit under `<scratch>/<unit>/brief.md` |
| selection | `aep plan artifact waves` (aep 0.68.0) for collisions; units group stories joined by `depends_on` |

`<worktrees>` is the managed worktree root, `<scratch>` the wave's cache directory; neither is committed.

## Units

Agent types: implementor `aep:implementor`, adversary `aep:adversary`.

| unit | wave | stories | worktree id | branch | build dir | stage |
|---|---|---|---|---|---|---|
| W1-1 | 1 | 429, 458 | `ess-054-w1-1` | `unit/ess-054-w1-1-ess23-identity-state` | `/dev/shm/ess-054/W1-1` | planned |
| W1-2 | 1 | 427, 428, 430 | `ess-054-w1-2` | `unit/ess-054-w1-2-conformance-defects` | `/dev/shm/ess-054/W1-2` | planned |
| W1-3 | 1 | 441, 442, 443, 444, 446, 447 | `ess-054-w1-3` | `unit/ess-054-w1-3-read-api-idioms` | `/dev/shm/ess-054/W1-3` | planned |
| W1-4 | 1 | ess-generate-check, 435, 436, 432, 433, 453, 457 | `ess-054-w1-4` | `unit/ess-054-w1-4-adoption-docs` | `/dev/shm/ess-054/W1-4` | planned |
| W2-1 | 2 | 452 | | | | planned |
| W2-2 | 2 | 438, 439 | | | | planned |
| W2-3 | 2 | 448, 426b, 426a | | | | planned |
| W2-4 | 2 | 454, 455, 456, 451, 449 | | | | planned |
| W3-1 | 3 | 459 | | | | planned |
| W3-2 | 3 | 450, 445, 440 | | | | planned |
| W3-3 | 3 | 434, 437 | | | | planned |

`story:feature-request-426` is the #426 decision record; it closes when 426a and 426b close.

## Cross-unit overlaps inside a wave (from `aep plan artifact waves`)

Each is a different function or a different table row; the coordinator runs
`git merge-tree --write-tree` before the second unit of each pair merges.

| wave | units | paths |
|---|---|---|
| 1 | W1-1, W1-2 | `crates/edge/ess-xtask/src/docs.rs` (format rows), `ess-conformance/src/synthesize.rs`, `website/docs/reference/spec-versions.md` |
| 1 | W1-1, W1-4 | `ess-conformance/src/interpret/execute.rs` |
| 2 | W2-1, W2-4 | `ess-conformance/src/synthesize.rs` |
| 2 | W2-2, W2-3 | `ess-primitives/src/predicate.rs` |
| 2 | W2-2, W2-4 | `website/docs/reference/predicates.md` |
| 2 | W2-3, W2-1 | `ess-domain/src/command.rs` (`TryFrom<RawOutcome> for Outcome`, disjoint ranges) |
| 2 | W2-3, W2-4 | `ess-domain/src/entity.rs`, `guards-and-predicates.md` |
| 3 | W3-2, W3-1 | `ess-domain/src/command.rs`, `ess-diff/src/change.rs` |
| 3 | W3-3, W3-2 | `ess-compiler/src/resolve.rs` |

`CHANGELOG.md` and the `ess/23` row of `spec-versions.md` beyond 429's row are coordinator-owned: units
report their lines.
