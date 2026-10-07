---
format: aep.planning-md/3
id: review-result:related-via-optional-input-20261004-r1
kind: review-result
status: active
title: 'Optional-input via adversary pass 1: absent witness missing beside input guards'
relations:
- reviews: story:related-via-optional-input
revision: 1
---
unit: 304-optional-input
verdict: red
cases: executed 57→67, red 2
origin: introduced 2, pre-existing 1, undecided 0
wrote-outside-worktree: review scratch (adv/ logs, tmp/) and the unit's build directory
needs-coordinator: yes — the reviewer took no worktree session lease

Publication copy of the first adversary pass on the #304 slice-1 unit (worktree `<worktrees>/ess/ess-optional-input-via-20261003`, HEAD `9ccbc0beb` plus the uncommitted unit diff). The reviewer added one file, `crates/verify/ess-conformance/tests/adversary_related_via_optional_pass1.rs`, and edited no production file.

## Cases

| Case | Asserts | Now |
|---|---|---|
| `adversary_304_input_guarded_refusal_beside_optional_via_synthesizes_and_agrees` (:252) | fixture plus input `hold: Boolean` and `{name: held, when: hold == true, error: Held}`: some scenario sends PublishRelease without `candidate`; honest interpreter passes; both fault targets fail | red |
| `adversary_304_the_unit_fixture_below_ess_22_is_refused_naming_ess_22` (:357) | the unit's own fixture (with `wrong-state`) at ess/21 is refused naming ess/22 | red |
| wrong_state first / no wrong_state / accepting related branch with refusing default / creating command with Optional via / its dependent PublishRelease | no extra refusal vs the required-input twin, an absent invocation exists, interpreter agrees with synthesis, both faults caught | green |
| explicit `candidate: null` / two Optional vias / absent coverage over `when:` branches | null is absent; refused as "one related row"; uncovered `hold=false` refused | green |

Red output: `input-guarded refusal beside: no scenario sends demo.release.PublishRelease with the Optional reference absent` (:193); `ess/21: the Optional via with its wrong_state sibling is refused without naming ess/22 … [conflicting_declaration] …` (:363). Before the absent-invocation assertion existed, `Fault::AbsentAsMissing` scored all 5 PublishRelease scenarios Passed (run3.log).

Suite: with the file, 8 passed / 2 failed in it and the unit's targets green (exit 101); without it, 57 passed (exit 0).

## Findings

A — `crates/verify/ess-conformance/src/synthesize/related_guard.rs:978`: with any `when:` input refusal beside an Optional via, synthesis emits no absent-reference invocation and records no refusal. Reached by any ess/22 command of the #304 reproduction's shape plus one input guard, through `synthesize()` (`synthesize.rs:2240`, `absent_selects`). Hypothesis (untested): `absent_input` builds candidates from the target branch's own guard only; other arrangement paths (`:658-663`) search with every input guard of the command.

B — `crates/specify/ess-domain/src/command/related_guard.rs:506`: below ess/22 the #282 `wrong_state` conflict `continue`s before the new format gate, so the unit's fixture at ess/21 is refused without naming ess/22. Base has the same gate at :474–482 (read, not executed). The unit's test `optional_release_at` (`ess-domain/tests/related_guard.rs:684`) drops the wrong-state branch.

C — `crates/specify/ess-compiler/tests/related_guard_ir.rs:101`: `issue_304_ess_20_related_fixtures_compile_to_identical_ir` pins no bytes. No golden ess/20 IR exists in the repository.

Attacked and not broken: #282 precedence in both orders with and without wrong_state; accepting related branch with refusing default; creating command; two Optional vias; null reference; `RelatedVia` + `PartialEq<str>` (no byte or message change). `when_subject` beside a via was not attacked (refused for every related guard).

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 978, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "with an input-guarded refusal beside the Optional via, synthesis emits no absent-reference invocation and records no refusal, so a target treating absent as a missing row passes every PublishRelease scenario"},
  {"file": "crates/specify/ess-domain/src/command/related_guard.rs", "line": 506, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "below ess/22 the wrong_state conflict continues before the Optional format gate, so the unit's own fixture at ess/21 is refused without naming ess/22"},
  {"file": "crates/specify/ess-compiler/tests/related_guard_ir.rs", "line": 101, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the test named compile_to_identical_ir pins no bytes; it asserts one field and one absent substring and would pass against most IR drift"}
]
```
