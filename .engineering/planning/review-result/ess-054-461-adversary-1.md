---
format: aep.planning-md/3
id: review-result:ess-054-461-adversary-1
kind: review-result
status: active
title: 'Adversary pass 1, #461 complete when_subject refusals'
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-461
revision: 1
---
unit: ess-054-461-adv, uncommitted working tree `~/.local/state/worktree/trees/b10x/ess/ess-054-461-20261005` on base `97271a43d`
verdict: CONFIRMED
cases: executed 9→18, red 4
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (scratch `~/.cache/ess-054-461/adv`; build dir and Go cache, both deleted)
needs-coordinator: severity of F1 and F2. I rated both `warning`; decide whether either holds the unit.

**1. `git --no-pager diff --stat`**
```
 crates/specify/ess-compiler/src/ir.rs              |   4 +-
 crates/specify/ess-compiler/src/resolve.rs         |  20 +-
 crates/specify/ess-domain/src/command.rs           |  13 +-
 .../specify/ess-domain/src/command/subject_fact.rs |  21 +-
 crates/verify/ess-conformance/src/synthesize.rs    |  61 ++++--
 .../ess-conformance/src/synthesize/disclosure.rs   |   1 +
 .../ess-conformance/src/synthesize/subject_fact.rs | 226 ++++++++++++++++++++-
 .../design/cross-record-and-stored-field-guards.md |  26 +++
 .../docs/guides/specify/guards-and-predicates.md   |  31 +++
 9 files changed, 365 insertions(+), 38 deletions(-)
```
All of those changes are the implementor's. My only write in the worktree is one new untracked test file, `crates/verify/ess-conformance/tests/adversary_461_pass1.rs`. No production file was touched.

**2. Cases added** (file `~/.local/state/worktree/trees/b10x/ess/ess-054-461-20261005/crates/verify/ess-conformance/tests/adversary_461_pass1.rs`)

| case | asserts | now |
|---|---|---|
| `adv461_precedence_earlier_state_refusal_adds_no_synthesis_refusal` | an earlier `removed-refused: state == Removed` adds no synthesis refusal at ess/23 compared with ess/22 | **red** |
| `adv461_claimed_row_that_writes_while_refusing_fails_in_each_state` | a target that answers `not-active` but writes the description fails, on a Suspended record and on a Removed record | **red** (Removed) |
| `adv461_not_active_sends_a_description_that_differs_from_the_stored_one` | each `not-active` send carries a description different from the one stored | **red** |
| `adv461_field_equals_refusal_compiles_complete` | a `{field, equals}` refusal validates and carries `complete_refusal`, as the docs claim | **red** |
| `adv461_precedence_model_honest_target_passes_every_scenario` | the precedence model passes an honest target, and `not-active` is fully observed | green |
| `adv461_refusal_that_moves_the_record_fails` | a target that answers `seed-change-refused` but moves the record fails | green |
| `adv461_state_refusal_beside_wrong_state` | `state != Active` beside `wrong_state:`: no new refusal, 2 complete rows, wrong-state snapshots unchanged, honest target passes, a mover fails | green |
| `adv461_go_runner_gives_the_reference_verdicts` | Go runner matches the Rust verdicts for the honest target and 3 faults, and kills each fault | green |
| `adv461_typescript_runner_gives_the_reference_verdicts` | the same check on the TypeScript runner | green |

Two first attempts failed for my own mistakes, not for findings, and I fixed them:
- An error name I added collided with an existing event name.
- My first `{field, equals}` variant on `SuspendInstance` was refused at validation. That refusal is the same root cause as F4.

Red output from each case run alone (`cargo test -p ess-conformance --test adversary_461_pass1 -- --exact <case>`, exit 101 each):
```
adversary_461_pass1.rs:324:5: a valid model whose earlier `state == Removed` refusal answers every removed record gains a synthesis refusal at ess/23 ...
  left: {"refusal[ESS-SYNTH-003]: outcome demo.inst.UpdateInstance/not-active has no scenario `demo.inst.UpdateInstance/outcome/not-active`\n  no candidate of the 3 tried satisfies ``demo.inst.Instance` stored state selecting this branch, over the rows 3 bounded arrangements left`..."}
 right: {}
```
```
adversary_461_pass1.rs:374:9: a target that refuses `not-active` on a Removed record and writes its description fails: { ... "demo.inst.UpdateInstance/outcome/not-active": "passed", ... }
```
```
adversary_461_pass1.rs:740:9: instance: `not-active` is sent the description the record already holds ...
  left: Some(Literal { value: Text("description") })
 right: Some(Literal { value: Text("description") })
```
```
adversary_461_pass1.rs:395:13: a `{field, equals}` refusal naming no subject of its own is refused at validation ...: [conflicting_declaration] command.demo.inst.UpdateInstance.outcomes.seed-change-refused.when_subject: a subject-state guard requires an existing moves or updates subject and input identity
```

**3. Suite run, after the cases existed**

`cargo test -p ess-conformance --locked --offline --no-fail-fast --test subject_fact_complete_refusal --test adversary_461_pass1` → **EXIT=101**
- `adversary_461_pass1`: 5 passed, 4 failed.
- `subject_fact_complete_refusal`: 9 passed, 0 failed.
- The "before" count of 9 is the implementor's own `green-conformance.log`.
- I did not re-run the ess-domain and ess-diff targets (implementor: 1 and 2 passed); nothing I added touches them.
- `rustfmt --check` on the new file: exit 0.

**4. Findings**

- **F1** at `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:5202` (and `:5212`). Warning, CONFIRMED, introduced.
  - **Measured:** `claimed_states` decides which states a refusal claims from its own predicate only, and ignores refusals declared before it. A valid ess/23 model with `removed-refused: state == Removed` before `not-active: state != Active` gains ESS-SYNTH-003. Its message says the scenario is missing, but the scenario exists and the honest target passes it.
  - **What reaches it:** the guide's new paragraph tells authors "Declaration order decides which refusal answers when both hold". The story's second adopter (refuse in Paused, Cancelled and Expired) is exactly this shape.
  - **Fix:** skip a state where an earlier-declared guarded branch's predicate, with only `state` bound, is True.
- **F2** at `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:5370`. Warning, CONFIRMED, introduced.
  - **Measured:** `not-active`'s main record (Removed) is created with description `description` and then sent `description` again. A target that refuses there but stores the sent value changes nothing the new complete comparison can see. Only the extra Suspended record catches a write, and only because its stored value happens to be `description-1`.
  - **What reaches it:** the issue's own complaint, a refusal that writes an unguarded field. It also passes in any state that only the main record covers.
  - **Fix:** for a complete refusal, send every input that an accepting branch's `sets:` writes with a value different from the one stored on the record.
- **F3:** the refusal text in F1 ("has no scenario") contradicts the design note at `docs/design/cross-record-and-stored-field-guards.md:458`, which says such a refusal is recorded "while it stands". It is folded into F1.
- **F4** at `crates/specify/ess-compiler/src/resolve.rs:5180`, also `ir.rs:889` and the design note at `:276`. Note, CONFIRMED, introduced.
  - **Measured:** these say "in either shape". But validation refuses a `{field, equals}` refusal that names no subject of its own (`crates/specify/ess-domain/src/command.rs:5876-5893`), so that half of the rule never applies to a valid model.
  - **Fix:** change the doc lines only.

**5. Attacked and could not break**
- A refusal that moves the record (state change) is caught by the complete snapshot.
- A write while refusing on the extra Suspended record is caught.
- `state != Active` beside `wrong_state:` gives no new refusal, and the wrong-state snapshot counts equal ess/22.
- The Go and TypeScript runners match the Rust verdicts for the honest target and 3 faults on the new scenarios.
- Byte identity below ess/23: the implementor's base vs final corpus (`corpus/base` and `corpus/final`: 183 models plus the ess/22 fixture, 670 files) differs only in log paths. Read-only `diff -rq`; I did not rebuild base.
- `compensates:` exclusion: checked by reading the code only (`outcome.subject.is_none()`); no case written.

**6. Paths written outside the worktree**
- `~/.cache/ess-054-461/adv/` holds `env.sh`, `build.log`, `case1.log`, `case2.log`, `case-desc.log`, `suite.log`, `final.log`, `fmt.log` and 7 per-case `adv461_*.log` files. One of those logs, `adv461_field_equals_refusal_beside_wrong_state.log`, is from the superseded first variant.
- `~/.cache/ess-054-461/adv/tmp/node-compile-cache/`
- `/dev/shm/ess-054/ess-054-461-adv`: removed with `cargo clean`.
- `~/.cache/b10x-go-cache/ess-054-461-adv`: removed.

**7.**
```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 5202
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: claimed_states ignores declaration-order precedence, so an earlier `state == Removed` refusal makes a valid ess/23 model gain ESS-SYNTH-003 for `not-active`, whose text says the existing scenario is missing
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 5370
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the refusal's main record is sent the description it already holds, so a target that refuses `not-active` on a Removed record and stores the sent description passes the complete comparison
- file: crates/specify/ess-compiler/src/resolve.rs
  line: 5180
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: resolve.rs, ir.rs:889 and the design note claim complete_refusal for `when_subject` refusals "in either shape", but validation refuses every subjectless `{field, equals}` refusal, so that half covers no valid model
```
