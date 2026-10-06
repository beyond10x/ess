---
format: aep.planning-md/3
id: review-result:ess-054-w1-2-adversary-1
kind: review-result
status: active
title: Adversary pass 1, unit W1-2 conformance defects
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-427
- reviews: story:feature-request-428
- reviews: story:feature-request-430
revision: 1
---
unit: W1-2 conformance-defects, range 9ffa94d66a..a69a5425a5 in `ess-054-w1-2`, plus my uncommitted test additions
verdict: CONFIRMED (1 red case)
cases: executed 37→41, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths (part 6)
needs-coordinator: whether to replace the first-fit claim rule. The story's own Decision (#427) says "first unclaimed occurrence that carries its values", so fixing finding 1 changes what was decided, not just the code.

**1. `git --no-pager diff --stat`**
```
 .../ess-conformance/tests/event_multiplicity.rs    | 151 +++++++++++++++++++++
 1 file changed, 151 insertions(+)
```
Only a test file changed. I created a probe test file, `tests/adversary_w12_probe.rs`, inside the worktree to explore synthesis and the interpreter, then deleted it before the suite run. Nothing is committed.

**2. Cases added** (all in `crates/verify/ess-conformance/tests/event_multiplicity.rs`)

| case | asserts | now |
|---|---|---|
| `adversary_w12_claim_order_does_not_decide_the_verdict` | The target publishes `Rewrapped` twice, `x` then `y`. Two authored acts each make two claims: one names no value, one names `batch_id: x`. Both orders should pass in Go, TypeScript and Rust. | **red** |
| `adversary_w12_each_act_claims_its_own_occurrences` | Two commands in one `/44` scenario, three claims each, pass an honest target in all three runners. A copy of the emitted Go runtime with the per-command reset (`r.claimed = …`, runtime.go:2165) deleted fails the second act. | green; the mutant is killed |
| `adversary_w12_extra_occurrences_pass_in_every_runner` | Four occurrences against three claims pass in Go and TypeScript, as they already do in Rust. | green |

Red output from the first run of the case on its own (`cargo test -p ess-conformance --test event_multiplicity adversary_w12_claim_order_does_not_decide_the_verdict -- --exact`, exit 101), verbatim except elisions:
```
test adversary_w12_claim_order_does_not_decide_the_verdict ... FAILED
assertion `left == right` failed: demo.batch/authored/any-then-x: Go failed, TypeScript failed, Rust ScenarioResult {
...                    expected: [ "demo.batch.Rewrapped.batch_id = \"x\"", ],
                    observed: [ "batch_id = \"y\"", ],
  left: ("failed", "failed", Failed)
 right: ("passed", "passed", Passed)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out
```

**3. Suite run** (after the cases existed)

Command: `cargo test --locked --offline --no-fail-fast -p ess-conformance --test event_multiplicity --test view_param_binding --test caller_struct_identity --test caller_fresh_identity --test related_copied_view_parameter`. Exit 101.

| binary | result |
|---|---|
| caller_fresh_identity | 9 passed |
| caller_struct_identity | 5 passed |
| event_multiplicity | 11 passed, 1 failed (`adversary_w12_claim_order_does_not_decide_the_verdict`) |
| related_copied_view_parameter | 9 passed |
| view_param_binding | 6 passed |

On the case count: 37 comes from the implementer's own final green log (`<scratch>/green-final.log`) for these five binaries. That run had only 8 event_multiplicity cases. The committed file has 9: `coverage_suite_with_repeated_claims_is_45` was not in that run. It passes here. 38 committed cases plus my 3 gives 41.

`cargo fmt -p ess-conformance --check` exit 0. `cargo clippy -p ess-conformance --test event_multiplicity -- -D warnings` on 1.98.1 exit 0.

**4. Findings**

| # | file:line | what breaks | case | verdict | origin |
|---|---|---|---|---|---|
| 1 | `src/runner.rs:2025`, `src/go/runtime.go:2498`, `src/ts/runtime.ts:3970` | Each claim takes the first unclaimed occurrence that carries its values. A claim naming no value, written first, takes `x` and leaves only `y` for the claim that needs `x`. An honest target then fails `ESS-CF-PAYLOAD`, and all three runners agree on that wrong verdict. This contradicts the comment at runner.rs:1997 ("lets an author write claims in any order"). | `adversary_w12_claim_order_does_not_decide_the_verdict` | CONFIRMED | introduced |
| 2 | `src/runner.rs:3049`, `src/go/runtime.go:2165`, `src/ts/runtime.ts:3376` | No unit case runs a `/44` scenario with two commands. Dropping the per-command reset of claimed occurrences in any runner would therefore pass the unit's own cases. | `adversary_w12_each_act_claims_its_own_occurrences` (closes the gap; the Go mutant is shown killed) | CONFIRMED | introduced |

What reaches finding 1: an authored `events:` list that names one event more than once in an act, with one entry less specific than another and written first. This is the documented authored form. Synthesis does not reach it: repeated `emits` entries get identical claims (synthesize.rs:3044-3054 and 11640).

Suggested fix, not applied: match the act's claims of one event as a set before reporting. Either try claims with more values first, or use a bipartite matching. Then report any claims left over against the occurrences left over. Do the same in Go and TypeScript.

**5. Attacked and not broken**
- **Cross-runner agreement:** distinct payloads in both orders, `x` claimed twice, three claims against two occurrences, extra occurrences, two acts. Rust, Go and TypeScript agreed every time.
- **#428 against the interpreter:** these fixture variants all passed:
  - a caller-swapped run
  - `!=`
  - `all` with two parameters
  - `ref.key` compared under a parameter named `tenant`
  - a root field named like the member
  - a nested struct member
  - an `updates` command
  - a count view
  - a member of a non-identity struct field

  The decoy row was always excluded correctly.
- **#430 member types:** Integer members (all members get one shared value, so the member map stays consistent), a Boolean member, an Optional member, and an ess/22 member copied into an event field: no honest failure. Two Boolean members, or an Integer member capped by a `when:` guard, run out of fresh values; the swap is dropped with a `CrossCallerUnswapped` note.
- **Bytes below `/44`:** the implementer's before/after measurement over 101 models (`<scratch>/bytes-base.txt` and `bytes-treatment.txt`) differs only on the 3 new fixtures. None of the 15 committed authored scenario files repeats an event in one act.
- **Observed, not tested:**
  - TypeScript's `eventuallyEvent` adds to `this.observed` (runtime.ts:4064) and Go's does not. No synthesized or authored act puts a claim after an `eventually_event`.
  - TypeScript's `matches` accepts a wanted `null` on an absent dotted path where Rust and Go refuse it. This predates the unit.
  - TypeScript's one-time admission omits `/36`–`/39`, `/42` and `/43` (one_time_response.ts:611), while Rust and Go admit every major from `/34`. This also predates the unit.
- I did not take a worktree session lease.

**6. Paths written outside the worktree**
- `~/.cache/ess-054-wave/W1-2/adv1/`: `probe1.log` to `probe7.log`, `red-order.log`, `suite.log`, `adversary_w12_each_act_claims_its_own_occurrences.log`, `adversary_w12_extra_occurrences_pass_in_every_runner.log`
- `~/.cache/ess-054-wave/W1-2/tmp/` (TMPDIR, 1.3M)
- `/dev/shm/ess-054/W1-2` (build dir; removed by `cargo clean`)
- `~/.cache/b10x-go-cache/W1-2` (GOCACHE, 951M, left in place)

**7. Findings block**
```findings
- file: crates/verify/ess-conformance/src/runner.rs
  line: 2025
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: 'first-fit claim matching fails an honest target when a claim naming no value precedes one naming a value, in Rust, Go and TypeScript alike, contradicting the any-order promise at runner.rs:1997'
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 2165
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 'no unit case runs a /44 scenario with two commands, so deleting the per-command reset of claimed occurrences in any runner survives the unit suite'
```
