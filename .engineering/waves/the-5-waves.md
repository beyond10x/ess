# the-5-waves

Eighteen stories under `epic:retrofit-findings-round-3` (beyond10x/ess #162–#176, #178, #179) and
three follow-up stories from retrofit wave 2, in five waves. Operator, 2026-09-27: "prepare the
waves … i want ALL waves to merged into local integration branch, then in the end one final PR
against origin/main"; goal: "the-5-waves completed, integrated on origin, new release cut".

Skill: aep 0.14.17 (`aep:implementing`, wave mode). Dispatch types: `aep:story-scoper`,
`aep:implementor`, `aep:adversary`. N = 4 (default budget; no budget stated).

## Commits this approval authorises

One commit per unit, the merges of unit branches into `integrate/the-5-waves`, the store commits
(scope, activation, evidence, closing), the `ess/16` registration commit (`e9c437303`), and after
the fifth wave: one PR `integrate/the-5-waves` → `main`, its merge, and the release (the goal names
it). Nothing else.

## Integration

| | |
|---|---|
| branch | `integrate/the-5-waves` |
| base | `472d35fbe` (main after #181) |
| registration | `e9c437303` — `ess/16` admitted; every unit forks from the integration head at dispatch |
| worktree | managed `ess-the-5-waves` |
| build dir | `~/.cache/b10x-target/ess-5w-int` |

## Decisions taken by the coordinator

- One source format for the whole release: `ess/16`. Units gate their construct on
  `FormatVersion::V16` and do not edit the registration files.
- One suite-format pair for every round-3 suite construct: `ess-conformance/26` (ordinary) and
  `/27` (coverage). The first unit that needs it (nested-struct, wave 1) registers it; later units
  add their construct to its `used_by`. Go and TypeScript runtimes refuse /26 and /27 by version, as
  they do /20–/25.
- `story:input-guard-overlap-precedence`: stated precedence (an input-guarded refusal wins over an
  accepting branch it overlaps), witnessed at the overlap point; no format change.
- `story:view-paging-and-caller-filters`: paging only; the caller-supplied free-form filter is
  declined (reasons in the story's derived scope).
- `story:consumer-imports-owner-types`: option 3, a composition-level conformance assertion.
- `story:upsert-outcome-by-existence`: widened to create-or-refuse (#164 follow-up comment).
- `story:ungrouped-aggregate-views-are-witnessed`: a delta assertion over a snapshot (the target is
  never assumed empty, `docs/design/aggregate-views.md:445-447`); refusal only if no delta can be
  stated.
- `view-filters` and `ungrouped-aggregate` are two units (the scoper found them disjoint apart
  from the shared `shows` helper).

## Waves

`aep plan artifact waves` reads the typed scope entries at file granularity; almost every unit
names `crates/verify/ess-conformance/src/synthesize.rs`, so the verb's lists are recorded below
once written, and the waves are assembled by function-level disjointness declared per brief, with
a `git merge-tree` dry run before the second merge of each wave.

| wave | unit | story | issues | main symbols |
|---|---|---|---|---|
| 1 | literal | `literal-fallback-after-else` | #163 | `command.rs` `else` parse; `value_expression.rs`; `expression_value` `InputOrGenerated` arm |
| 1 | leaves | `nested-struct-per-leaf-comparison` | #179 | `expression_value` `Struct` arm, `determined_payload`, `settled`; `runner.rs` `expect_payload`; suite /26–/27 |
| 1 | defined | `defined-over-optional-aggregates` | #176 | `expression.rs` `Defined`; `ess-primitives` facts; `input.rs`; `runner.rs` `row_facts`; Go/TS `predicate` |
| 1 | whensubj | `when-subject-witness-and-diagnostics` | #172, #173 | `refused_here`, `reach`, `plain_guards`; `synthesize/subject_fact.rs` |
| 2 | narrow | `optional-input-narrowed-after-refusal` | #169 | `command.rs` payload/sets typing; `resolve.rs` `crossing` |
| 2 | overlap | `input-guard-overlap-precedence` | #178 | `admits_plain`, `boundary_inputs` |
| 2 | search | `witness-search-beyond-64-candidates` | #155 follow-up | `witness.rs` `enumerate`, `candidates` |
| 2 | filters | `view-filters-witnessed-on-matching-rows` | — | `view_expectations`, `arrange_beside` |
| 3 | aggdelta | `ungrouped-aggregate-views-are-witnessed` | #148 follow-up | `synthesize/aggregate.rs`; delta step |
| 3 | related | `related-record-value-source` | #166 | `PayloadSource` related variant; `arrange_owner` |
| 3 | absent | `absent-command-input-outcome` | #170 | `OutcomeCondition`; `ExecuteCommand` without input |
| 3 | now | `current-time-guard-operand` | #171 | `Operand` now; `ScenarioValue`; clock |
| 4 | caller | `caller-value-source-and-guard` | #168 | `actor.rs`; caller source; adapter protocol |
| 4 | upsert | `upsert-outcome-by-existence` | #164 | existence-selected outcomes; `unknown_instance` pattern |
| 4 | seteff | `set-effects-over-filtered-instances` | #167, #175 | `RawOutcome` `instances:`/`affects:`; `ResolvedInstance` |
| 4 | retry | `bounded-retry-bindings` | #165 | `binding.rs`; on-failure synthesis; invocation count |
| 5 | paging | `view-paging-and-caller-filters` | #174 | `view.rs` paging; `ViewExpectation` page/total |
| 5 | compose | `consumer-imports-owner-types` | #162 | `ess-composition` conformance assertion |

## Units

Triple per unit: worktree (managed id `ess-5w-<unit>`), build dir
`~/.cache/b10x-target/ess-5w-<unit>`, scratch `~/.cache/ess-5w/<unit>/scratch`.

| unit | branch | head | stage |
|---|---|---|---|
| literal | `impl/literal-fallback-after-else` | — | planned |
| leaves | `impl/nested-struct-per-leaf-comparison` | — | planned |
| defined | `impl/defined-over-optional-aggregates` | — | planned |
| whensubj | `impl/when-subject-witness-and-diagnostics` | — | planned |

## Pre-flight (2026-09-27 20:51)

- `/` 118G free; `/dev/shm` 32G free; sccache wired (`~/.cargo/config.toml` `rustc-wrapper`).
- Load average 21 on 20 cores from other users: every build `nice -n 19`, 4 jobs, 4 test threads.
- Primary checkout carries another session's untracked `docs/reviews/2026-09-25-consumer-request-input-constraints.md`; no unit touches it.

## Disk rule learned 2026-09-27

Prune a unit build dir (`debug/deps` executables) only when no agent of that unit is running. On
2026-09-27 the coordinator pruned `ess-5w-w1fix` while its adversary ran the ess-cli suite, and 47
targets reported "never executed". Delete a unit build dir as soon as the unit merges; delete
`ess-5w-int` between merges; the disk floor is 10G.

Keeping only the newest build of each test executable older than 60 minutes (older hashes of the
same test name) is safe while its agent runs, and recovered 28G on 2026-09-28 (absent 28G → 14G).
A background loop does it every 10 minutes during the waves.
Names repeat across crates (`current_time_guard` in four crates), so a 5-minute threshold deleted
current binaries of other crates and 44 targets reported "never executed" on 2026-09-28.
