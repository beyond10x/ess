---
format: aep.planning-md/3
id: story:feature-request-453
kind: story
status: implemented
title: 'Concurrency: a process-local lock serialising every transition'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#453
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/tests/linearizability.rs
- confidence: cited
  path: website/docs/concepts/ess.md
- confidence: cited
  path: website/docs/guides/verify/explore.md
- confidence: cited
  path: website/docs/guides/verify/one-time-responses.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:29Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:30Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:48:55Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome
Resolve beyond10x/ess#453: Concurrency: a process-local lock serialising every transition.

## Origin
beyond10x/ess#453, filed 2026-10-05; raised by an adopter whose service serialises every transition behind one process-local lock. Under a multi-instance deployment that lock provides no claim at the store. The adopter asks whether the specification states that transitions are serialised.

## Fit review
1. Need: say what a specification promises when two commands on one record race from two instances, so that a suite knows which outcomes it may accept. The requester proposes no syntax. Their repro idea: two concurrent commands on one record from two instances. Minimal form: on `examples/billing`, two clients each send `PayInvoice` for one issued invoice, the calls overlap, and both answer `settled`.
2. Class: a gap in the documentation of existing semantics. No authored surface is missing. ESS already gives the answer. `ess verify conform check-history` accepts a concurrent history only when some sequential order of the calls is one the specification's model accepts, answer for answer (`crates/verify/ess-conformance/src/linearize.rs:1-25`; `website/docs/guides/verify/explore.md:128-155`). This has shipped since 0.39.0 (`website/docs/releases/what-changed.md:176-180`), and the `story:concurrent-history-*` stories are `implemented`. The lifecycle has one `settle` move, `Issued` to `Paid` (`examples/billing/domains/invoice.yaml:156-158`), so the two-`settled` history has no explaining order. The design names this bug class: "two transitions out of one state that both succeed" (`docs/design/concurrent-history-conformance.md:26-28`). The gap is that no concepts or reference page states this as what a specification means. The concept pages say nothing about concurrency (grep of `website/docs/concepts/`). `website/docs/guides/verify/one-time-responses.md:59-60` files "concurrent atomicity" under implementation obligations, without saying a history check exists for the general case.
3. Existing idiom: the promise is linearizability per command, checked by `check-history`. Two instances behind one process-local lock are two clients whose calls interleave. Existing tests already reach a violation for this case: `a_recorded_two_client_history_against_lost_update_is_a_violation` and `lost_update_is_a_violation_exactly_when_the_payments_overlap` (`crates/verify/ess-conformance/tests/linearizability.rs:167`, `:187`). Not run for this review (no builds); cited only. How an implementation serialises (a lock, a store-level claim, optimistic concurrency) is its own business. The model already refuses to guess at shared state: a stateful workload with `replicas.min > 1` and no shared store is `ESS-TOPOLOGY-004` (`crates/specify/ess-domain/src/topology.rs:177-194`; probe `b` in `<fit-review scratch>/probe-432/`).
4. Fit: a `serialised:` / `concurrency:` key would restate what the model already means. It would also open a second, weaker reading (a command not serialised), which no checker, runner or target could represent: the interpreter is sequential (`linearize.rs:1-6`). What the documentation must also bound, so the promise is not overstated: reads are judged subject by subject, which is weaker than one snapshot (`linearize.rs:97-99`); multi-record atomicity of a set effect is not claimed (`website/docs/guides/specify/selection-effects.md:93-94`); commands that read related rows or select a set share one partition (`linearize.rs:21-23`); an unanswered call may or may not have taken effect (`linearize.rs:13-15`).
5. Second adopter: a booking service whose two replicas each accept `ConfirmSeat` for the last seat. Same question, same answer: at most one `confirmed` is explainable. This is general, not one adopter's policy.
6. Cost: one concepts section and one cross-link. No keyword, diagnostic, format bump or generated-API change.
7. Alternatives: (a) change nothing and leave the answer implicit in a verify guide; refused, because an adopter reading the concepts cannot find it. (b) a per-command `serialised: true | false` key; refused for the reason in question 4, and it would need `ess/23` plus a checker mode for "no promise". (c) state the existing promise on the concepts page, with its limits and the check that holds it; chosen.

## Decisions
Decline, with the idiom. No authored surface and no format bump. The story body is the decline record. Add the section `## When commands race` to `website/docs/concepts/ess.md`, after `## What gets derived`, stating:
- every command takes effect at one point between its call and its answer, so a concurrent history must be explainable by one sequential order the model accepts (linearizable, per subject; a shared partition where a command reads related rows or selects a set);
- the mechanism is the implementation's, and a lock local to one process does not serialise across instances;
- the four limits from question 4;
- `ess verify conform check-history` (with the Go and TypeScript concurrent explorers) is the check, and a single-client suite cannot see a race (`explore.md:130-131`).
Link to the section from `one-time-responses.md:59-60` and from `explore.md` "Check a concurrent history". Reply on #453 with this decision.

## Acceptance
- concepts_page_states_the_race_promise: `website/docs/concepts/ess.md` has the heading `## When commands race`, after `## What gets derived`. The section contains "one sequential order", "per subject", "lock local to one process", "`check-history`" and "a single-client suite cannot see a race". It also names the four limits as "subject by subject", "set effect", "shared partition" and "unanswered call". The case reads the page and fails on a missing heading or phrase. It lives in `crates/verify/ess-conformance/tests/linearizability.rs`, beside the two tests that hold the promise: `a_recorded_two_client_history_against_lost_update_is_a_violation` (:167) and `lost_update_is_a_violation_exactly_when_the_payments_overlap` (:187), which stay unchanged and green.
- race_section_is_linked_from_the_verify_guides: `website/docs/guides/verify/one-time-responses.md` (:59-60) and the "Check a concurrent history" section of `website/docs/guides/verify/explore.md` each link `../../concepts/ess.md#when-commands-race`. Checked by `crates/edge/ess-xtask/tests/site_navigation.rs` (`every_relative_link_lands_on_a_page_and_an_anchor`, :324), which fails if the anchor does not land, and by the case above, which also reads the two pages for the link.

## Scope
- website/docs/concepts/ess.md  cited — new "When commands race" section; the concepts pages have no concurrency statement today
- website/docs/guides/verify/one-time-responses.md  cited — :59-60 link to the promise
- website/docs/guides/verify/explore.md  cited — :128 "Check a concurrent history" links back
- crates/verify/ess-conformance/tests/linearizability.rs  cited — gains the page case beside the two promise tests (:167, :187)
