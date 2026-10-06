---
format: aep.planning-md/3
id: story:feature-request-197
kind: story
status: implemented
title: a refused command cannot declare the compensating change the service makes before answering
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#197
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:45:01Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-06T09:45:02Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:45:02Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

a refused command cannot declare the compensating change the service makes before answering (beyond10x/ess#197).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# story:feature-request-197 — fit review (beyond10x/ess#197)

Read tree: `gaps-270` (origin/main 6c3a81111 + 22 integration commits). CLI: `ess 0.44.0`; 0.45–0.48 and the unreleased `ess/20` may differ (CHANGELOG.md:3-5).

1. **Need.** A command answers an error and still leaves its subject changed: the upstream refuses a join, the service moves the order to `Offline`, then answers `Refused`. Today nothing states or checks the move, so a service that stops doing it passes. Repro `repro-197/` (ess 0.44.0):
   - `v1-refusal-moves.yaml` (`error:` + `moves:` + `instance:`) is refused as `refusal_mutated_state`, ESS-COMMAND-004.
   - `v2-refusal-affects.yaml` (`affects:` on the refusal) is refused as `missing_declaration`, ESS-COMMAND-005 ("declares `affects:` and no subject").
   - `v3-refusal-sets.yaml` (`sets:` on the refusal) is refused as `unobservable_fact`, ESS-COMMAND-003.
   - Requester's syntax: a `then:` / `compensates:` effect, admitted only on an `external:` refusal, naming one move or a `sets:` of the addressed instance.
2. **Class: gap.** Not a defect: the rule is documented and deliberate (`crates/specify/ess-domain/src/command.rs:2353-2379`; `examples/billing/domains/invoice.yaml:333-335`; `docs/design/cross-record-and-stored-field-guards.md:45`). Not local policy: see Q5.
3. **Already expressible? No.**
   - `affects:` (ess/16) needs a subject: `docs/design/set-effects-over-filtered-instances.md:47-48`, and `v2` above.
   - `instances:` on a refusal is refused too (`command/set_effects.rs:489-505`).
   - Nearest form: `repro-197/alt/v4-accepted-reset.yaml`, an `external:` branch with no `error:` that moves `reset` and emits. It validates on 0.44.0, but it declares an accepted answer, so the error the caller receives is lost. It is not the same fact.
4. **Fit: fails as proposed.**
   - Vocabulary: `then:` / `compensates:` names a concept ESS already spells as `moves:` / `sets:` / `affects:` (red flag 1).
   - Composition: "error ⇒ no change" is held in four places, and every one changes:
     - the domain rule (`command.rs:2353-2379`);
     - the interpreter (`crates/verify/ess-conformance/src/interpret/execute.rs:986-987`: "moves, writes and emits nothing");
     - lifecycle-cause accounting (`crates/specify/ess-domain/src/entity.rs:1049-1055`: a refusal's subject is not a cause, so `reset` reached only on failure has no cause);
     - AEP's `AuditRecord::validate` (`command.rs:2357-2360`).
   - Siblings: the proposal covers `external:` refusals only. Input-guarded refusals (`when:`), `wrong_state` and `when_related` refusals raise the same question (the second-adopter case in Q5 is input/state-decided, not external). Refusing them would need a reason that has not been written.
   - Targets: Entity Runtime, Rust/Go/TS/web synthesis, `ess verify diff` and generated docs all assume a refusal changes nothing. Each would have to handle the new construct or refuse it by name.
   - Overlap with `story:related-record-effects`: both are "an outcome changes more than its subject". That story changes *other* rows on an *accepting* branch and needs no rule change. This one changes the *subject* on an *error* branch and reverses `refusal_mutated_state`. If this is accepted, it should reuse that story's `moves:` / `affects:` spelling rather than add a new key.
5. **Second adopter: yes.**
   - A sign-in answers `InvalidCredentials` and increments `failed_attempts`, locking the account at N.
   - A payment capture is declined upstream: the order moves to `PaymentFailed` and the answer is `Declined`.
   - The need is general. Whether ESS models it as a *refusal* is not settled.
6. **Cost.**
   - Source format: rides the unreleased `ess/20` only if it lands before release (`git log`: c1f4353df).
   - The meaning of "refusal" changes, and the docs and the billing example change with it.
   - New or relaxed diagnostics.
   - Synthesis: assert the error, then read the row back in its target state.
   - Interpreter, lifecycle-cause rule, Entity Runtime (refuse by name), and an `ess-diff` classification.
   - Breaking for no existing document (all are refused today).
7. **Alternatives.**
   - (a) Change nothing: use the `v4` accepted-external form where the answer on the wire is not an error, or a comment otherwise.
   - (b) The requester's `compensates:`, external refusals only: refused (red flags 1 and 2).
   - (c) Separate "answers an error" from "changed nothing": admit `error:` beside `moves:` / `updates:` + `instance:` (and `affects:`) on an explicitly marked failure-with-effect branch, under every condition. It keeps one spelling, but it is a semantic decision about ESS's refusal concept and AEP's audit rule that nobody has made.
   - Hence defer.

## Decisions

- **defer (proposed):**
  - **Why:** the need is real and general (a failed sign-in counter, a declined capture). Every design reverses `refusal_mutated_state`, which the domain, the interpreter (`execute.rs:986`), the lifecycle-cause rule (`entity.rs:1049`) and AEP's audit rule hold.
  - **Decision-blocker** with `blocks: story:feature-request-197`: "May a branch that answers an error change state? If yes, is it a refusal with an effect, or an error-answering branch that states its change? Under which conditions (external only, or also `when:`, `wrong_state`, `when_related`)?"
  - **Requester's `then:` / `compensates:` key is refused in any case:** ESS already spells the move as `moves:` / `affects:`.
  - **Overlap, not a duplicate:** `story:related-record-effects` (#229, effect half). Do that one first. A future #197 design reuses its `affects[].moves` spelling.
  - **Related:** #269 (per-refusal failure policy, bindings) and closed #260 (effect committed, delivery failed). Neither covers this.
  - **Already fixed?** No.

- Coordinator (2026-10-01): defer, blocked by decision-blocker:refusal-may-change-state; after story:related-record-effects.
