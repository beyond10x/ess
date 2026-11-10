---
format: aep.planning-md/3
id: review-result:zi-w1-lane2-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for
relations:
- reviews: story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for
revision: 2
---
unit: story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for (beyond10x/ess#729). The findings cover the tree at 2a03419cd7 plus my one untracked test file.
verdict: NEEDS-CHANGE
cases: executed 43→49, red 3
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: 4 paths
needs-coordinator: whether the nested and `not`-wrapped shapes block this story or go to the backlog. The story's Fit review says the rule holds for "any connective … either under not, at any depth". The numbered acceptance items 1–7 do not test those shapes.

**1. `git --no-pager diff --stat`:** empty, so no tracked file changed. The only untracked file I added is `crates/verify/ess-conformance/tests/adversary_t1_lane2_presence_child.rs`. The file `adversary_t1_lane1_absent_payload.rs` is the other adversary's, and I did not touch it.

**2. The cases** (first run of the file alone, `adv/red-lane2.txt`, exit 101; second run with the nested case added, `adv/red2-lane2.txt`, exit 101):
- `a_read_child_beside_a_presence_sibling_under_not_is_not_refused` (line 177): **RED.** The guard `{not: {all: [{prompt: {defined: true}}, {prompt: {starts_with: "hold "}}]}}` is still refused. Verbatim: `ChildUnwitnessed { predicate: All([Not(Defined(FactPath(prompt))), TextMatch { path: FactPath(prompt), op: StartsWith, ...`. That goal needs the field absent and a read of the same field decided, the exact shape #729 describes. The commit re-pins this refusal in `mutation-manifest-4-scoped.json` (`guard-negate/desk.entry.Open/leading-hold`, `/trailing-hold`).
- `..._integer` (line 185): **RED.** Same shape with `not: {all: [defined(minimum), minimum > 10]}`. Verbatim: `ChildUnwitnessed { predicate: All([Not(Defined(FactPath(minimum))), Compare { left: Fact(FactPath(minimum)), op: Gt, ...`.
- `a_nested_presence_child_beside_a_read_is_not_refused` (line 198): **RED.** The guard is `any: [level == 1, {all: [defined(minimum), minimum > 10]}]`. Here the presence child is the one being isolated (the case the change is meant to fix), but nested, so the `swapped` goal applies. `isolating` drops the sibling, but the mutant clause the `swapped` path adds still needs `minimum > 10` decided, so the result is still `ChildUnwitnessed { predicate: All([Not(Defined(FactPath(minimum))), Any([...` on `accepted`.
- `acceptance_5_a_target_reading_absent_as_unequal_fails_recorded`: GREEN. The unit's own test file never runs a target. Kept because it is the only test of acceptance 5's hand-written target.
- `acceptance_5_the_interpreter_passes_every_scenario_of_1_to_4`: GREEN. Kept because no other test runs the interpreter over specifications 1 to 4.
- `a_target_dropping_the_presence_child_beside_a_mixed_sibling_fails`: GREEN. It checks that the lenient rule does not let a target that drops `defined(minimum)` survive beside `any: [minimum > 10, level == 1]`. Kept because nothing else pins that.

**3. Suite** (run after the cases existed; outfiles in `adv/`):
- `build.sh adv/suite-lane2.txt <tree> test -p ess-conformance --locked --no-fail-fast` with `--test` set to presence_child_beside_read, nested_connective_mutants, authored_child_witness, mutation_identical_answer, synthesis_suite_bytes_table and adversary_t1_lane2_presence_child.
  - Result: exit 101. Only my file fails (3 passed, 3 failed).
  - The other targets pass: authored_child_witness 5, mutation_identical_answer 22, nested_connective_mutants 10, presence_child_beside_read 4, synthesis_suite_bytes_table 2.
  - The 43 in the header is those five targets with my file deselected, from this run; 49 includes my 6 cases.
- `build.sh adv/cli-lane2.txt <tree> test -p ess-ess-cli --locked --test mutate_stored_guards`: exit 0, 4 passed.

**4. Findings:**

| file:line | verdict | origin | what reaches it |
|---|---|---|---|
| `crates/verify/ess-conformance/src/synthesize.rs:17590` (`left_absent`) | NEEDS-CHANGE | pre-existing | The narrowing only applies when the isolated child is itself a presence test. When the isolated child is the read and its presence sibling has to be false (a negative `all` or `any`), the goal "absent and read decided" is still refused. This is reached by the repository's own `guard-negate` mutants of `mutation-identical-answer.yaml`, which the commit re-pinned with this refusal. |
| `website/docs/reference/predicates.md:1832` against `synthesize.rs:17814` (`isolating`, `swapped`) | NEEDS-CHANGE | introduced | The commit's doc text says a presence child "is witnessed by the input that leaves x out, even beside a comparison on x", with no depth limit. When the child is nested under `any`/`not`, the code still refuses with ESS-SYNTH-022. Reached by any guard of the shape `any: [a, all: [defined(x), x op v]]`. |

Possible fix for both, which I did not apply: when the goal needs a path absent while a read under it is decided, directly or through the `swapped` clause, do not refuse. `crate::witness::unsatisfiable` already has a branch for refusing on unsatisfiable goals, and this case could go there.

**5. Attacked and not broken:**
- Spec 1 and spec 2: the interpreter passes, and acceptance 5's wrong target fails `recorded`.
- `any: [not defined(minimum), minimum > 10]` sends rows to the correct branch.
- A target that drops the presence child beside a mixed sibling is caught.
- `not not defined` and `missing` cannot be reached: `Predicate::not` folds a double `not`.
- `mutate_stored_guards` and the byte pins stay green.

**6. Paths written outside the worktree:**
- `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-connective-child-masked-by-an-earlier-branch-is-not-asked-for/adv/red-lane2.txt`
- `.../adv/red2-lane2.txt`
- `.../adv/suite-lane2.txt`
- `.../adv/cli-lane2.txt`

Builds ran in the shared slots `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/targets/{A,B}`, as `build.sh` directs.

**7.**
```findings
[
  {
    "file": "crates/verify/ess-conformance/src/synthesize.rs",
    "line": 17590,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "pre-existing",
    "message": "a read child isolated beside a presence sibling that must be false (not: {all: [defined(x), x op v]}) is still refused with ESS-SYNTH-022 on a goal needing x absent and the read decided, and the commit re-pins that refusal in mutation-manifest-4-scoped.json"
  },
  {
    "file": "website/docs/reference/predicates.md",
    "line": 1832,
    "category": "contract-drift",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "the new rule text promises any presence child is witnessed beside a read of x, but a nested one (any: [level == 1, {all: [defined(minimum), minimum > 10]}]) is still refused because the swapped goal at synthesize.rs:17814 still needs the read decided"
  }
]
```
