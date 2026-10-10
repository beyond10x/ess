# Wave 3 of `epic:one-selection-plan` — three consumers read the plan

> **Status: approved 2026-10-07 by the operator (option A).** Written on `integrate/selection-plan` `0f4e89a2c`
> (wave 2 merged in; PR #488 open, its merge to `main` held until the ESS 0.56.0 release lands,
> conductor DEC-20261007-91). Skill: `aep:implementing` 0.20.1, wave mode. The wave stacks on the
> same integration branch and is delivered by PR #488's successor, or by #488 itself if it has not
> merged when this wave closes.

## 1. Pre-flight — measured 2026-10-07

| fact | evidence |
|---|---|
| root filesystem 80 G free of 848 G, 91 % | `df -h /` |
| ess linked trees: 9 (this wave's integration tree is one; none of this wave's units exist yet) | `git worktree list` |
| `aep 0.68.0` | `aep --version` |
| one measured unit build: U1 of wave 2 (`ess-domain`, `ess-compiler`, five `ess-conformance` targets) reached 9.4 G | `du -sh` before its cache was discarded |

## 2. Units

| unit | story | crate | scope (cited) |
|---|---|---|---|
| U1 | `story:entity-runtime-lowering-reads-selection-plan` | `ess-entity-runtime` | `src/lib.rs` (`lower_command`'s `sort_by_key`) |
| U2 | `story:generated-behaviour-reads-selection-plan` | `ess-synth` | `src/rust/behaviour.rs`, `src/go/behaviour.rs`, `src/determined.rs` |
| U3 | `story:validation-reads-selection-plan` | `ess-domain` | `src/command.rs`, `src/command/subject_state.rs`, `src/command/related_guard.rs` |

All three serve `vision:O2` and depend only on `story:selection-plan-design-and-type` (implemented).
Each reads `ess_compiler::ir::PrecedencePlan` / `ess_domain::command::precedence` and the scoped
phase-order override wave 2 delivered.

### `aep plan artifact waves --kind story --status draft`, this epic's stories

Selection path: the verb (`aep 0.68.0`).

```text
wave 1: story:entity-runtime-lowering-reads-selection-plan
wave 1: story:generated-behaviour-reads-selection-plan
wave 1: story:validation-reads-selection-plan
wave 3: story:interpreter-reads-selection-plan
wave 3: story:synthesis-reads-selection-plan
wave 4: story:overlap-witnesses-per-phase
```

Collisions naming this wave's stories or the interpreter story:

```text
{"a":"story:feature-request-361","b":"story:interpreter-reads-selection-plan","path":"crates/verify/ess-conformance/src/interpret/execute.rs","confidence":"inferred"}
{"a":"story:feature-request-362","b":"story:interpreter-reads-selection-plan","path":"crates/verify/ess-conformance/src/interpret/execute.rs","confidence":"inferred"}
{"a":"story:generated-behaviour-reads-selection-plan","b":"story:rust-binding-unused-event","path":"crates/generate/ess-synth/tests","confidence":"inferred"}
{"a":"story:interpreter-reads-selection-plan","b":"story:the-interpreter-executes-stored-field-guards","path":"crates/verify/ess-conformance/src/interpret/execute.rs","confidence":"cited"}
```

Unassessed: none of this epic's stories.

**The verb and the epic's table disagree, and the verb wins.** The epic put the interpreter, the
lowering and the emitters in this wave and validation in the last. The verb keeps the interpreter
out of its first wave because three draft stories outside the epic also scope
`interpret/execute.rs` (`feature-request-361`, `feature-request-362`,
`the-interpreter-executes-stored-field-guards`), and brings validation forward because its only
dependency is implemented and its files collide with nothing drafted.

## 3. Unit records

| unit | managed worktree id | branch | build directory | scratch root | stage |
|---|---|---|---|---|---|
| integration | `ess-selection-plan` | `integrate/selection-plan` | its own `target/` | — | at `0f4e89a2c` |
| U1 | `ess-selection-plan-w3-u1` | `unit/selection-plan-w3-u1-lowering` | its own `target/` | `~/.cache/ess-selection-plan/w3-u1-scratch` | merged `7425b6ebf` |
| U2 | `ess-selection-plan-w3-u2` | `unit/selection-plan-w3-u2-emitters` | its own `target/` | `~/.cache/ess-selection-plan/w3-u2-scratch` | merged `4d923b0e8` |
| U3 | `ess-selection-plan-w3-u3` | `unit/selection-plan-w3-u3-validation` | its own `target/` | `~/.cache/ess-selection-plan/w3-u3-scratch` | merged `f32c29191` |

Dispatch types: `aep:implementor` per unit, then `aep:adversary` per unit, at most two passes each.

## 3a. Close — 2026-10-08

| unit | unit commit | merged | adversary passes | verification |
|---|---|---|---|---|
| U1 lowering | `ac263b14a` | `7425b6ebf` | 2: 5 findings fixed (the compiler-table re-pin and one doc line by the coordinator) | lowered definitions of 239 models identical to the base table except the added exception fixture; 16 requests answer alike in base and plan order on entity_core |
| U2 emitters | `293531f9a` | `4d923b0e8` | 1: no answer change; F1 (probe blind spots) pinned, F2 (precheck) covered by the adversary's cases; no implementation change followed, so no second pass | adversary base-against-unit comparison of 9,720 Rust and 9,600 Go models identical; E-U2/E-U6 309 of 309 |
| U3 validation | `c7dea8ca5` | `f32c29191` | 2: 4 findings fixed (per-pair tie-breaks, then the earliest-read phase) | 18,530 (model, check) outputs identical to the base, 131 refused, same text |

Integrated package gate on `4d923b0e8`: `ess-domain` 147, `ess-compiler` 64,
`ess-entity-runtime` 43, `ess-synth` 104 result blocks, all ok; clippy and fmt exit 0.

Decided during the wave: validation refuses a `when: true` refusal (Timo, option A;
beyond10x/ess#489, PR #490), after four consumers answered that shape differently. R2
(`wrong_state:` beside a present-related refusal and acceptance) stays refused by validation:
this epic changes no verdict.

Disk: the wave paused about 40 minutes at conductor's hold (DEC-20261007-106) after `/` fell to
2 G; caches were discarded per unit as each merged.

Cost per agent (as the harness reported: tokens / tool uses / wall seconds):

| agent | rounds |
|---|---|
| U1 implementor | 201,432 / 95 / 779; 262,554 / 53 / 782 |
| U2 implementor | 342,002 / 65 / 2,499; 355,348 / 200 / 274 (two runs cut by HTTP 429 and resumed) |
| U3 implementor | 286,237 / 138 / 1,683; 334,703 / 26 / 329; 374,121 / 30 / 779; 409,365 / 24 / 627 |
| U1 adversary | 228,103 / 99 / 1,934; 305,960 / 140 / 410 |
| U2 adversary | 328,692 / 139 / 3,172 |
| U3 adversary | 160,245 / 47 / 1,780; 173,627 / 57 / 1,971; 253,043 / 83 / 542 |

## 4. Commits this wave makes

The opening store commit (this page, three stories moved to `active`) on `integrate/selection-plan`;
one commit per unit on its branch; the three merges into `integrate/selection-plan`; the closing
store commit; the push of `integrate/selection-plan`. If PR #488 has merged by then, one new pull
request against `main`; otherwise the push updates #488. Merged through the App when `Gate` is
green and the conductor's hold allows. Nothing else: no tag, no release, no later wave.
