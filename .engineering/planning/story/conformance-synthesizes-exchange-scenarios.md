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
revision: 3
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

- `ess verify conform synthesize` on the specification declaring `example.calls.Session` emits,
  under `example.calls.Session/…` scenario names: open (`Connect`, then the row is `Open`); each
  `initiator_sends` command in each state its guards admit and its refusal in a state they do not
  (`HangUp` after `Ended` answers its held-state refusal); each `responder_sends` event where an
  outcome emits it; with `max_followers: {count: 1, error: E}`, a second opening on the same row
  answered with `E` and no effect.
- `ess verify conform run --target interpreted` passes every synthesized exchange scenario (exit 0),
  and an interpreter mutant that admits a second follower over the bound fails one.
- A specification without exchanges keeps its suite bytes (`synthesis_suite_bytes_table` rows do
  not move).
- The suite format moves only if a scenario needs a new step kind; the design page
  (`docs/design/message-exchanges.md`) decides, and a move adds the ordinary and coverage pair.
- Tests in `crates/verify/ess-conformance/tests/exchange_scenarios.rs` (new).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/synthesize/ (a new exchange module)
- cited: crates/verify/ess-conformance/src/interpret.rs
- inferred: crates/verify/ess-conformance/src/interpret/command.rs (follower bound)
- cited: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs (unchanged rows)
- inferred: crates/verify/ess-conformance/tests/exchange_scenarios.rs (new)
confidence: medium

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
6. **Cost.** New scenarios only for specifications with exchanges; possibly a suite format pair if a
   step kind is needed, decided on the design page.
7. **Alternatives.** (a) Change nothing: exchanges are documentation only, unchecked. (b) Wait for a
   wire target: checks nothing until a socket target exists. (c) Chosen: synthesize from the model
   and run on the interpreted target.

## Decisions

**accept as proposed**, as the accepted ADR `messages-are-exchanged-independent-of-the-wire` § 3
states it. Refined: the after-close refusal is the command's own held-state refusal (no separate
error key), matching `story:an-exchange-is-declared-in-the-specification`.

## Source

github:beyond10x/ess#650
