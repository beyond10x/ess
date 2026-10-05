---
format: aep.planning-md/3
id: review-result:ess-054-w1-1-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W1-1 ess/23 identity and state
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-429
- reviews: story:feature-request-458
revision: 1
---
unit: W1-1 (story:feature-request-429, story:feature-request-458), range 9ffa94d66a..d1b6c00bc in ~/.local/state/worktree/trees/b10x/ess/ess-054-w1-1, plus 2 uncommitted test files
verdict: NEEDS-CHANGE
cases: executed 16→28, red 3
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (part 6)
needs-coordinator: whether ess/22 suites may gain scenarios (finding 1); the wave invariant says no

I found three problems that need a change and one minor point about the generated Rust:
1. The suites of some older ess/22 documents change.
2. `{subject: state}` is never checked on a refusal selected by the record's lifecycle state (a `when_subject_state:` refusal).
3. The design note says a struct identity can be renamed, but validation refuses it.

## 1. Diff
`git diff --stat` shows nothing because both files are new and untracked. `git status --short`:
```
?? crates/verify/ess-conformance/tests/adversary_w1_1_pass1.rs                       (450 lines)
?? crates/verify/ess-conformance/tests/fixtures/adversary-w1-1-identity-selector-ess22.yaml (73 lines)
```
Both are test files. No implementation file was touched.

## 2. Cases added (all in `adversary_w1_1_pass1.rs`)
Each red output below is from a run of that case alone, before the suite ran.

| case | asserts | now |
|---|---|---|
| `adv_an_ess22_identity_selector_keeps_its_suite_bytes` | an ess/22 document whose `when_related` reads the identity keeps its suite bytes from the base build | **red** |
| `adv_subject_state_on_a_held_state_refusal_is_compared` | the `<entity>/state/Archived/refuses/TouchDoc` scenario requires `current == Archived` | **red** |
| `adv_rename_of_a_struct_identity_is_admitted_as_the_design_note_says` | the struct-identity rename validates, as the design note says | **red** |
| `adv_an_ess22_identity_selector_suite_passes_the_reference` | the interpreter passes the scenarios that ess/22 suite gained | green |
| `adv_rename_beside_unprojected_and_parameterised_views_passes_the_reference` | a rename beside a view without the identity and a view filtered by `owner` | green |
| `adv_rename_from_an_instance_input_named_otherwise_passes_the_reference` | `instance: current` | green |
| `adv_rename_with_the_collision_answer_declared_last_passes_the_reference` | `taken` declared after `renamed` | green |
| `adv_rename_back_to_a_previous_name_restores_the_row` | alpha→gamma→alpha in the interpreter | green |
| `adv_subject_state_on_an_update_passes_the_reference`, `..._on_an_update_event_is_compared` | `{subject: state}` on an `updates:`, in `sets:` and in an event | green |
| `adv_subject_state_in_sets_alone_is_refused_below_ess23`, `..._in_an_event_payload_alone_...` | each refused alone with `unsupported_format_version` naming ess/23 | green |

Red output, verbatim:
```
adv_an_ess22_identity_selector_keeps_its_suite_bytes ... panicked at adversary_w1_1_pass1.rs:389:5:
assertion `left == right` failed: the ess/22 suite moved:
---- refusals
  left: "9bc0f42981e15699aa21788c29cb96d0fe4e7f9d4a624f706501a2ea76acd875"
 right: "c9cc15173d057fbe168d1a35738c4cb58f133d5888d7ef7c7300c8db64fa256c"

adv_subject_state_on_a_held_state_refusal_is_compared ... panicked at adversary_w1_1_pass1.rs:296:17:
assertion `left == right` failed: demo.docs.Doc/state/Archived/refuses/demo.docs.TouchDoc: the frozen refusal answers the held state
  left: None
 right: Some(Text("Archived"))

adv_rename_of_a_struct_identity_is_admitted_as_the_design_note_says ... panicked at adversary_w1_1_pass1.rs:118:5:
the design note admits a struct identity written whole: [ ValidationError { code: TypeMismatch,
  location: "command.demo.vault.RenameSecret.outcomes.taken.when_related",
  message: "`name == input.new_name`: operator `==` does not admit `demo.vault.SecretName` (aggregate) and `demo.vault.SecretName` (aggregate)" ...
```

## 3. Suite run (after the cases existed)
`cargo test -p ess-conformance --locked --offline --no-fail-fast --test identity_changing_updates --test subject_state_source --test adversary_w1_1_pass1`
```
adversary_w1_1_pass1:      FAILED. 9 passed; 3 failed
identity_changing_updates: ok. 9 passed; 0 failed
subject_state_source:      ok. 7 passed; 0 failed
EXIT=101
```
"Before" is 16: the unit's two target files in this same run, with my file left out of the count.

## 4. Findings (tree: d1b6c00bc plus the two files above)

| # | file:line | what breaks | case | verdict / origin |
|---|---|---|---|---|
| 1 | `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:488`, `synthesize/row_set.rs:454` (also the `resolved_literals`/`closed` and `arrange_toward_bound` steering changes) | **Measured:** I built the base separately in its own target dir and ran `adversary_w3_266_267_dump.rs` there, then again at head. Three ess/22 probe documents with no ess/23 construct moved. At base they refused `NameSuccessor/named` and `successor-is-a-team` ("its row set is Unknown"); at head they synthesize those scenarios. Suite version stays `ess-conformance/34`. The interpreter passes the new scenarios. This contradicts `docs/design/identity-changing-updates.md:105` ("so no other model's suite moves") and the wave rule that old formats keep their bytes. **What reaches it:** any ess/≤22 document with a `when_related` selector that reads the identity (`where: <id> == input.x`), or that compares against an input naming an arranged instance. None of the 312 repository models has one, which is why the implementor's digests were unchanged. **Fix:** apply the identity binding and steering only from ess/23, or accept the change and correct the note. | `adv_an_ess22_identity_selector_keeps_its_suite_bytes` | NEEDS-CHANGE / introduced |
| 2 | `crates/verify/ess-conformance/src/synthesize.rs:3842` (`state_refusals`) | **Measured:** `state_refusals` passes `&run.before_settled` to `expect_error` instead of `with_held_state(…, Some(state))`. So `current: {subject: state}` on a `when_subject_state:` refusal is never compared, and a target can answer any state there. This misses the story's decision ("compares it with the state each refusal scenario arranges"). It also contradicts the guide (`commands-and-outcomes.md`: "each `<entity>/state/<S>/refuses/<command>` scenario requires `S` there"). **What reaches it:** any ess/23 document with that form; validation admits it. **Fix:** use `with_held_state` in `state_refusals`, as `refused_here` already does. | `adv_subject_state_on_a_held_state_refusal_is_compared` | NEEDS-CHANGE / introduced |
| 3 | `docs/design/identity-changing-updates.md:36` | **Measured:** the note says "A struct identity is written whole". But `identity_write::collision_answered` only accepts `<identity> == input.<field>` as the collision answer, and `==` refuses aggregates (`type_mismatch`). So a struct identity can never get the collision answer validation requires, and its rename can never validate. **What reaches it:** any entity whose identity type is a struct. **Fix:** either accept another collision shape for aggregates, or correct the note. | `adv_rename_of_a_struct_identity_is_admitted_as_the_design_note_says` | CONFIRMED / introduced |
| 4 | `crates/generate/ess-synth/src/rust/behaviour.rs:1913` | **Judgement only, no failing case.** The generated rename runs `delete(old)` and then `put(next)`. The storage port's `put` returns `()`, so a port that fails can only panic, and a panic after the delete loses the record. Because the collision refusal guarantees the new identity differs from the old one, writing `put` before `delete` would cost nothing and lose nothing. **What reaches it:** only a user port whose `put` panics. The generated in-memory store does not. | none | CONFIRMED / introduced |

## 5. Attacked and could not break
- Rename beside a view without the identity, a view filtered by another field, an `instance:` input with another name, the collision answer declared last, and a rename back to an earlier name: the interpreter passes the synthesized suite each time.
- Wrong targets (ignores the write, keeps the old row, drops a carried field, renames over a collision, renames onto itself): the unit's own Vault test already fails each one in the Rust and Go runners.
- `{subject: state}` below ess/23: refused in `sets:` alone and in an event payload alone, at the right location, naming ess/23.
- `{subject: state}` on an `updates:` in `sets:` and in an event payload: compared, and the interpreter passes. In the Rust and Go generators, `held_state` is declared in every branch that reads it (read only, nothing built).
- Old IR is never read back from disk, so the derived `identity_write` cannot reinterpret an older compiled IR.
- Not tested:
  - Stored-field (`when_subject:`) refusals: synthesis could not arrange my probe model at all. That is a separate, older arrangement limit, so I removed that case.
  - `{subject: …}` on `deletes:`: refused, as it was at base (the message wrongly says "`creates:` outcome"; that is not this unit's).

## 6. Paths written outside the worktree
- `~/.cache/ess-054-wave/W1-1/adv1/` (99M): the base export (`base/`), probe models (`models/`), base and head dumps (`dump-base/`, `dump-head/`), logs (`logs/`), `run-dump.zsh`, `run-test.zsh`, `tmp/`
- `/dev/shm/ess-054/W1-1-adv1-base` (base build dir): removed with `cargo clean`
- `/dev/shm/ess-054/W1-1` (unit build dir, reused): cleared with `cargo clean`, as the brief says
- `~/.cache/b10x-go-cache/W1-1` (GOCACHE, set by my run scripts)

## 7. Findings block
```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 488
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'Binding and steering on the identity apply at every format, so an ess/22 document whose when_related selector reads the identity now synthesizes scenarios its base suite refused (digest c9cc15… at base, 9bc0f4… at head), against the old-format byte invariant and design note line 105.'
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 3842
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'state_refusals builds expect_error from run.before_settled rather than with_held_state, so current: {subject: state} on a when_subject_state refusal is never compared and any answered state passes.'
- file: docs/design/identity-changing-updates.md
  line: 36
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'The note admits renaming a struct identity written whole, but the only accepted collision answer is identity == input.field, which == refuses for aggregates, so no struct-identity rename can validate.'
- file: crates/generate/ess-synth/src/rust/behaviour.rs
  line: 1913
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'The generated rename deletes the old row before put, so a storage port whose infallible put panics loses the record; put-then-delete costs nothing because the collision refusal guarantees the identities differ.'
```
