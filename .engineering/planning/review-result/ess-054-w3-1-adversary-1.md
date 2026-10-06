---
format: aep.planning-md/3
id: review-result:ess-054-w3-1-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W3-1 one record per element
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-459
revision: 1
---
unit: W3-1 (story:feature-request-459), commit range 09ec1dc33..c3a9357af in the ess-054-w3-1 worktree
verdict: NEEDS-CHANGE
cases: executed 69→77, red 7
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: none

## 1. Diff stat

`git --no-pager diff --stat` is empty because the only changes are two new untracked test files. `git status --short`:

```
?? crates/specify/ess-domain/tests/adversary_459_pass1.rs        (57 lines)
?? crates/verify/ess-conformance/tests/adversary_459_pass1.rs    (623 lines)
```

Both are test files. No implementation file was touched, and nothing is committed.

## 2. Cases added (all red now, each run alone first)

| Case | Asserts | Red output (first run) |
|---|---|---|
| ess-domain `distinct_behind_a_subject_guard_does_not_hold_the_member_distinct` | A refusal with `when_subject: {predicate: label == "locked"}` plus `when: {not: {distinct: …}}` does not hold the member distinct, so validate must return `missing_declaration` at `ran.affects[0].each`. | `panicked at …adversary_459_pass1.rs:42:18: admitted: a repeated document_id reaches ran wherever the source's label is not locked, and no distinct: holds it distinct there` |
| conformance `a_repeated_member_reaches_the_each_branch_when_the_refusal_is_subject_scoped` | Under that model the interpreter does not accept two elements naming `d1`. | `left: Some("demo.feed.RunSource/ran") … two elements naming d1 were accepted; d1 holds Some(Text("second"))` |
| conformance `each_entry_reading_a_boolean_member_is_witnessed` | An element with a `Boolean` member that the entry reads gives a synthesis with no refusals. | `NoWitness … path: "ran.affects[0]" … "offers no four elements that keep three identities apart, change every member the entry reads on a held row, and reach the branch"` |
| conformance `a_target_replacing_the_held_row_fails_the_each_scenario` | A target that overwrites the held row and drops `note` (a field the entry does not write) fails `ran`. | `a target replacing the held row passes ran: { … "demo.feed.RunSource/outcome/ran": Passed }` |
| conformance `a_target_writing_element_rows_on_a_refused_run_fails` | A target that writes the element rows and then refuses fails that refusal's scenario. | `WritesOnDuplicate: …/duplicated is Some(Passed)` and `WritesOnUnknownSource: …/no-such-source is Some(Passed)` |
| conformance `a_target_refusing_an_empty_list_fails` | A target that refuses `applied: []` for a held source fails some scenario. | `a target refusing an empty list passes every scenario: {added: Passed, duplicated: Passed, no-such-source: Passed, ran: Passed}` |
| conformance `an_element_naming_the_subject_leaves_the_subject_its_own_writes` | An `each:` entry over the subject's own entity, with an element naming the subject, is either refused or leaves the subject with its own `sets:`. | `left: Text("element") right: Text("own") the element overwrote the subject's own sets: Instance { state: Live, fields: {"label": Text("element")} }` |

`premise_the_fixture_validates` (ess-domain) is green. It is a guard that the unedited fixture validates.

## 3. Suite run (after the cases existed)

Command: `cargo test --locked --offline --no-fail-fast -p ess-domain --test set_effects --test adversary_459_pass1`, then the same with `-p ess-conformance --test set_effects --test interpreted_set_effects --test adversary_459_pass1`. Both used `CARGO_TARGET_DIR=/dev/shm/ess-054/W3-1` and the invariants environment.

```
ess-domain set_effects                 ok. 35 passed; 0 failed
ess-domain adversary_459_pass1         FAILED. 1 passed; 1 failed
DOMAIN_EXIT=101
ess-conformance set_effects            ok. 18 passed; 0 failed
ess-conformance interpreted_set_effects ok. 16 passed; 0 failed
ess-conformance adversary_459_pass1    FAILED. 0 passed; 6 failed
CONF_EXIT=101
```

- **Before (69):** the unit's own targets in this same run, with my two files left out of the count (35 + 18 + 16).
- **After (77):** 69 plus my 8 cases.
- **Separately:** ess-compiler `--test set_effects_ir` ran 4 passed, exit 0. It is not counted above.

## 4. Findings (cover commit c3a9357af)

| # | file:line | What breaks | Case | Verdict / origin |
|---|---|---|---|---|
| 1 | crates/specify/ess-domain/src/command/set_effects.rs:2260 | `input_guard` gives `refuses` the input guard of `SubjectPredicate`, `SubjectField` and `Related` refusals. Those refusals only fire when their subject or related condition also holds, so repeated identities reach the `each:` branch with a last-write-wins result. Reached by an author who scopes the duplicate refusal by subject. Fix: in the refusal path, count only `OutcomeCondition::When`. | both `…subject…` cases | NEEDS-CHANGE / introduced |
| 2 | crates/verify/ess-conformance/src/synthesize/set_effects.rs:1502 (with :1482) | `held_row` is built at distinction 11+4a and `again` at 13+4a, which always have the same parity. A `Boolean` is parity-based, so every attempt counts as "unchanged" and the branch is refused as `NoWitness`. Reached by any element with a Boolean member the entry reads. Probably also enums with an even number of variants, but I did not run that. Fix: build `again` at a distinction of the other parity. | `each_entry_reading_a_boolean_member_is_witnessed` | NEEDS-CHANGE / introduced |
| 3 | crates/verify/ess-conformance/src/interpret/execute/set_effects.rs:75 | The `each:` plan has no subject exclusion. Filter entries do have one (:93, and the design note at :167 says "the subject itself excepted"), and validate admits an `each:` entry over the subject's entity. So an element naming the subject overwrites the branch's own `sets:`. The design note leaves this undecided, and the suite never sends such an element, so targets can disagree. | `an_element_naming_the_subject_…` | NEEDS-CHANGE / introduced |
| 4 | docs/design/set-effects-over-filtered-instances.md:263 (`each_reads`, synthesize/set_effects.rs:1626) | The note says "A target that only creates … fails the scenario". But the held row is set up only by the same entry, in `initial`, so a target that overwrites the row (drops fields the entry doesn't write, resets the state) passes. Only the "skip held rows" kind of create-only target is caught. Reached by hand-written targets; the generated targets refuse `each:`. | `a_target_replacing_the_held_row_…` | CONFIRMED / introduced |
| 5 | docs/design/set-effects-over-filtered-instances.md:249 | "A refused run writes none" holds in the interpreter, but no synthesized refusal scenario reads the `SeenDocument` rows. So a target that writes the rows and then refuses passes both `duplicated` and `no-such-source`. I did not run whether this also applies to filter entries. | `a_target_writing_element_rows_…` | CONFIRMED / introduced |
| 6 | docs/design/set-effects-over-filtered-instances.md:224 | "An empty list … is an accepted answer" is checked only by the interpreter test. No scenario sends `[]` to an accepting branch, so a target that refuses an empty list passes everything. | `a_target_refusing_an_empty_list_fails` | CONFIRMED / introduced |
| 7 | crates/specify/ess-domain/src/command/set_effects.rs:498 and schemas/generated/ess.schema.json:1638 | Old formats: an ess/22 entry with no `where:` used to fail parsing with ``missing field `where` ``. It now gets `missing_declaration` plus an `empty_declaration` ("Invite declares no outcomes"). An ess/22 entry carrying `instance:` changed from ``unknown field `instance` `` to `conflicting_declaration` mentioning `each:`, with no format hint. The schema now drops `where` from `required` for every format. The `empty_declaration` cascade itself is pre-existing: base does the same for an unparsable `where:`. | none (scratch probe) | CONFIRMED / introduced |

## 5. Attacked and not broken

- **Old-format bytes:** IR (`to_canonical_json`) and suite JSON were byte-identical between base 09ec1dc33 and head. Each tree was built in its own target dir, for `set-effects.yaml` at ess/16 through ess/23 and `set-deletes.yaml`, which is 18 files.
- **Old-format refusals:** an ess/15 model with `affects:` gives byte-identical refusals.
- **Distinct guards that are not sufficient:** `distinct:` under `any:` in the accepting guard, `distinct:` over another member (the unit tests this), and `all:[not distinct, X]` in a refusal are all refused. The refusal declaration order doesn't matter, because input-guarded refusals answer first.
- **Unit's own mutants:** the update-only mutant, the skip-held create-only mutant and the adds-a-second-row mutant are all caught.
- **Absent Optional member:** it clears the field, the same as `sets:` from an absent Optional input. A required field filled from an Optional member is refused as `type_mismatch`.
- **Below ess/23:** an `each:` entry is refused alone, with no cascade.

## 6. Paths written outside the worktree

- `~/.cache/ess-054-wave/W3-1/adv1/`: logs, the probe models (`models/`, `models2/`), the probe outputs (`out-*`, `out2-*`), `zz_adv_bytes.rs` and `tmp/`. The base and head source copies were deleted.
- `/dev/shm/ess-054/W3-1/base-target` and `/dev/shm/ess-054/W3-1/head-target`: removed with `cargo clean`.
- `/dev/shm/ess-054/W3-1`: removed with `cargo clean`, as the adversary brief says. Rebuilding is needed to re-run the cases.

## 7. Findings block

```findings
- file: crates/specify/ess-domain/src/command/set_effects.rs
  line: 2260
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'held_distinct accepts a negated distinct inside a refusal scoped by when_subject/when_related, which refuses repeats only where that subject guard holds, so repeated identities reach the each: branch with an order-dependent row'
- file: crates/verify/ess-conformance/src/synthesize/set_effects.rs
  line: 1502
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'each_lists builds the held and second-call elements at distinctions of equal parity, so a Boolean member the entry reads never differs and every such branch is refused as NoWitness'
- file: crates/verify/ess-conformance/src/interpret/execute/set_effects.rs
  line: 75
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'an each: entry over the subject entity whose element names the subject overwrites the branch own sets:, unlike every filter entry which excepts the subject, and neither validate nor the design note decides it'
- file: docs/design/set-effects-over-filtered-instances.md
  line: 263
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'a create-only target that overwrites the held row, losing fields the entry does not write, passes the each: scenario because the held row is only ever arranged by the same entry in initial'
- file: docs/design/set-effects-over-filtered-instances.md
  line: 249
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'a target writing the element rows and then refusing passes both the duplicated and no-such-source scenarios, so the suite does not hold targets to a refused run writes none'
- file: docs/design/set-effects-over-filtered-instances.md
  line: 224
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'no scenario sends an empty list to the accepting branch, so a target refusing an empty list passes the whole suite'
- file: crates/specify/ess-domain/src/command/set_effects.rs
  line: 498
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'an ess/22 affects entry without where: or with instance: changed from a parse refusal to missing_declaration or conflicting_declaration plus an empty_declaration cascade, and the generated schema no longer requires where for any format'
```
