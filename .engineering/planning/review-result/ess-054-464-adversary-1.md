---
format: aep.planning-md/3
id: review-result:ess-054-464-adversary-1
kind: review-result
status: active
title: 'Adversary pass 1, #464 external witness beside held-state guards'
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-464
revision: 1
---
unit: ess-054-464 (story:feature-request-464), commit `bafb78c55` (range `bc4884203..bafb78c55`) in `~/.local/state/worktree/trees/b10x/ess/ess-054-464-20261006`, plus my uncommitted test file
verdict: NEEDS-CHANGE
cases: executed 52→71, red 8
origin: introduced 4 / pre-existing 4 / undecided 0
wrote-outside-worktree: 5 paths
needs-coordinator: decide whether the 4 pre-existing shapes (rows 5–8) are fixed in this unit or split out; delete my two scratch build dirs (part 6)

## 1. `git --no-pager diff --stat`

No tracked file is changed. The only change is one new untracked test file:
```
?? crates/verify/ess-conformance/tests/adversary_464_pass1.rs
 .../ess-conformance/tests/adversary_464_pass1.rs   | 1104 ++++++++++++++++++++
```
I took no worktree lease, because the invariants forbid worktree commands.

## 2. Cases added (`crates/verify/ess-conformance/tests/adversary_464_pass1.rs`)

Each red case was run alone first. The output below is from that first run.

| Case (line) | Asserts | Now | Red output (excerpt) |
|---|---|---|---|
| `…external_driver_beside_when_subject_reaches_its_state` (:307) | A state reached only through external `unlisted` is driven with an input `stale` does not claim | red | `refusal[ESS-SYNTH-004]: … no scenario demo.desk.Pick/state/Unlisted/refuses/demo.desk.CheckPick … the route runs through demo.desk.CheckPick/unlisted, which no input reaches` |
| `…subjectless_external_refusal_beside_when_subject_state` (:461) | An error-only external `offline` beside `shut` passes the interpreter | red | `demo.doors.Operate/outcome/offline: Failed … observed: no declared outcome was reached` |
| `…subjectless_external_refusal_beside_when_subject` (:469) | An error-only external `unavailable` beside `stale` passes | red | `expected: outcome = unavailable / observed: outcome = wrong-state`; steps arrange a `pick`, then send `pick_id = "00000000-0000-4000-8000-c5788a0d9200"` |
| `…external_refusal_beside_stored_reference…` (:502) | Error-only external beside `when_related: {via: blocked_by}` passes | red | `CompleteTask/outcome/scheduler-down … observed: outcome = wrong-state` |
| `…external_move_beside_stored_reference…` (:512) | Moving external beside the same guard passes | red | `CompleteTask/outcome/forced … observed: outcome = blocker-missing` |
| `…external_guard_over_compared_field_is_witnessed` (:576) | `unlisted` with `when: revision > 5` beside `stale` is written | red | `refusal[ESS-SYNTH-003] … no candidate of the 14 tried satisfies … none of stale (revision != input.revision)` |
| `…rewitnessed_held_state_external_writes_a_new_value` (:933) | A re-witnessed `stalled` that `sets: {label: input.label}` sends a label the door does not already hold | red | `Gentle: demo.doors.Operate/outcome/stalled: label Literal { value: Text("label") } is what the door already holds` |
| `…rewitnessed_stored_guard_external_writes_a_new_value` (:974) | The same for `unlisted` with `sets: {note: input.note}` | red | `demo.desk.CheckPick/outcome/unlisted: note Literal { value: Text("note") } is what the pick already holds` |
| `…unclaimed_external_witnesses_keep_their_base_bytes` (:1055) | Five external scenarios that no guard claims keep their base digests | green now, red on mutants | On the mutated copy: `door: demo.doors.Operate/outcome/stalled: base 4856:38de8940626a100, here 5372:f57ab15df1f11533` and 3 more |

The other 10 cases are green, including a control at :891 and a dump case at :839 that only writes when an environment variable is set.

Two of my first cases did not compile into a valid model, because validation refuses the shape. They are not findings: I recast one (:370) and deleted the other.

## 3. Suite run (after the cases existed)

`cargo test -p ess-conformance --test adversary_464_pass1 --test external_beside_held_guard --test guarded_external --test retained_replay --locked --offline --no-fail-fast` exited 101:
- `adversary_464_pass1`: 11 passed, 8 failed
- `external_beside_held_guard`: 10 passed
- `guarded_external`: 6 passed
- `retained_replay`: 36 passed

The "before" count of 52 is the same run with my file left out.

Clippy on `--test adversary_464_pass1` with 1.98.1 and with stable: both exit 0. `cargo fmt -p ess-conformance --check`: exit 0.

## 4. Findings

| # | file:line | What breaks | Case | Verdict / origin / severity | What reaches it |
|---|---|---|---|---|---|
| 1 | `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:2039` | A witness exists (a pick created and checked at revision 6), but synthesis refuses with ESS-SYNTH-003. The row search hints leave out the branch's own `when:`. Base wrote a failing scenario here. | :576 | NEEDS-CHANGE / introduced / warning | The story's own second adopter: `amount != input.amount` beside an external decline guarded by `amount >= 1000`. No repository model has this shape. |
| 2 | `crates/verify/ess-conformance/src/synthesize.rs:3736` | The re-witnessed input skips `freshened` (beyond10x/ess#161), so a target that never writes the field passes. Base moved the value off the row. | :933 | NEEDS-CHANGE / introduced / warning | Any claimed external that `sets:` a field from its input |
| 3 | `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:2074` | Same defect on the stored-field path | :974 | NEEDS-CHANGE / introduced / warning | Same |
| 4 | `crates/verify/ess-conformance/tests/external_beside_held_guard.rs:885` | The acceptance test drops every scenario that could change from its comparison, and its base table has none outside the new fixture. Two mutants pass all 52 unit tests (`~/.cache/ess-054-wave/464-adv/mutants.diff`): M1 makes `held_state_claims` ignore refutation; M2 drops the plain-witness keep in `external_witness`. | :1055 catches both | NEEDS-CHANGE / introduced / warning | The acceptance item `repository_model_suites_change_only_where_claimed` |
| 5 | `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:2022` | An error-only external on a command with a stored-field guard is written as a scenario the interpreter fails. The head arranges a row but sends a literal identity naming nothing. The control without `stale` (:891) passes. | :469 | CONFIRMED / pre-existing (fails at base) / warning | Error-only externals are common in repository fixtures (`binding-arrangement.yaml` `unavailable`) |
| 6 | `crates/verify/ess-conformance/src/synthesize.rs:3713` | `unclaimed_external` returns early when the outcome names no subject. The failing scenario has the same bytes at base and head. | :461 | CONFIRMED / pre-existing / warning | Same shape beside `when_subject_state` |
| 7 | `crates/verify/ess-conformance/src/synthesize/related_guard.rs:184` | The known stored-reference gap writes a failing suite, not a refusal | :502, :512 | CONFIRMED / pre-existing / warning | The repository's `related-guard-stored-reference.yaml` plus one external branch |
| 8 | `crates/verify/ess-conformance/src/synthesize.rs:4885` | `advance` routes the external driver through `subject_fact::step`, which never selects an external branch. The ESS-SYNTH-004 text "no input reaches `CheckPick/unlisted`" is false: that scenario is written and passes. | :307 | CONFIRMED / pre-existing (refused at base too) / note | A state reached only through an external branch of a command with a `when_subject` guard. Nothing found in the repository. |

Fixes I would make (none applied):
- **Rows 2 and 3:** pass the chosen input through `existence::fresh_created` and `freshened`, as `arranged_plain` does.
- **Row 1:** translate the branch's own guard into the row-search hints.
- **Rows 5 and 6:** bind the arranged instance into the shared subject's identity field, or refuse.
- **Row 7:** refuse by name where `stored::field(command).is_some()`.
- **Row 4:** adopt my :1055 case.

Judgement-only residue: none beyond these rows.

## 5. Attacked and not broken
- A held-state predicate with an input guard (`state == Picked` plus `when:`), both guard polarities: witnessed, and the interpreter passes.
- An external `when:` identical to `shut`'s guard: refused naming `shut`, both pushes.
- An accepting `when_subject` declared after the external: refused naming `stale` and `same`.
- Two externals; a two-field `any` stored guard; an ess/6 `{field, equals}` enum sibling; an external moving from a later state; an external that moves nothing beside siblings claiming two of its states with no input guard: all pass.
- Base against head on 13 models: every scenario whose bytes changed was failing at base; no passing scenario moved.
- Not reachable, because validation refuses the model: `when_subject_state` beside `when_subject` or `when_related`, and `wrong_state` beside `when_subject_state`.
- Not attacked: the Rust/Go/TypeScript runner lanes (the unit adds no new step kind), and CLI `mutate` on my probe models.

## 6. Paths written outside the worktree
- `~/.cache/ess-054-wave/464-adv/` (206M): logs, `env.sh`, `tmp/`, `dump-base/`, `dump-head/`, `mutants.diff`, `*.orig`, `base-src/` (export of `bc4884203` with my test file added), `mut-src/` (export of `bafb78c55` with mutants M1/M2). `tmp/` also holds two `ess-adversary-retained-go-*` directories left by the existing `retained_replay` tests.
- `/dev/shm/ess-054/464/adv-base` (401M): the base build. Mine to delete; I left it as the origin evidence.
- `/dev/shm/ess-054/464/adv-mut` (507M): the mutant build. Same.
- `/dev/shm/ess-054/464`: the assigned build dir, now also holding my test target and clippy artifacts. Not cleaned, per the dispatch.
- `~/.cache/b10x-go-cache/464`: used by the `retained_replay` Go lane.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2039
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'An external branch guarded by when revision > 5 beside stale (revision != input.revision) is refused ESS-SYNTH-003 although a pick created and checked at revision 6 is a witness, because the row search hints omit the branch''s own guard.'
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 3736
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'The held-state re-witness returns reach_external_beside output without freshened, so a stalled branch that sets label is sent the label the door already holds and a target that skips the write passes (beyond10x/ess#161).'
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2074
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'external_witness returns the searched input without freshened, so an unlisted branch that sets note is sent the note the pick already holds and a target that skips the write passes (beyond10x/ess#161).'
- file: crates/verify/ess-conformance/tests/external_beside_held_guard.rs
  line: 885
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'repository_model_suites_change_only_where_claimed drops every claimable scenario from its comparison, so mutants that re-witness unclaimed externals pass all 52 unit tests while adversary_464_unclaimed_external_witnesses_keep_their_base_bytes catches them.'
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 2022
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: 'An error-only external branch on a command with a when_subject sibling is written as a failing scenario: the head arranges a Pick the send never names, the literal pick_id names no row, and the interpreter answers wrong-state.'
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 3713
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: 'unclaimed_external returns early for an external branch with no subject of its own, so an error-only external beside when_subject_state is sent a literal door_id and the interpreter reaches no declared outcome.'
- file: crates/verify/ess-conformance/src/synthesize/related_guard.rs
  line: 184
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: 'An external branch beside a when_related guard that reads a stored reference is written as a failing scenario (blocker-missing for a move, wrong-state for an error-only branch) instead of being witnessed or refused by name.'
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 4885
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: 'advance drives an external branch of a when_subject command through subject_fact::step, which never selects an external branch, so the Unlisted state refusals are refused ESS-SYNTH-004 claiming no input reaches CheckPick/unlisted while that scenario is written and passes.'
```
