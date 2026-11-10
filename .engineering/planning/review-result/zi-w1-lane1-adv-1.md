---
format: aep.planning-md/3
id: review-result:zi-w1-lane1-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null
relations:
- reviews: story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null
revision: 2
---
unit: story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null (commit 401700ceac, tree zi-t1-adv at 2a03419cd7)
verdict: broken
new failing cases: 3 (adversary_t1_lane1_absent_payload.rs::adversary_t1_lane1_an_error_field_read_absent_is_still_claimed, ::adversary_t1_lane1_a_present_error_field_where_the_row_holds_it_absent_fails, ::adversary_t1_lane1_a_nested_absent_read_passes_on_the_interpreted_target)
mutants survived: 1 (a target that reports a present `agent: "rogue"` on an error where the row holds `agent` absent passes the whole suite)

`git diff --stat` is empty. My only file is the untracked `crates/verify/ess-conformance/tests/adversary_t1_lane1_absent_payload.rs`, and I changed no implementation file. The other adversary's untracked `adversary_t1_lane2_presence_child.rs` is also in the tree, and I left it alone. Cases run: 13 before, 18 after, 3 red.

1. **Blocker, introduced: expected-error fields lost their absent claim.** The commit changed `determined_fields` (`src/synthesize.rs:10055`) to skip every evaluated `null`, and `expect_error` (`:9984`) uses that same function. An error has no shape leaf to take over the claim, so the field is no longer compared at all. On the base, `:10045` skipped `null` only for `absent_by_reference`, so a `{subject: f}` error field held absent was claimed `null`, which expected errors already read as "absent or null". In my test model, a `when_subject:` refusal reads `{subject: agent}`:
   - The claims test finds `owner = "blocked"` and no `agent` key (`left: None, right: Some(Null)`, test file line 346).
   - The real interpreted target passes. The mutant that reports `agent: "rogue"` passes too (line 369, `a target reporting agent: rogue on Blocked passes: {}`).
   - This contradicts the story's own decision, "Error payloads are untouched".
   - Fix: skip the `null` only on the event-payload path, for example by filtering in `determined_payload`, and leave `expect_error`'s fields as they were.
2. **Backlog, origin undecided: a nested absent member fails on the interpreted target.** When `agent` sits inside a struct event field (`holder: {owner: {subject: owner}, agent: {subject: agent}}`), the suite synthesized from that model fails on the interpreted target with `ESS-CF-PAYLOAD`: expected `holder = {"agent":null,"owner":"owner"}`, observed `holder = {"owner":"owner"}` (line 383). The Struct branch of `determined_fields` was not touched by the diff and the acceptance covers top-level fields only. I did not run the base, so origin stays undecided.
3. **Two passing cases kept, because they pin what the unit's test does not.** The unit's test only reads the suite bytes. These two drive acceptance items 1 and 3 through a runner:
   - The reproduction passes on the interpreted target with no presence and with `omitted_when_absent`.
   - A target publishing `agent` present fails `WithdrawToken/outcome/withdrawn` with `ESS-CF-PAYLOAD`, and one publishing `null` passes.

**Command and output.** Every run went through `build.sh` as `test -p ess-conformance --locked --no-fail-fast` with these targets: `adversary_t1_lane1_absent_payload`, `absent_payload_read`, `omitted_or_null`, `adversary_w1fix_pass2_fallback` and `synthesis_suite_bytes_table`. The output is in `adv/run4.txt` (exit 101): my file had 2 passed and 3 failed, and the other four targets were green (3, 5, 3 and 2 passed). Runs 1 to 3 (`adv/run1.txt` to `adv/run3.txt`) were my own fixture mistakes, not findings: an unreachable transition, a `wrong_state` row the interpreter could not build, and a wrapper that ignored `direct_events`.

**Rule bent.** The budget of four cargo runs forced my case and the gate targets into one invocation, so the case was not run alone before the suite. Each target's output is still reported separately.

**Attacked and could not break.**
- `is_absent_readable` covers every source `expression_value` evaluates; the others never evaluate to `null`.
- The callers that pass an empty `before` (`existence.rs:1926`, `synthesize.rs:6379`, `set_effects.rs:1717`) reach no new `null`.

**Written outside the worktree:** all under `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null/adv/`: `run1.txt` to `run4.txt` and `run1.out` to `run4.out`, `patch.pl`, and the `.touch` files `build.sh` may have written. I took and released my own lease (`adv-t1-lane1`).

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/synthesize.rs",
    "line": 10055,
    "category": "contract-drift",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "determined_fields now drops every evaluated null for expect_error too, so an error field the arrangement holds absent is no longer compared and a target reporting a present value there passes"
  },
  {
    "file": "crates/verify/ess-conformance/src/synthesize.rs",
    "line": 10061,
    "category": "boundary",
    "severity": "warning",
    "verdict": "CONFIRMED",
    "origin": "undecided",
    "message": "an absent read inside a struct event field is still claimed as an exact nested null, so the interpreted target that leaves the member out fails its own synthesized suite with ESS-CF-PAYLOAD"
  }
]
```
