---
format: aep.planning-md/3
id: review-result:adversary-b-distinguish-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.41 unit distinguish
relations:
- reviews: story:sets-retarget-mutants-get-a-separating-witness
revision: 1
---
unit: distinguish (beyond10x/ess#202), working tree ess-b-distinguish on c6172c3b9b, uncommitted
verdict: CONFIRMED
cases: executed 1564→1570, red 3
origin: introduced 0 / pre-existing 2 / undecided 1
wrote-outside-worktree: 5 (~/.cache/ess-wave-n2/distinguish/adv1/{review.md,cases-alone.log,case3-alone.log,debug.log,suite.log}) plus build dir ~/.cache/b10x-target/ess-b-distinguish (assigned)
needs-coordinator: yes. Option (a) only helps when a sets target and an input share a name. Decide whether finding 1 (fields named differently from their inputs) is in scope for #202.

## 1. Diff stat

```
 crates/verify/ess-conformance/src/synthesize.rs    | 118 ++++++++++++++++----    (implementor's, not touched by me)
 .../tests/connective_and_source_mutants.rs         | 123 +++++++++++++++++++++  (implementor's)
?? crates/verify/ess-conformance/tests/adversary_distinguish_pass1.rs           (mine, untracked, test file)
```

I did not edit any implementation file.

## 2. Cases added (crates/verify/ess-conformance/tests/adversary_distinguish_pass1.rs)

The oracle: after a scenario sends the command with literal input, take every view row it then requires. The mutant is killed if one of those rows differs from what the **original** spec writes (`target := input[original source]`). Every case also asserts that at least one row was observed.

| case | shape | now |
|---|---|---|
| `retargets_are_killed_when_field_names_differ_from_input_names` :212 | `active: input.is_active`, `can_cancel: input.cancellable`, `can_extend: input.extendable` | RED |
| `retargets_of_a_declared_rotation_are_killed` :279 | `a: input.b`, `b: input.c`, `c: input.a` | RED |
| `retargets_are_killed_on_a_branch_the_stored_row_search_arranges` :365 | `updates` branch next to a `when_subject: locked == true` refusal, three Booleans | RED |
| `retargets_are_killed_on_a_command_an_event_invokes` :463 | the command a binding invokes from an event carrying three Booleans | green |
| `retargets_are_killed_among_three_inputs_of_a_two_variant_enum` :534 | three inputs of `enum [On, Off]` | green |
| `a_retarget_mutant_suite_is_the_same_on_every_synthesis` :547 | synthesize twice, compare the suites | green |

Red output, each case run alone:

```
---- retargets_are_killed_when_field_names_differ_from_input_names stdout ----
retargets survive: [
    "sets-retarget/shop.plan.OpenPlan/opened/active: 1 rows [active<-is_active:Some(Bool(true))/Some(Bool(true)) can_cancel<-cancellable:Some(Bool(true))/Some(Bool(true)) can_extend<-extendable:Some(Bool(false))/Some(Bool(false))]",
    "sets-retarget/shop.plan.OpenPlan/opened/can_cancel: 1 rows [active<-is_active:Some(Bool(true))/Some(Bool(true)) can_cancel<-cancellable:Some(Bool(true))/Some(Bool(true)) can_extend<-extendable:Some(Bool(false))/Some(Bool(false))]",
]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.01s

---- retargets_of_a_declared_rotation_are_killed stdout ----
retargets survive: [
    "sets-retarget/shop.dial.Rotate/rotated/a: 1 rows [a<-b:Some(Bool(false))/Some(Bool(false)) b<-c:Some(Bool(true))/Some(Bool(true)) c<-a:Some(Bool(false))/Some(Bool(false))]",
]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.01s

---- retargets_are_killed_on_a_branch_the_stored_row_search_arranges stdout ----
retargets survive: [
    "sets-retarget/shop.order.SetFlags/flagged/gift: 3 rows [... | paid<-paid:Some(Bool(true))/Some(Bool(true)) gift<-gift:Some(Bool(true))/Some(Bool(true)) rush<-rush:Some(Bool(true))/Some(Bool(true)) | ...]",
    "sets-retarget/shop.order.SetFlags/flagged/paid: 3 rows [... same: every input true ...]",
    "sets-retarget/shop.order.SetFlags/flagged/rush: 3 rows [... same: every input true ...]",
]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.02s
```

The stored-row case first ran with 0 rows, because the fixture's view did not project `state` (ESS-SYNTH-001). I fixed the fixture and added the non-empty assertion. The quote above is the re-run after that fix. The full logs are `cases-alone.log` and `case3-alone.log`.

## 3. Suite run, after the cases existed

`cargo test --locked -p ess-conformance --no-fail-fast` → passed 1567, failed 3, `EXIT=101`

```
test retargets_are_killed_when_field_names_differ_from_input_names ... FAILED
test retargets_of_a_declared_rotation_are_killed ... FAILED
test retargets_are_killed_on_a_branch_the_stored_row_search_arranges ... FAILED
test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

`before` = 1564. That is the sum of the passed counts in the implementor's `~/.cache/ess-wave-n2/distinguish/after.log`.

## 4. Findings

1. **Names that do not line up** (`synthesize.rs:5786-5809`). The mutated model `active: input.cancellable` has no target that names an input, so nothing is pinned, and the `active` and `can_cancel` mutants survive. With no pins, `moved_apart` is exactly the base loop, so this shape behaves as it did at base.
   - Reached by: any spec whose field names differ from its input names (`is_active` → `active`). This may be the adopter's case; I did not see their spec.
   - Candidate fix, not applied: in the mutated model, one input feeds two `sets` targets and a same-typed input feeds none. Separate the doubly-read source from the unread sibling first.
   - CONFIRMED / pre-existing / warning.
2. **Pins from the unmutated part crowd out the mutated pair** (`synthesize.rs:5786`). Take the rotation spec. The generator's mutant is `a: input.a`, which makes no pin. The untouched entries still pin (c,b) and (a,c). With Booleans that forces a == b, the one pair the mutant joined, so it survives silently.
   - This also contradicts the claim "unmutated specs make no pins". Any cross-named entry, such as `owner: input.new_owner` beside an input `owner`, pins.
   - Reached by: nothing I found in the repository's specs; I constructed it.
   - The base run was not available, so the origin is undecided. This pinning code is new, so it is likely introduced.
   - INFEASIBLE / undecided / warning.
3. **The stored-row path never separates siblings** (`synthesize.rs:2107` → `subject_fact.rs:1491`). A branch that `subject_fact::routes` returns early and never calls `freshened` or `distinguished`. Every Boolean is sent `true`, and all three mutants survive. The two-input #161 shape has the same gap.
   - Reached by: any `updates` or `moves` branch on a command with a `when_subject` stored-field guard. That is a common shape, and it is a candidate for the adopter's "event writing three booleans".
   - The diff does not touch this path.
   - CONFIRMED / pre-existing / warning.
4. **A pair that cannot be separated fails silently.** When `moved_apart` returns `None` for a target-named pair, no refusal or note is produced, so the mutant survives with no diagnostic. This is not a separate failing case; the red output in findings 1 and 2 shows it.
   - `synthesize.rs:5796`, CONFIRMED / undecided / note.

## 5. Attacked and not broken

- The command an event invokes, when field and input names line up: killed. Binding scenarios assert invocations, not rows, and the invoked command's outcome scenario takes the fixed path.
- Two-variant enum with three inputs: killed.
- Determinism: same suite on repeated synthesis.
- Removing the pin check in `moved_apart`: by hand-trace, the implementor's case would go red, because the `rush` step would move `gift` back to `paid`. Not run on a mutated copy.
- Literal-fallback second run: it is built from `freshened` output, so pins hold for the fields it sends. Read, not run.
- Two retargets competing for one input: the generator emits one retarget per mutant, so they compete only through the unmutated entries (finding 2).

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/distinguish/adv1/review.md
- ~/.cache/ess-wave-n2/distinguish/adv1/cases-alone.log
- ~/.cache/ess-wave-n2/distinguish/adv1/case3-alone.log
- ~/.cache/ess-wave-n2/distinguish/adv1/debug.log
- ~/.cache/ess-wave-n2/distinguish/adv1/suite.log
- ~/.cache/b10x-target/ess-b-distinguish (the assigned build dir)

## 7. Findings block

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5786, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "with no sets target naming an input (active: input.is_active and siblings) nothing is pinned and two of three sets-retarget mutants survive with the joined pair equal"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5786, "category": "property", "severity": "warning", "verdict": "INFEASIBLE", "origin": "undecided", "message": "pins taken from unmutated cross-named sets entries (a declared rotation) force the mutant's joined pair equal on Booleans, so the a-retarget survives; the claim that unmutated specs make no pins does not hold"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 2107, "category": "boundary", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "branches routed to subject_fact::prepare never pass through distinguished, so an updates branch beside a when_subject guard sends every Boolean true and all sets-retarget mutants survive"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5796, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "undecided", "message": "a target-named pair that cannot be moved apart is dropped silently, with no refusal or note, so the surviving mutant carries no diagnostic"}
]
```
