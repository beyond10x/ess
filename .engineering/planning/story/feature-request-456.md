---
format: aep.planning-md/3
id: story:feature-request-456
kind: story
status: implemented
title: when_subject_state names one state, not a list
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#456
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/listed_subject_states.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_state_scoped_pass1.rs
- confidence: cited
  path: website/docs/guides/specify/guards-and-predicates.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T14:53:32Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T14:53:32Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:49:01Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome
Resolve beyond10x/ess#456: when_subject_state names one state, not a list.

## Origin
beyond10x/ess#456, filed 2026-10-05; a downstream specification wrote four identical branches, one per live state, for a command that preserves its subject in any of the four.

## Fit review
1. Need: one outcome branch that holds in any of several held lifecycle states, with synthesis witnessing each of them. Minimal reproduction, written fresh: a `Ticket` with states `New, Triaged, Active, Waiting, Closed`; `TouchTicket` preserves the ticket in the four live states and refuses a closed one (`<fit-review scratch>/probe-456/spec/system.yaml`). The requester proposed `when_subject_state: [A, B, C, D]` or `when_subject_state: {in: [...]}`.
2. Class: convenience, already delivered. The list form has been in the language since `ess/18`, release 0.41.0 (`website/docs/reference/spec-versions.md:57`; `website/docs/reference/predicates.md:41`). No defect: the list is admitted on accepting branches as well as on refusals (`docs/design/subject-state-outcome-guards.md:59-61`, `:71-77`).
3. Existing idiom: `when_subject_state: [New, Triaged, Active, Waiting]` beside `preserves:` and `instance:`. On the installed `ess 0.52.0` (the probe did not use the 0.53.0 binary), `ess specify validate --path spec` printed `probe v1 — 1 file(s), valid`. `ess verify conform synthesize` wrote one scenario, `probe.ticket.TouchTicket/outcome/kept`, that arranges and observes a row in each listed state: `jq` over its steps found 3 observations each of `New`, `Triaged`, `Active` and `Waiting`. That is the `Listed(n)` witness run (`crates/verify/ess-conformance/src/synthesize.rs:2903-2905`). It is held by `a_listed_accepting_guard_is_witnessed_in_every_state_it_lists` (`crates/verify/ess-conformance/tests/adversary_state_scoped_pass1.rs:256`), whose mutants answer correctly in only one of the listed states. A list in any order compiles to one IR (`crates/specify/ess-domain/src/entity.rs:149-157`; test at `adversary_state_scoped_pass1.rs:347`). An alternative spelling that already exists is `when_subject: {predicate: state in [...]}`, from `ess/18` (`predicates.md:26`).
4. Fit: the `{in: [...]}` form would be a second spelling of an existing construct. That is a red flag in `.agents/skills/assessing-external-requests/SKILL.md` ("names a concept ESS already spells differently elsewhere"). It is refused today with "`when_subject_state` is one state, such as `Shipped`, or a list of states, such as `[Delivered, Cancelled]`" (`entity.rs:220-228`, reproduced in `probe-456/in-form/`). The list form already composes with `when:` (`crates/verify/ess-conformance/tests/state_scoped_refusals.rs:870`), refusals, the finite partition proof (`subject-state-outcome-guards.md:67-70`) and Entity Runtime lowering to membership of `$from_state` (`subject-state-outcome-guards.md:77-78`).
5. Second adopter: an order kept unchanged on a re-sent `ship` while `Shipped` or `Delivered`. This is the exact shape of `listed_accepting()` in `adversary_state_scoped_pass1.rs:224-231`. It is already a domain fact ESS expresses.
6. Cost: nothing new to build. A docs-only change is optional: the guide shows the list form only on a refusal (`website/docs/guides/specify/guards-and-predicates.md:44-48`), which is why an adopter could miss that it works on `preserves:`. No format, keyword, diagnostic or diff change.
7. Alternatives: (a) change nothing and reply with the idiom; (b) the idiom plus one guide example of a listed accepting branch; (c) admit `{in: [...]}` as well, which was rejected because it adds a second spelling, a new `HeldStates` deserialization arm, and a reason for every target to read two shapes for one fact. (b) is chosen. The predicate reference page's runner splices only `when`, `invariants` and `filter` fragments (`crates/edge/ess-cli/tests/predicate_reference_page.rs:9-12`), so an outcome-list example belongs in the guide. The requester's first option is the existing syntax. Their second option is refused.

I don't know which format header the downstream specification declared. If it was below `ess/18`, the list was refused as `unsupported_format_version`, and the answer is to raise the header.

## Decisions
Decline, with the idiom. The story body is the decline record, with the probe output above. The idiom is `when_subject_state: [S1, S2, …]` (`ess/18`, 0.41.0) on any subject branch, including `preserves:`. Synthesis already witnesses each listed state. `{in: [...]}` stays refused. In `website/docs/guides/specify/guards-and-predicates.md`, add the subsection `### Keep a record unchanged in any of several states` inside `## Select an outcome from the held subject state`, after the `ShipOrder` example. It shows one listed `preserves:` branch, the shape `listed_accepting()` tests. Reply to the requester. No format bump: no `ess/23` and no `ess-conformance/N`.

## Acceptance
- a_listed_accepting_guard_is_witnessed_in_every_state_it_lists: the existing test stays green, and it holds the guide example's shape (`crates/verify/ess-conformance/tests/adversary_state_scoped_pass1.rs:224-256`).
- listed_preserves_guide_example_is_present_and_valid: `website/docs/guides/specify/guards-and-predicates.md` has the heading `### Keep a record unchanged in any of several states` after the `ShipOrder` example. Its fenced YAML contains `preserves:` and `when_subject_state: [`, and validates when spliced into a minimal entity with those states. The case reads the page and fails on a missing heading, missing block or refused block.
- in_form_is_refused_with_the_list_hint: `when_subject_state: {in: [New, Active]}` is refused with the message from `crates/specify/ess-domain/src/entity.rs:225-226`, "`when_subject_state` is one state, such as `Shipped`, or a list of states". No existing test pins that message (grep of `crates/**/tests` for "or a list of states" finds none), so this case is new.

Both new cases live in `crates/specify/ess-domain/tests/listed_subject_states.rs`.

## Scope
- website/docs/guides/specify/guards-and-predicates.md  cited — the list form is shown only on a refusal at lines 44-48; add the listed `preserves:` subsection
- crates/verify/ess-conformance/tests/adversary_state_scoped_pass1.rs  cited — the existing witness test the idiom rests on
- crates/specify/ess-domain/src/entity.rs  cited — the `HeldStates` refusal for `{in:}`, unchanged
- crates/specify/ess-domain/tests/listed_subject_states.rs  inferred — the guide-example and `{in:}` refusal cases
