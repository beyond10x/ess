---
format: aep.planning-md/3
id: review-result:zi-w1-caller-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:caller-attributes-walk-every-caller-read-of-a-command
relations:
- reviews: story:caller-attributes-walk-every-caller-read-of-a-command
revision: 2
---
unit: story:caller-attributes-walk-every-caller-read-of-a-command (tree zi-t1-4, HEAD aa2b372bb4 plus one uncommitted test file)
verdict: weak
new failing cases: 1 (crates/generate/ess-gen/tests/adversary_caller_attributes.rs::a_caller_read_through_a_struct_attribute_is_listed)
mutants survived: 1 (if the "root named `caller`" skip in `caller_attributes` (http.rs:214-219) is deleted, the unit's own suite stays green: neither of its models has a field named `caller`. I worked this out from the models and did not run the mutant.)

The only change in the tree is one new untracked test file, `?? crates/generate/ess-gen/tests/adversary_caller_attributes.rs`. `git diff --stat` is empty and no implementation file was touched.

- **Red, outside the acceptance (backlog), pre-existing.** An actor attribute can be a struct, and a branch can read it through a member: `when: caller.org.id != input.org_id`. Validation admits this, and so does the domain check `is_caller_path`, which counts any path with more than one segment. The branch answers `403`, but `caller_attributes` returns `[]` because http.rs:222 keeps only paths with exactly 2 segments. Command: `build.sh adv/red.txt … test -p ess-gen --locked --test adversary_caller_attributes`. Failure at adversary_caller_attributes.rs:80: `left: [] right: ["org"]`. That `403` assertion just before it passed. The same `== 2` filter is at the base, line 199, and I confirmed that by reading the base file, not by running the case there. The same filter also sits in ess-conformance `synthesize/caller.rs:1122`. Fix: use `is_caller_path`'s rule (more than one segment, take `segments()[1]`).
- **Green, kept because they pin things the suite did not:**
  - `a_field_named_caller_is_no_caller_read` pins the skip from the mutant line above.
  - `the_input_guard_of_an_external_subject_branch_is_listed` pins acceptance position 1, the input guard of `external:` + `when_subject:`. The unit's `Veto` model reads the caller only in the predicate.
- **Suite run, after the cases existed.** `build.sh adv/suite2.txt … test -p ess-gen --locked --no-fail-fast --test caller_values --test adversary_caller_attributes` exited 101. `caller_values` went 5 of 5 before, 5 of 5 after. The new file ran 3 cases, 1 red. The first suite run (`adv/suite.txt`) stopped at the failing target without `--no-fail-fast`, so `caller_values` did not run there.
- **Byte pin.** `build.sh adv/pin.txt … test -p ess-openapi --locked --test adversary_ess_issues_2_u6` passed 7 of 7, exit 0. None of its fixtures and no model under `examples/` reads the caller, so leaving every pin unchanged is correct.
- **Gate step that tested nothing (judgement, introduced).** The unit's own `lib.txt` shows `cargo test -p ess-compiler --lib expression` ran 0 tests (30 filtered out). The new `command_predicate_sites` (expression.rs:355) therefore has no direct test. It is covered only through `caller_values`.
- **Attacked and could not break:**
  - Error payload, `when_related` input guards and `external:` + `when_subject:`: all listed.
  - A `caller` root in input or stored fields: validation and generation agree.
  - Caller reads inside `RelatedSelection`/`RelatedHead` `where:`, `Expression`, and `instances:`/`affects:`: validation refuses all of them. `with_caller` is enabled only in `caller_value.rs`, and `set_effects.rs:1838` refuses `{caller:}`. So treating those values as leaves loses nothing.
  - `command_sites` and `caller_value::validate` use the same roots for every guard.

Paths written outside the worktree: `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/caller-attributes-walk-every-caller-read-of-a-command/adv/{red.txt,suite.txt,suite2.txt,pin.txt}`. The builds used the shared slot targets A and B, which build.sh owns.

```findings
- file: crates/generate/ess-gen/src/http.rs
  line: 222
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: a caller read through a struct attribute member (caller.org.id) is admitted by validation and decides a 403 branch, yet caller_attributes drops it because it keeps only two-segment paths, so x-ess-caller omits org
- file: crates/generate/ess-gen/src/http.rs
  line: 214
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: deleting the skip for a root named caller leaves the unit's suite green; now pinned by a_field_named_caller_is_no_caller_read
- file: crates/specify/ess-compiler/src/expression.rs
  line: 355
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the gate step for the compiler's predicate-site walk (cargo test -p ess-compiler --lib expression) ran 0 tests, so command_predicate_sites has no direct test
```
