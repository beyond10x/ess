---
format: aep.planning-md/3
id: story:conformance-synthesizes-exchange-scenarios
kind: story
status: draft
title: Conformance synthesizes exchange scenarios
summary: Open, admitted and refused messages, close, follower bound; interpreted target passes; size L
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- depends_on: story:an-exchange-is-declared-in-the-specification
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/command.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/exchange.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/exchange_scenarios.rs
revision: 9
---
## Need

A declared exchange must be checked, not only documented: the suite has to show that the opening
command opens it, that each message is admitted in the states the followed row's lifecycle and the
commands' guards admit and refused elsewhere, that a message after close is refused, and that the
follower bound holds. ADR `messages-are-exchanged-independent-of-the-wire` § 3 (lines 124-126):
conformance synthesizes these from the specification alone; a binding's target adapter carries
them over its wire later.

Reproduction: on 0.92.0 an exchange cannot be declared (`triage/650/repro/run.txt`), so no suite
holds one.

## Acceptance

All tests are in `crates/verify/ess-conformance/tests/exchange_scenarios.rs` (new), run with
`cargo test -p ess-conformance --locked --test exchange_scenarios`, on the `ess/37` specification
declaring `example.calls.Session` (story:an-exchange-is-declared-in-the-specification) with
`max_followers: {count: 1, error: E}`.

- `exchange_open_is_synthesized`: the synthesized suite holds a scenario named
  `example.calls.Session/open` that executes `Connect` and expects the followed row in `Open`.
- `exchange_messages_are_synthesized_per_state`: for each `initiator_sends` command, the suite holds
  one `example.calls.Session/…` scenario per state its guards admit, expecting its accepting
  outcome, and one per state they do not, expecting its refusal; `HangUp` after `Ended` expects
  `HangUp`'s held-state refusal.
- `exchange_responder_events_are_synthesized`: for each `responder_sends` event, the suite holds an
  `example.calls.Session/…` scenario that expects the event after the outcome that emits it.
- `exchange_follower_bound_is_synthesized`: the suite holds a scenario that opens the row, then
  sends a second `Connect` on the same row and expects error `E` and the row unchanged.
- `exchange_scenarios_pass_the_interpreted_target`: `ess verify conform run --target interpreted`
  on the specification and its synthesized suite exits 0.
- `an_unbounded_target_fails_the_follower_scenario`: the same suite, run against the interpreted
  target of the specification with `max_followers.count: 2`, fails exactly the follower-bound
  scenario.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes with no row of
  `synthesis-suite-bytes-table.tsv` moved (no committed model declares an exchange).
- `exchange_scenarios_keep_the_suite_major`: the synthesized suite's `suite_version` is the one the
  same specification without `exchanges:` gets (no `ess-conformance/64` or `/65` is written for an
  exchange).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/synthesize/exchange.rs
- cited: crates/verify/ess-conformance/src/interpret.rs
- inferred: crates/verify/ess-conformance/src/interpret/command.rs
- inferred: crates/verify/ess-conformance/tests/exchange_scenarios.rs

confidence: medium

`synthesize.rs` declares the new `exchange` module, which writes the scenarios;
`interpret/command.rs` enforces the follower bound. Shared surfaces: moves none (no suite-bytes
row, no suite format).

## Fit review

1. **Need.** The suite checks an exchange's open, per-state admission, close and follower bound.
   The requester asked for the binding "validated against the specification" (issue body).
2. **Class: gap.** No scenario kind exists for a conversation (reproduction above).
3. **Already expressible?** Per-state admission of each command is already synthesized for commands
   on their own; what is new is the follower bound and grouping the scenarios under the exchange.
   Without this story the follower bound is a figure the suite never checks (policy 1,
   `docs/design/generic-language-features.md:588-594`).
4. **Fit.** No grid cell (`docs/design/language-grids.md`); reuses the existing held-state and
   guard witnesses for admission (F2 (a), ADR line 165), so no new witness for order. Wire targets
   (HTTP stage 3, a future socket target) carry the same scenarios; that adapter is not here.
5. **Second adopter.** The price feed: `Unsubscribe` after `Closed` refused, a second subscriber
   over a bound of one refused.
6. **Cost.** New scenarios only for specifications with exchanges; no suite format pair (see
   Decisions).
7. **Alternatives.** (a) Change nothing: exchanges are documentation only, unchecked. (b) Wait for a
   wire target: checks nothing until a socket target exists. (c) Chosen: synthesize from the model
   and run on the interpreted target.

## Decisions

**accept as proposed**, as the accepted ADR `messages-are-exchanged-independent-of-the-wire` § 3
states it. Refined: the after-close refusal is the command's own held-state refusal (no separate
error key), matching `story:an-exchange-is-declared-in-the-specification`.

- No suite format moves: every exchange scenario is written with step kinds that exist today
  (execute a command, expect its outcome or error, expect a row's state, expect an event), and the
  follower bound is enforced by the target, not read by the runner, so a runner that admits
  today's suite majors runs them.
- The bound's mutant is a specification mutant (`count: 2`) run through the interpreted target,
  because the interpreter has no mutation switch and the specification mutant gives the same
  over-bound target.

## Source

github:beyond10x/ess#650
