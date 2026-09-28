---
format: aep.planning-md/3
id: review-result:adversary-b-distinguish-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.41 unit distinguish
relations:
- reviews: story:sets-retarget-mutants-get-a-separating-witness
revision: 1
---
unit: distinguish (beyond10x/ess#202), working tree ess-b-distinguish on c6172c3b9b, uncommitted (correction 1)
verdict: CONFIRMED
cases: executed 1571→1579, red 3
origin: introduced 1 / pre-existing 0 / undecided 2
wrote-outside-worktree: 4 (~/.cache/ess-wave-n2/distinguish/adv2/{review.md,cases-alone.log,case1-alone.log,suite.log}) plus assigned build dir ~/.cache/b10x-target/ess-b-distinguish
needs-coordinator: yes. Finding 2 is outside the decided joined-source rule (the generator's retarget makes no join), so it needs a scope call: widen the rule to read the generator's choice, or record it as a follow-up.

## 1. Diff stat

```
 crates/verify/ess-conformance/src/synthesize.rs    | 202 ++++++++++++++++++---   (implementor's)
 .../ess-conformance/src/synthesize/subject_fact.rs |  10 +                     (implementor's)
 .../tests/connective_and_source_mutants.rs         | 174 ++++++++++++++++++     (implementor's)
?? crates/verify/ess-conformance/tests/adversary_distinguish_pass1.rs            (pass 1, test file)
?? crates/verify/ess-conformance/tests/adversary_distinguish_pass2.rs            (mine, test file)
```

No implementation file edited. After the runs I ran `rustfmt` on my file only; it changed whitespace, not behaviour.

## 2. Cases added (crates/verify/ess-conformance/tests/adversary_distinguish_pass2.rs)

| case | line | shape | now |
|---|---|---|---|
| `a_join_the_declared_model_makes_does_not_crowd_out_the_retargeted_pair` | 239 | the implementor's FLAGS plus `title` feeding both `name` and `display_name` | RED |
| `a_retarget_onto_an_input_only_the_payload_reads_is_killed` | 269 | FLAGS plus `notify: Boolean`, declared first and read only by the payload | RED |
| `the_unread_input_named_like_a_target_is_the_one_separated` | 293 | inputs `paid, notify, gift, rush`; `gift` mutant; pins the target-name preference | green |
| `a_declared_model_joining_nothing_carries_no_unseparated_note_even_when_guards_hold_inputs_equal` | 314 | guard `paid == true, notify == true`; pins the `targets.len() < 2` threshold | green |
| `retargets_are_killed_on_each_of_two_outcomes_of_one_command` | 346 | `express` and `placed` both write the three flags | green |
| `the_stored_row_path_names_a_pair_its_guards_hold_equal` | 375 | `updates` beside `when_subject`, guard holds `paid`/`gift` true | green |
| `every_unseparated_note_names_a_scenario_the_suite_holds` | 483 | ess/16, a `now` guard plus a fixed instant after the reference (refused by `now_offset::install`) | RED |
| `the_unseparated_notes_are_the_same_on_every_synthesis` | 578 | synthesize twice; compare notes and canonical JSON | green |

Red output, the new file run alone (`cargo test --locked -p ess-conformance --test adversary_distinguish_pass2 --no-fail-fast`):

```
---- a_retarget_onto_an_input_only_the_payload_reads_is_killed stdout ----
retargets survive: [
    "sets-retarget/shop.order.PlaceOrder/placed/gift: 1 rows [paid<-paid:Some(Bool(false))/Some(Bool(false)) gift<-gift:Some(Bool(false))/Some(Bool(false)) rush<-rush:Some(Bool(true))/Some(Bool(true))]",
    "sets-retarget/shop.order.PlaceOrder/placed/paid: 1 rows [paid<-paid:Some(Bool(false))/Some(Bool(false)) gift<-gift:Some(Bool(false))/Some(Bool(false)) rush<-rush:Some(Bool(true))/Some(Bool(true))]",
]

---- every_unseparated_note_names_a_scenario_the_suite_holds stdout ----
`sets-retarget/demo.jobs.ScheduleJob/scheduled/gift`: notes name scenarios the suite does not hold: [("demo.jobs.ScheduleJob/outcome/scheduled", "paid", "gift")]; held: []

test result: FAILED. 5 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

The first run of case 1 also listed the `name` and `display_name` mutants. That was a gap in my oracle: it compared only the flag fields. I added `name<-title` and `display_name<-title` to the oracle and ran case 1 alone again (`case1-alone.log`):

```
---- a_join_the_declared_model_makes_does_not_crowd_out_the_retargeted_pair stdout ----
retargets survive: [
    "sets-retarget/shop.order.PlaceOrder/placed/gift: 1 rows [name<-title:Some(Text(\"title\"))/Some(Text(\"title\")) display_name<-title:Some(Text(\"title\"))/Some(Text(\"title\")) paid<-paid:Some(Bool(true))/Some(Bool(true)) gift<-gift:Some(Bool(true))/Some(Bool(true)) rush<-rush:Some(Bool(false))/Some(Bool(false))]",
    "sets-retarget/shop.order.PlaceOrder/placed/paid: 1 rows [... same: paid true, gift true ...]",
]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.01s
```

## 3. Suite run, after the cases existed

`cargo test --locked -p ess-conformance --no-fail-fast` → passed 1576, failed 3, `EXIT=101`

```
test a_retarget_onto_an_input_only_the_payload_reads_is_killed ... FAILED
test a_join_the_declared_model_makes_does_not_crowd_out_the_retargeted_pair ... FAILED
test every_unseparated_note_names_a_scenario_the_suite_holds ... FAILED
```

`before` = 1571, the sum of passed counts in the implementor's `~/.cache/ess-wave-n2/distinguish/after2.log` (0 failed; the pass-1 file is included).

## 4. Findings

1. **A join in the declared model takes the one pin** (`synthesize.rs:5861`). `joined_pair` returns the first qualifying source in input order, and only one per outcome.
   - With `name: input.title, display_name: input.title` beside the unread identity `order_id` (String), the String pair is separated and pinned in every mutant.
   - The Boolean pair the `gift` and `paid` retargets join gets the unpinned loop, and both mutants survive.
   - No `UnseparatedSources` note is recorded for the Boolean pair either. That breaks "never dropped silently" (decisions #202 revision).
   - Control: the implementor's FLAGS test, which has the same Booleans and no title join, is green.
   - Reached by: any branch that writes one input to two fields beside a same-typed unread input. The identity counts as unread, so any String-identity entity copying a String input twice qualifies.
   - Fix, not applied: collect every joined pair in the outcome, separate and pin each, and note each one that stays equal.
   - NEEDS-CHANGE / undecided / warning.
2. **The retarget the generator makes can be one that joins nothing** (`synthesize.rs:5861`, generator `mutate.rs:653`). The generator retargets to the first same-typed input. When that input feeds no target (`notify`, read only by the payload and declared first), each mutant is `X: input.notify`. No input then feeds two targets, `joined_pair` is `None`, and the rule never fires. Two of three mutants survive, with no note.
   - Reached by: any command with a flag-typed input that is read only by a payload or a guard and is declared before the `sets:` inputs. This is a common shape.
   - It is outside the rule as decided, because the mutated model shows no join. A shape to consider: a source whose target is named like an input that no target reads.
   - NEEDS-CHANGE / undecided / warning.
3. **A note can name a scenario the suite does not hold** (`synthesize.rs:1427`, recorded before `now_offset::install` at :1456 and `fixtures::install` remove scenarios). The mutant's note names `demo.jobs.ScheduleJob/outcome/scheduled`, but the suite holds no scenario at all. This is the leak the implementor mentioned, and duplicates are not the only route to it.
   - Reached by: any joined pair on a command whose scenarios the now-offset or fixture pass refuses. That is constructed here, but the model compiles.
   - Fix, not applied: compute the notes after those passes, or drop notes whose scenario was removed.
   - CONFIRMED / introduced / note.

## 5. Attacked and not broken

- Suite format: `Note` derives no `Serialize`. The notes are printed only in `ess conform generate` text output (`ess-cli/src/main.rs:3921`), never in the suite JSON or `schemas/`, and have no Go or TypeScript reader. No format drift. The implementor's GEN_EXIT=0.
- Two outcomes of one command: each branch's mutants are killed.
- Stored-row path where no move keeps the branch: the note is recorded with the right scenario.
- The target-name preference and the two-target threshold: both are pinned by the green cases at :293 and :314. I traced by hand that dropping the preference picks `notify` and that `< 1` would note (paid, notify). Neither was run on a mutated copy.
- Determinism: notes and canonical JSON are identical across runs.
- Declared models with every input feeding one target: no note, and the cases do not change.
- Enums and the event-invoked command: pass 1 covered these, still green in this suite.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/distinguish/adv2/review.md
- ~/.cache/ess-wave-n2/distinguish/adv2/cases-alone.log
- ~/.cache/ess-wave-n2/distinguish/adv2/case1-alone.log
- ~/.cache/ess-wave-n2/distinguish/adv2/suite.log
- ~/.cache/b10x-target/ess-b-distinguish (assigned build dir)

## 7. Findings block

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5861, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "joined_pair takes only the first joined source per outcome, so a declared join (title into name and display_name beside the String identity) takes the pin and the gift and paid Boolean retargets survive with no UnseparatedSources note"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5861, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "when the first same-typed input is read only by the payload, the generator retargets every flag onto it, the mutated model joins nothing, the rule never fires and two of three mutants survive silently"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 1427, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "UnseparatedSources is recorded before now_offset and fixture installation remove scenarios, so a note can name a scenario the suite does not hold"}
]
```
