# Wave — the open GitHub issues (#75, #92–#96), releases A–D in one branch

> **Status: approved 2026-09-25 by the operator in session.** Written against `origin/main`
> `d45255cc2f0` plus open PR #90 (merged into this branch as the #75 design basis). Stage 1 waived by
> the operator. **Close differs from the skill default:** all units land on
> `integrate/ess-issues-20260925`, which is delivered as **one pull request**; the wave does not
> merge into `main`, tag or release.

## 1. Pre-flight — measured 2026-09-25

| fact | evidence |
|---|---|
| root filesystem 60 G free of 848 G, 93 % | `df -h /` |
| `ess` primary checkout 67 behind, one untracked review file | `git status -sb`; left untouched |
| 80 linked trees on the ess repository | `git worktree list`; none belong to this wave |
| `aep --version` | `aep 0.59.3` |
| gates `publish` refuses ess pushes since the merge-queue ruleset change | ess PR #99, `exact App-only branch authority unavailable`; blocks the final push until gates accepts the exclude |

## 2. Units and order

| unit | stories | issue | depends on | surface |
|---|---|---|---|---|
| A1 | `optional-guards-mean-what-they-say`, `list-and-text-guards-are-synthesized` | #93, #94 | — | `ess-primitives`, `ess-compiler/src/expression.rs`, `ess-conformance`, evaluator lanes |
| A2 | `predicate-reference-page` | #92 | A1 decisions (written to them) | `website/docs/reference/predicates.md`, `website/sidebars.ts`, one new `ess-cli` test |
| B0 | (prerequisite of `string-prefix-suffix-substring-operators`) | #95 | — | **entity-runtime** `crates/entity-core` |
| B1 | `string-prefix-suffix-substring-operators` | #95 | A1 merged, B0 pushed | predicate grammar, type check, finite coverage, 3 lanes, emitters, `ess-entity-runtime`, suite format gate, design page |
| C | `stored-field-guards` | #75 | A1 merged | per `docs/design/cross-record-and-stored-field-guards.md` (ess/8) |
| D1 | `aggregate-views` (design page) | #96 | — | `docs/design/aggregate-views.md` |
| D2 | `aggregate-views` (implementation) | #96 | C merged, D1 merged | per D1's page |

Parallel now: A1, A2, B0, D1. Then B1 ∥ C. Then D2. B1 and C both touch format versions and the
witness module; they merge one at a time with a `git merge-tree` dry run first. At most two ess
units compile at once (disk). Dispatch types: `aep:implementor`, then `aep:adversary`, per unit.

Operator defaults taken: #93 refuses `== null`; #94 synthesizes lists, text byte-wise; #96 open
questions answered in `story:aggregate-views`.

## 3. Unit records

Build directories are `b10x-target/ess-wave-20260925-<unit>` and scratch roots
`ess-wave-20260925/<unit>-scratch`, both under the user cache.

| unit | managed worktree id | branch | stage |
|---|---|---|---|
| integration | `ess-wave-20260925-int` | `integrate/ess-issues-20260925` | open |
| A1 | `ess-wave-20260925-a1` | `impl/a1-guard-witness-agreement` | dispatched |
| A2 | `ess-wave-20260925-a2` | `impl/a2-predicate-reference-page` | dispatched |
| B0 | `er-wave-20260925-b0` (entity-runtime) | `feat/string-prefix-suffix-conditions` | dispatched |
| D1 | `ess-wave-20260925-d1` | `impl/d1-aggregate-views-design` | dispatched |
| B1, C, D2 | — | — | waiting on dependencies |

## 4. Commits this wave makes

One commit per unit, the merges into `integrate/ess-issues-20260925`, the opening and closing store
commits, one entity-runtime branch commit for B0 (published for the ess pin, with its own pull
request), and the push of the integration branch with one pull request against `main`. No merge
into `main`, no tag, no release.
