---
format: aep.planning-md/3
id: review-result:ess-054-w2-2-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W2-2 aggregate parameters
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-438
- reviews: story:feature-request-439
revision: 1
---
unit: W2-2 aggregate-params, range 9bcbcffae9..f52e0f3b3d, plus one uncommitted adversary test file in the W2-2 worktree
verdict: NEEDS-CHANGE
cases: executed 0→8 (new binary `adversary_aggregate_params_pass1`; the unit's targets ran 223 per the implementer's `final-conf.log` and were not re-run), red 3
origin: introduced 2 / pre-existing 0 / undecided 1
wrote-outside-worktree: 2 directories: `~/.cache/ess-054-wave/W2-2/adv1/` and `/dev/shm/ess-054/W2-2` (removed by `cargo clean`)
needs-coordinator: whether finding 1 holds the unit (I rated it a blocker)

**1. `git --no-pager diff --stat`**

The tracked diff is empty. The only change is one new test file:
```
?? crates/verify/ess-conformance/tests/adversary_aggregate_params_pass1.rs
```
No implementation files were changed.

**2. Cases added**

How the test works: the suite is synthesized from the honest model. The native interpreter then runs that suite against a mutated model that behaves like a faulty target. "Ignores `from`" becomes `{any: [started_at >= param.from, started_at < param.from]}`, and "inclusive `to`" becomes `started_at <= param.to`.

| Case | Asserts | Now |
|---|---|---|
| `adv_window_beside_a_state_conjunct_witnesses_its_bounds` | the window fixture with `filter: [state == Reviewed, started_at >= param.from, started_at < param.to]`, where Reviewed is reached by one move: the ignores-from and inclusive-to mutants fail both views | red |
| `adv_membership_hint_for_a_scalar_parameter_names_a_form_that_validates` | for `queue_id: {in: param.queue}` with `queue: Integer`, the `exists` form the refusal suggests validates | red |
| `adv_in_ignore_case_naming_a_parameter_is_refused_like_in` | `[{label: {in_ignore_case: [param.queues]}}, param.queues.count >= 0]` is refused, as the `{label: [param.queues]}` control is | red |
| `adv_window_mutants_fail_the_committed_fixture` | control: the same mutants fail the committed fixture | green |
| `adv_window_other_orderings_are_witnessed` | `param.from < started_at`, `param.to > started_at`: the honest model passes; inclusive-from and ignores-to fail | green |
| `adv_window_rows_respect_a_clock_guard_on_the_creating_command` | creating command has a `stale` branch on `input.started_at < now - 1h`: the view is either refused by name or passes honestly | green |
| `adv_list_views_pass_honestly_and_fail_when_the_list_is_ignored` | the honest list views pass; the ignores-list mutant fails `InQueues` | green |
| `adv_list_selector_written_the_other_way_round_passes_honestly` | `q == queue_id`, with the empty-list disjunct second, passes honestly | green |

Red output, each case run alone. The window case was rerun once after I changed it to collect every surviving mutant instead of stopping at the first; this is the second run:
```
assertion `left == right` failed: a target with one of these defects passes the window view
  left: ["ignores-from demo.window.CallsInRange/aggregate: Some(\"passed\")", "ignores-from demo.window.QueueCallsInRange/aggregate: Some(\"passed\")", "inclusive-to demo.window.CallsInRange/aggregate: Some(\"passed\")", "inclusive-to demo.window.QueueCallsInRange/aggregate: Some(\"passed\")"]
 right: []
```
```
the hinted rewrite is refused:
[type_mismatch] view.demo.calls.InQueues.filter: `exists x in param.queue: (queue_id == x)`: quantifier target `param.queue` is `Integer` (Number) and not a collection; Forall/Exists require List or Map
```
```
`label: {in_ignore_case: [param.queues]}` validates, reading the text
```
Two cases failed on their first run because I had built them wrongly. They are not findings, and I rewrote both before counting them:
- `0 == param.queues.count` does not parse.
- The first `in_ignore_case` version was refused as `unobservable_fact` because the parameter was read nowhere else.

**3. Suite run, after all cases existed**

`cargo test --locked --offline -p ess-conformance --test adversary_aggregate_params_pass1` → `test result: FAILED. 5 passed; 3 failed`, exit 101. The run was repeated after `rustfmt` with the same result. `cargo fmt -p ess-conformance --check` passes.

**4. Findings** (they cover f52e0f3b3d plus the uncommitted test file)

| # | file:line | What breaks | Case | Verdict / origin |
|---|---|---|---|---|
| 1 | `crates/verify/ess-conformance/src/synthesize/aggregate.rs:1165` (`window_rows`) | Rows outside the window are only the rows the plan already counts as refuted (x, c and the y clones of x). Each is arranged in the cheapest state that refutes it, so it stays in the initial state. When another conjunct, such as a non-initial `state ==`, already refutes them, a target that ignores a bound or makes `to` inclusive passes both views. This contradicts `docs/design/read-api-view-idioms.md:216` and the range-selector paragraph in `aggregate-group-selection.md`. **Reached by:** any window view with a conjunct those rows fail by default, e.g. "completed calls in range". No such view is committed in the repo. **Fix:** arrange each outside row so every non-window conjunct holds, or refuse the view by name. | `adv_window_beside_a_state_conjunct_witnesses_its_bounds` | NEEDS-CHANGE / introduced |
| 2 | `crates/specify/ess-domain/src/expression.rs:1862` | For a scalar `param.`/`input.` operand, the hint suggests `exists: {in: param.queue, …}`, which is itself refused. At the base the refusal had no hint. **Reached by:** an author writing `{in: param.x}` against a scalar parameter. **Fix:** suggest `exists` only for a List or Map; for a scalar, suggest `queue_id == param.queue`. | `adv_membership_hint_for_a_scalar_parameter_names_a_form_that_validates` | CONFIRMED / introduced |
| 3 | `crates/specify/ess-domain/src/expression.rs:2871` (`FoldMatch`) | `in_ignore_case: [param.queues]` still compares against the text `param.queues`; this is the same misread the unit refuses for `[param.queues]`. The story's operator list leaves it out. I did not run it against the base; the diff does not touch this arm. | `adv_in_ignore_case_naming_a_parameter_is_refused_like_in` | CONFIRMED / undecided |

**5. Attacked and not broken**
- List witnesses pass under the honest interpreter. The ignores-list mutant fails, and the reversed membership and disjunct order pass. The first-element-only and `[]` faults are caught by the unit's three runtimes; I found no case that slips past them.
- The offset-spelled instant parses as an instant in the interpreter and in the Go and TypeScript fixture runtimes. Exclusive and inclusive bounds written either way round are witnessed.
- The membership refusal covers only a whole `param.<ident>` or `input.<ident>` word. `param.x y`, `param.timeout-ms` and `param.v2.0` are not refused, and the scalar shorthand `key: param.x` is still a literal. The exact text `param.x` in a String membership list is refused in every format, which is the cost the story documents (Fit review 6).
- Old-format suite bytes: for equality selectors, `selecting`, `selections` and `unheld` produce the same output as before, and the new paths only replace former ESS-SYNTH-017 refusals. I checked this by reading the code, not by diffing a generated suite.
- I took no worktree lease: the `worktree` CLI shows no acquire command.

**6. Paths written outside the worktree**
- `~/.cache/ess-054-wave/W2-2/adv1/`: `env.sh`, `adv1-438.txt`, `adv1-439.txt`, `build.log`, `suite.log`, `suite2.log`, 12 `case*.log` files and an empty `tmp/`.
- `/dev/shm/ess-054/W2-2`: build output, removed by `cargo clean` (396.2 MiB).
- `~/.cache/b10x-go-cache/W2-2`: created with `mkdir -p` if it was missing; nothing written into it.

```findings
- file: crates/verify/ess-conformance/src/synthesize/aggregate.rs
  line: 1165
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: 'window rows outside a bound are only the plan''s refuted rows, so beside a non-initial state conjunct a target that ignores from or makes to inclusive passes both window views'
- file: crates/specify/ess-domain/src/expression.rs
  line: 1862
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'the membership refusal hint names exists over a scalar parameter, a rewrite the checker itself refuses as not a collection'
- file: crates/specify/ess-domain/src/expression.rs
  line: 2871
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: 'in_ignore_case: [param.queues] still validates as the text literal, the misread the unit refuses for the other membership operators'
```
