# Concurrent history — wave 5

Follow-up to `epic:concurrent-history-conformance` (waves 1–4 on the page
`concurrent-history-wave-1.md`). Operator, 2026-09-28: "plan out the next wave and then use /wave to
implement it with multiple agents". Skill: aep 0.16.0 (`aep:planning`, `aep:implementing` wave mode).
Store: `aep.project/3`; writes go through a private `aep 0.61.1` because the installed 0.62.0
refuses this format (see the wave-1 page).

## Stories filed for it (2026-09-28)

| story | from |
|---|---|
| `story:recorded-log-adapter-domain` | recorded-history implementor: `ess-history-adapter/1` has no ESS domain |
| `story:explorers-record-view-reads` | runner and fault-injection implementors: Go/TS explorers write no `rows` |
| `story:concurrent-history-records-inputs` | checker adversary pass 2, finding 4 |

All three `serve vision:O2`; none decomposes the epic. Critic panel not run: the three are
follow-ups filed from findings, not the decomposition of one parent (`aep:planning` § 7).

## `aep plan artifact waves --kind story --status draft --format json`

Selection path: computed by the verb (`aep 0.61.1`), from typed scope written 2026-09-28 by three
`aep:story-scoper` runs. For these three stories: wave 1 `story:concurrent-history-records-inputs`;
wave 2 `story:explorers-record-view-reads`, `story:recorded-log-adapter-domain`. Collisions
(verbatim pairs):

| a | b | path | confidence |
|---|---|---|---|
| `story:concurrent-history-records-inputs` | `story:explorers-record-view-reads` | `crates/edge/ess-cli/tests/explore_concurrent.rs` | inferred |
| `story:concurrent-history-records-inputs` | `story:explorers-record-view-reads` | `crates/verify/ess-conformance/src/go/explore.go` | cited |
| `story:concurrent-history-records-inputs` | `story:explorers-record-view-reads` | `crates/verify/ess-conformance/src/ts/explore.ts` | cited |
| `story:concurrent-history-records-inputs` | `story:recorded-log-adapter-domain` | `crates/verify/ess-conformance/src/recorded.rs` | inferred |

Unassessed: none of the three. Cycles: none. Full output: `waves-w5.json` in the coordinator scratch.

Coordinator ordering: the verb's wave 2 runs first as wave 5 (two units, as asked); records-inputs is
wave 6. Both depend on `story:declared-fault-injection` landing first (its explorer and history
changes), so neither is dispatched before it merges.

## Units

Triple per unit: worktree `ess-chc-<unit>`, build dir `~/.cache/b10x-target/ess-chc-<unit>`, scratch
`~/.cache/ess-chc/<unit>/scratch`.

| unit | story | serves | scope | branch | head | stage |
|---|---|---|---|---|---|---|
| adapter | `story:recorded-log-adapter-domain` | `vision:O2` | cited: `models/recorded-log-adapter`, `recorded.rs`; inferred: schema, `ess-xtask` test, `Taskfile.yml` | `impl/recorded-log-adapter-domain` | — | planned |
| reads | `story:explorers-record-view-reads` | `vision:O2` | cited: `src/go/explore.go`, `src/ts/explore.ts`; inferred: explore fixtures, `explore_concurrent.rs` | `impl/explorers-record-view-reads` | — | planned |

## Commits this approval authorises

One commit per unit, their merges into `integrate/concurrent-history`, the store commits (scope,
activation, reviews, evidence, close) and this page. The PR and release are the goal's, not this
wave's.
