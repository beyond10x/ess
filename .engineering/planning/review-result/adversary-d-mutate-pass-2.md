---
format: aep.planning-md/3
id: review-result:adversary-d-mutate-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.42 unit mutate
relations:
- reviews: story:unkillable-mutants-are-not-scored-survived
revision: 1
---
unit: mutate-dead-guard (beyond10x/ess#218) pass 2, worktree ess-d-mutate on 4e7c3867e (uncommitted correction-1 diff + /3 registration patch + adversary test files)
verdict: CONFIRMED
cases: executed 1631→1632, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths (part 6)
needs-coordinator: none

## 1. git --no-pager diff --stat

17 tracked files changed, 741 insertions and 98 deletions. This is the implementor's diff; I touched no tracked file. Untracked:
```
?? crates/verify/ess-conformance/tests/adversary_mutate_dead_guard_pass1.rs   (pass 1)
?? crates/verify/ess-conformance/tests/adversary_mutate_dead_guard_pass2.rs   <- mine, the only path I added
?? crates/verify/ess-conformance/tests/fixtures/mutation-unwitnessed-outcome.yaml   (implementor)
?? crates/verify/ess-conformance/tests/mutation_unkillable.rs                 (implementor)
```

## 2. Case added: tests/adversary_mutate_dead_guard_pass2.rs (red)

`a_mutant_on_a_transition_only_a_set_outcome_performs_is_not_survived`: in the spec, `escalate` is performed only by `EscalateAll/escalated-all`, which is an `instances:` set outcome (ess/16). The baseline refuses that outcome's scenario with `ESS-SYNTH-001 desk.case.EscalateAll/outcome/escalated-all`. `flag` keeps `Escalated` reachable, so `transition-to/…escalate` compiles. The case runs the built-in interpreter target and asserts the mutant is not `survived`.

Red run of the case alone (`cargo test -p ess-conformance --test adversary_mutate_dead_guard_pass2 --no-fail-fast`, EXIT=101), verbatim:
```
transition-to/desk.case.Case.escalate Survived added=None baseline=None
assertion `left != right` failed: only the set outcome `escalated-all` performs `escalate`, and the baseline refuses its scenario (ESS-SYNTH-001); no scenario of either suite can kill this mutant, so it is not a survivor: MutantEntry { added_refusals: None, baseline_refusals: None, change: "`to: Escalated` becomes `to: Closed`", class: TransitionTo, ...
  left: Survived
 right: Survived
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```
Clippy `-D warnings` on the file exited 0, and rustfmt --check exited 0.

## 3. Suite run (after the case existed)

`cargo test -p ess-conformance --no-fail-fast`. Final lines, verbatim:
```
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

error: 1 target failed:
    `-p ess-conformance --test adversary_mutate_dead_guard_pass2`
EXIT=101
```
Totals: 1631 passed, 1 failed (the failure is mine), 3 ignored. `before` is 1631, the same run with my file deselected. All 5 pass-1 cases are green now.

## 4. Findings

| file:line | severity | verdict | origin | finding | what reaches it | fix |
|---|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/mutate.rs:1843 | warning | CONFIRMED | introduced | `performers` reads only `outcome.subject`. An `instances:` set outcome has `subject: None` and carries its `moves` in `outcome.instances` (ir.rs:920). So a transition performed only by set outcomes has no performer. `get(transition)?` returns `None`, and a `transition-to` or `from-drop` mutant on it is `survived` (exit 1). The baseline refuses every set-outcome scenario with `ESS-SYNTH-001 …instances`, so nothing can kill that mutant. This is the pass-1 class (silent drop), reached through the second way an outcome moves an entity. Where a transition is performed by both a subject outcome and a set outcome, the verdict is right, but `baseline_refusals` leaves out the set outcome's refusal | Any ess/16 spec with a `moves:` + `instances:` outcome (fixture `ess-compiler/tests/fixtures/set-effects.yaml` has this shape). `transition-to` applies to every transition, `--target` and `--collect` alike (both go through `performers`) | Also read `outcome.instances` where `effect == "moves"` (entity from `instances.entity`), or build the relation from the IR once and write it beside the suite rather than parse `ir.json` twice |

Judgement notes, not cases:
- `/3` is always written, and no flag asks for `/2`. That follows decisions #218 (`/3` unreleased, `--collect` reads `/1`–`/2`). A consumer pinned to `ess-mutation-report/2` breaks on 0.42.0. That is a release-note item, not a defect.
- `--collect` of a 0.41.0 `/2` manifest now applies the baseline-refused-outcome and transition rules from `baseline/ir.json`. So a 0.41.0 emission can re-score `survived` → `unwitnessed`, which exits 3 instead of 1. This is consistent with the changelog draft ("scores no mutant of either `equivalent`"). Where `ir.json` is absent, transition mutants are scored as before (`performers` returns None).

## 5. Attacked, not broken

- `satisfiable` with numbers: only `==`, `!=` and membership reach enumeration. Ordering returns None, so `x > 3 and x < 5` is undecided and never called dead. `Number` equality is by exact value (facts.rs:550): `-0`/`0` and `1`/`1.0` are one value, so `literals.contains(other)` agrees with `evaluate`, and the "other" value is truly other. A `!=` chain of n literals gets n+1 candidates, so an unused one always exists.
- Text with `starts_with`/`contains`/fold, optional leaves, `.count`, collections, Timestamp including newtypes (`terminal` is after unwrap), truthiness of text or number, and fact-vs-fact comparisons: all return None.
- Invariants are not applied before counting, so the answer errs only towards satisfiable. Two spellings of one input (`x` and `input.x`) become independent leaves, which also errs towards satisfiable.
- A transition performed by one witnessed and one unwitnessed subject outcome: `witnessed.contains` returns None, so the verdict stays `survived`.
- `with_dead_guard` is skipped when `hidden`, and `Inconclusive` stands.
- `/2` manifest with `unsatisfiable_guard` is refused; `/1`, `/2` and `/3` are read (:2387).

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/mutate-d/adv2/case-run1.log … case-run5.log, case-final.log
- ~/.cache/ess-wave-n2/mutate-d/adv2/suite-ess-conformance.log, clippy.log
- ~/.cache/ess-wave-n2/mutate-d/adv2/review.md (this file)
- ~/.cache/ess-wave-n2/mutate-d/adv2/baseline-suite.json: a probe dump, deleted
- build output went into the assigned ~/.cache/b10x-target/ess-d-mutate

No worktree lease was taken (the `worktree` CLI has no lease verb).

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/mutate.rs
  line: 1843
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: performers ignores instances set outcomes, so a transition-to or from-drop mutant on a transition only a baseline-refused set outcome performs is scored survived instead of unwitnessed
```
