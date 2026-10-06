---
format: aep.planning-md/3
id: review-result:ess-054-w1-3-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W1-3 read-API idioms
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-442
- reviews: story:feature-request-446
revision: 1
---
unit: W1-3 read-api-idioms, worktree `~/.local/state/worktree/trees/b10x/ess/ess-054-w1-3`, findings cover head ff2bb99d1a plus my uncommitted test additions
verdict: CONFIRMED (2 red cases)
cases: executed 24→33, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (scratch dir); the build dir was created, then cleaned
needs-coordinator: none

**1. Diff of the worktree (`git --no-pager diff --stat`)**
```
 .../ess-conformance/tests/read_api_view_idioms.rs  | 203 +++++++++++++++++++++
 1 file changed, 203 insertions(+)
```
I changed only this test file and only appended to it: no existing line was removed or edited. The additions are uncommitted. I ran `rustfmt` on this one file only, and `cargo fmt -p ess-conformance --check` then exited 0.

**2. Cases added** (all in `crates/verify/ess-conformance/tests/read_api_view_idioms.rs`; each was run alone with `--exact` before the suite ran)

| Case (line) | Asserts | Now |
|---|---|---|
| `adversary_correlated_later_begin_of_the_key_is_the_held_row` (1053) | Note line 65 says the begin is "held as one row per correlation key … so a later begin replaces the earlier one". The case runs two begins for the same (customer, session) with `span_key` begin-1 and begin-2, then an End without a duration. It expects `derived` with `began_at` taken from the later begin. | **red** |
| `adversary_latest_event_suite_decides_the_tie_rule` (1010) | Note line 150 says "The identity is the tie rule … Synthesis … asserts the order". The case runs the idioms.latest suite against a model ordered `[at desc, event_id desc]` and expects it to fail. | **red** |
| `adversary_correlated_two_spans_of_one_key_refuse_the_end_today` (1125) | Shows why the first red case fails: the same timeline passes with End `ambiguous` and 2 rows in `OpenSpans`. | green |
| `adversary_note_names_every_synthesis_refusal_of_the_example` (944) | The example's synthesis refusals are exactly the 3 views the note names, all `ESS-SYNTH-017`. | green |
| `adversary_example_suite_passes_interpreted` (973) | The example's whole synthesized suite passes on the interpreter. | green |
| `adversary_latest_event_suite_decides_the_declared_order` (991) | A model with `at asc` fails the latest suite. | green |
| `adversary_computed_field_refusals_carry_the_codes_the_note_names` (1077) | The computed-field refusal carries both `ESS-VIEW-001` and `ESS-VIEW-005`. | green |
| `adversary_unit_field_summary_reaches_the_schema_description` (1091) | A field's `summary:` appears as the JSON Schema `description`. | green |
| `adversary_unit_max_keeps_the_newtype` (1106) | `max` declared as `Optional<Millis>` validates. | green |

Red output from the solo runs, verbatim:
```
thread 'adversary_latest_event_suite_decides_the_tie_rule' panicked at crates/verify/ess-conformance/tests/read_api_view_idioms.rs:1018:5:
assertion `left != right` failed: a target breaking ties by descending identity fails
  left: 0
 right: 0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 31 filtered out
```
```
thread 'adversary_correlated_later_begin_of_the_key_is_the_held_row' panicked at crates/verify/ess-conformance/tests/read_api_view_idioms.rs:1065:5:
assertion `left == right` failed: one held row per correlation key: {
    "idioms.correlated/authored/later-begin-of-the-key-is-held": Failed,
}
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 31 filtered out
```

**3. Suite run** (after the cases existed; rerun after fmt with the same result)
`cargo test -p ess-conformance --test read_api_view_idioms --locked --offline`, with the unit's `env.sh` and TMPDIR set to the scratch dir.
```
test adversary_correlated_later_begin_of_the_key_is_the_held_row ... FAILED
test adversary_latest_event_suite_decides_the_tie_rule ... FAILED
test result: FAILED. 31 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
EXIT=101
```
The 24 original cases all still pass.

**4. Findings**

| file:line | What breaks | Case | Verdict | Origin |
|---|---|---|---|---|
| `docs/design/read-api-view-idioms.md:65` and the fixture's line 26 | The fixture's held row is keyed by a caller-supplied `span_key`, not by the correlation fields (customer, session). So two begins for one correlation key are two rows, and the End is refused `ambiguous` instead of reading the later begin. The note never says the caller must build `span_key` from the correlation fields. **What reaches it:** an adopter following the note whose begins carry their own id. The issue's source rows have no span key at all. **Fix (named, not applied):** state in the note that the identity is the correlation key, composed by the caller, or switch the fixture to a struct identity of (customer, session). | `adversary_correlated_later_begin_of_the_key_is_the_held_row` | NEEDS-CHANGE | introduced |
| `docs/design/read-api-view-idioms.md:150` | The synthesized idioms.latest suite does not test the declared tie-break: a target ordering by `event_id desc` passes every scenario, so the claim that the identity is the tie rule goes unchecked. The synthesizer's lack of tying rows is existing behaviour; the claim is new in this unit. **What reaches it:** any target that breaks ties wrongly. **Fix:** a note line saying ties are not arranged, or an authored scenario with two equal `at` values. | `adversary_latest_event_suite_decides_the_tie_rule` | CONFIRMED | introduced |

**5. Attacked, did not break**
- The "three refusals" claim in the note's intro is exact.
- The example's full synthesized suite passes on the interpreter.
- The latest suite catches a reversed `at` order.
- `ESS-VIEW-001` and `ESS-VIEW-005` both hold.
- A field's `summary:` reaches the schema `description`.
- `max` keeps the newtype.
- The guide link format matches the 7 other guide pages that link `github.com/.../blob/main/docs/design`.
- The CLI command syntax quoted in the note matches other docs. Not executed: building ess-cli was skipped to save disk.

**6. Paths written outside the worktree**
- `~/.cache/ess-054-wave/W1-3/adv1/`: build.log, suite.log, suite2.log, companion.log, fmt.log, one `<case>.log` per case, and `tmp/`.
- `/dev/shm/ess-054/W1-3`: the build dir, removed with `cargo clean`.

I did not take a worktree session lease.

**7. Findings block**
```findings
- file: docs/design/read-api-view-idioms.md
  line: 65
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'the held row is keyed by a caller-supplied span_key, not the correlation fields, so two begins of one (customer, session) are refused ambiguous instead of the later one replacing the earlier as the note states'
- file: docs/design/read-api-view-idioms.md
  line: 150
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'the note calls the identity the tie rule, but the synthesized idioms.latest suite passes a target ordering ties by descending event_id, so the tie rule is unchecked'
```
