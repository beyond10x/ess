---
format: aep.planning-md/3
id: review-result:adversary-served-events-pass-1
kind: review-result
status: active
title: Adversary pass 1, generated server published events
relations:
- reviews: story:generated-server-publishes-and-reads-headers
revision: 1
---
unit: story:generated-server-publishes-and-reads-headers, uncommitted working tree the ess-sf-http worktree on HEAD 393d8f5ca0 (0.46.0)
verdict: NEEDS-CHANGE
cases: executed 557→569, red 5
origin: introduced 5 / pre-existing 0 / undecided 1
wrote-outside-worktree: 16 paths under ~/.cache/ess-sf/ (listed in part 6)
needs-coordinator: whether a Go served surface must be safe for concurrent requests (the fix is a mutex in the generated ServeX handler); whether "input-absent → 400" is in this story's scope

## 1. git --no-pager diff --stat

Tracked diff: `30 files changed, 1735 insertions(+), 202 deletions(-)`, all the implementor's; I changed no tracked file.
My only changes are 3 new, untracked test files:
- crates/generate/ess-gen/tests/adversary_published_pass1.rs
- crates/generate/ess-synth/tests/adversary_served_pass1.rs
- crates/generate/ess-synth/tests/adversary_served_pass1_gatepass.rs
Charter check: 0 non-test paths.

## 2. Cases added (red output from the run of that case alone, before the suite)

| test | asserts | now |
|---|---|---|
| adversary_published_pass1::the_right_answer_for_a_two_event_branch_validates | [Opened, Stamped] validates | green |
| ::the_contract_refuses_a_two_event_branch_answered_out_of_publication_order | [Stamped, Opened] is refused | RED |
| ::the_contract_refuses_a_two_event_branch_answered_with_one_event_twice | [Opened, Opened] is refused | RED |
| adversary_served_pass1::a_two_event_branch_is_answered_in_emit_order_and_validates | served order == emit order; validates | green |
| ::every_refusal_branch_answer_validates_against_the_served_contract | 422 / 502 / 502 answers validate, published [] | green |
| ::a_command_without_input_is_answered_and_validates_with_or_without_a_body | Ping with `{}` and with no body → 202 | RED (no body → 400) |
| ::after_a_failed_delivery_the_next_answer_is_its_own_and_nothing_accumulates | 501, then the next answer is its own, kept=0 | green |
| ::a_failed_pump_does_not_lose_the_other_bindings_delivery | tally-on-open ran for all 3 Opened | RED (2) |
| ::headers_keep_arrival_order_duplicates_lower_cased_names_and_trimmed_utf8_values | real socket: order, duplicates, lower-casing, trimming, UTF-8 | green |
| ::a_hundred_headers_are_kept_and_the_hundred_and_first_is_431 | 100 → ok, 101 → 431 | green |
| adversary_served_pass1_gatepass::rust_and_go_gatepass_servers_answer_every_command_identically_and_to_their_contract | 10 steps: same status and JSON from both, each valid against its served contract | green |
| ::the_go_gatepass_server_answers_concurrent_commands_without_a_data_race | 8×10 concurrent registers are all answered, no race | RED |

Red output, verbatim (logs adv1-gen-red.log, adv1-served-red.log, adv1-gatepass-race-red4-1.log):

```
the contract says `published` is in publication order, and admits {"outcome":"opened","published":[{"event":"ops.core.Stamped","payload":{"id":"t"}},{"event":"ops.core.Opened","payload":{"id":"t"}}]}
the branch publishes `Opened` and `Stamped` once each, and the contract admits {"outcome":"opened","published":[{"event":"ops.core.Opened","payload":{"id":"t"}},{"event":"ops.core.Opened","payload":{"id":"t"}}]}
assertion `left == right` failed: `tally-on-open` ran for every published `Opened`, including the one whose pump failed on `note-on-open`
  left: 2
 right: 3
assertion `left == right` failed: `ping-no-body`: {"refused":"the body is not JSON: at byte 0: expected a value"}
  left: 400
 right: 202
78 of 80 concurrent commands were never answered; 36 goroutines were inside System.Pump when the server was stopped; data race reported: true
goroutine 25 [runnable]: runtime.asyncPreempt … passservice.(*PassService).DrainOutbox … system.(*System).collect … system.(*System).Pump
```

The first version of the Go case had no read timeout. It hung for over 10 minutes with the server at 178% CPU and 7 connections stuck (adv1-gatepass-red-run1-hung.log). I then added timeouts and a SIGQUIT goroutine dump.
Out of 5 timed runs: 3 ended with the server alive but spinning (72, 75 and 78 of 80 unanswered). 2 ended with the server crashed by `fatal error: concurrent map writes` in the hand-written realization.

## 3. Suite, after the cases existed

Command: `CARGO_TARGET_DIR=$HOME/.cache/b10x-target/ess-sf-http CARGO_INCREMENTAL=0 cargo test --locked -p ess-gen -p ess-synth --no-fail-fast`. Result: EXIT=101.
- 3 targets failed, all mine: `ess-gen --test adversary_published_pass1`, `ess-synth --test adversary_served_pass1`, `ess-synth --test adversary_served_pass1_gatepass`.
- All targets: 564 passed, 5 failed.
- Without my 3 binaries: 557 passed, 0 failed. That is the `before` figure, taken from this same run with my files excluded.

## 4. Findings

| # | file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|---|
| 1 | crates/generate/ess-synth/src/go/http.rs:1159 | NEEDS-CHANGE | introduced | The Go served handler now calls `system.Pump()` and `TakePublished()`. `http.Serve` (passservice.go:90) runs each connection on its own goroutine, and nothing locks the shared log or cursor. Under 8 concurrent clients the Pump loop (`system.go:120`, `for { collect; if cursor == len(published) {return}; cursor++ }`) livelocks: goroutines stay runnable inside Pump/collect, 72–78 of 80 requests are never answered, and the server stays up at ~180% CPU. | Any two simultaneous clients of a generated Go server; net/http is concurrent by default. The realization's own map race (it crashed 2 of 5 runs) existed before this change. The livelock is in generated code, so a correctly locked realization cannot prevent it. Fix: a `sync.Mutex` held across each served command, from the port call through TakePublished. |
| 2 | crates/generate/ess-synth/src/rust/system.rs:823 (take_published doc), mechanism at system.rs:1000 | CONFIRMED | introduced | `take_published`'s doc says "Events the pump has not yet delivered stay on the log… taking never skips a binding". The pump advances `cursor` before `deliver`. When `note-on-open`'s owed escalation is unmet, the pump fails and `tally-on-open` never runs for that `Opened`. The next served command's `take_published` then drains it. Tally ran 2 times for 3 Opened (`at_least_once`). The Go pump behaves the same (`s.cursor++` before `s.deliver`). | Any served system whose binding obligation is still the Unimplemented stub, which is the default until realized. The served dispatch is a new caller of pump+take, so the lost delivery is newly reached, and once taken it cannot be `redeliver`ed. Fix: advance the cursor only after `deliver` succeeds (duplicates are allowed under at-least-once), or rewind it to the failed event on `Err`. |
| 3 | crates/generate/ess-gen/src/openapi.rs:1017 (`"items": items`) | CONFIRMED | introduced | `published` is typed as `items: oneOf[...]` with min/maxItems = emit count. For a branch emitting [Opened, Stamped], the contract accepts [Stamped, Opened] and [Opened, Opened], while its own description (openapi.rs:978) says "in publication order". | Any branch with two or more emits. That is legal in the model (items.rs numbers repeats), but no committed example has one. Fix: `prefixItems` in emit order plus `items: false`, keeping minItems 0 for the unknown-instance answer. |
| 4 | crates/generate/ess-synth/tests/adversary_served_pass1.rs:482 (served no-input command) | CONFIRMED | undecided | The contract for a command with no input declares no `requestBody`, but the served Rust surface answers a POST without a body with 400 "the body is not JSON". A client that follows the contract is refused. | Any command with no input, called by a client that follows the contract. The refusing code is outside this diff, but I did not run it against the base. |

Judgement residue, not made into cases:
- A 501 now means two different things. Before, it meant "the port was not run". Now it can also mean "the command's effect and events are committed, and delivery failed" (rust/http.rs `settle`, go/http.rs:1159). 501 is not declared in the contract, and a client that retries duplicates the effect. Tree: same as above. Verdict CONFIRMED, origin introduced, severity note.
- `http::Request` has a new public field `headers`, so any shell that builds a `Request` literal no longer compiles. The unit's own harnesses had to be edited for this. The changelog should say so. Severity note.

## 5. Attacked and not broken

- Rust served answers against the served schema, for every branch kind in my model (accepting with 2 events, 422, 502 external, no-input with `{}`) and in gatepass (202, 422, 409 wrong-state with payload, 409 unknown instance without payload): all validate.
- `published` order equals emit order on Rust (ops model) and on Rust and Go (gatepass).
- Cascade: events published by a binding's command (Noted, Tallied, NoteDropped) never appear in the answer, and kept=0 after every successful answer.
- After a 501 the next answer carries only its own events and kept returns to 0. No half-taken state is visible in any answer.
- Rust server concurrency: `SERVE_BODY` is a sequential accept loop, so two requests cannot interleave.
- Headers over a real socket: arrival order, duplicate names kept twice, mixed case lower-cased, OWS trimmed, UTF-8 values kept; exactly 100 headers accepted, the 101st answered 431.
- Rust and Go gatepass across all 3 commands and 10 steps (optional `printed_at` present and absent, base64, unknown instance): same status and same JSON, and each serves the same contract.

## 6. Paths written outside the worktree

- ~/.cache/ess-sf/adv1-gen-red.log
- ~/.cache/ess-sf/adv1-served-red.log
- ~/.cache/ess-sf/adv1-gatepass-red.log
- ~/.cache/ess-sf/adv1-gatepass-red-run1-hung.log
- ~/.cache/ess-sf/adv1-gatepass-race-red.log
- ~/.cache/ess-sf/adv1-gatepass-race-red2.log
- ~/.cache/ess-sf/adv1-gatepass-race-red3-{1,2,3}.log
- ~/.cache/ess-sf/adv1-gatepass-race-red4-{1,2,3}.log
- ~/.cache/ess-sf/adv1-suite.log, ~/.cache/ess-sf/adv1-suite-counts.txt
- ~/.cache/ess-sf/adv1-review.md (this file)
- Build output in the assigned dir $HOME/.cache/b10x-target/ess-sf-http. My test scratch goes under its tmp/ and each test deletes it. I deleted my two leftover `tmp/adversary-gatepass-go-race-*` binaries. The Go build cache under ~/.cache/go-build is shared.

## 7. findings

```findings
- file: crates/generate/ess-synth/src/go/http.rs
  line: 1159
  category: concurrency
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the Go served handler pumps and takes from the shared log with no lock while net/http serves connections concurrently, and Pump livelocks, leaving 72-78 of 80 concurrent commands unanswered at ~180% CPU
- file: crates/generate/ess-synth/src/rust/system.rs
  line: 823
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: take_published promises it never skips a binding, but a pump that fails on one binding has already advanced past the event, so the next served take drains it and the other binding's at-least-once delivery is lost for good
- file: crates/generate/ess-gen/src/openapi.rs
  line: 1017
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the published schema uses unordered oneOf items, so a two-event branch's contract accepts the events reversed or one of them twice despite describing publication order
- file: crates/generate/ess-synth/tests/adversary_served_pass1.rs
  line: 482
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: a command with no input declares no requestBody in its contract, yet the served surface refuses a bodiless POST with 400
- file: crates/generate/ess-synth/src/rust/http.rs
  line: 869
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: 501 now also means the command's effect and events are committed but delivery failed, which the contract does not declare, and a retrying client duplicates the effect
- file: crates/generate/ess-synth/src/rust/http.rs
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the new public Request.headers field breaks every shell that builds a Request literal, and the changelog should name that
```
