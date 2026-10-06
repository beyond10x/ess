---
format: aep.planning-md/3
id: story:feature-request-465
kind: story
status: active
title: Caller-swapped run resends the same identity to a non-creating command whose instance the target arranges for a forced outcome
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#465
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/caller_addressed_identity.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/caller_fresh_identity.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/caller-addressed-identity.yaml
- confidence: cited
  path: docs/design/caller-values.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T00:09:32Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-06T00:09:33Z", actor: "human:timo", revision: 5}
---
## Outcome
Resolve beyond10x/ess#465: Caller-swapped run resends the same identity to a non-creating command whose instance the target arranges for a forced outcome.

## Origin
beyond10x/ess#465, filed 2026-10-05; an adopter's specification (`ess/22`, ess 0.53.0) whose target arranges the addressed record for a forced external refusal; 3 scenarios error in the caller-swapped run. Reproduced minimally and fresh in `<fit-review scratch>/probe-465/`.

## Fit review
1. Need: the caller-swapped run must not send an identity the first run already sent, including one sent only to a command that does not create it. Minimal reproduction `probe-465/a/` (`ess/22`, installed ess 0.53.0):
   - `Batch` keyed `transaction: Uuid`. `PrepareBatch` creates it. `CommitTransaction {tenant, transaction}` has `invalid-reference` (`external:`, `error:`), `committed` (moves, `sets: {committed_by: {caller: subject}}`) and `wrong-state`.
   - `CommitTransaction/outcome/invalid-reference` sends `transaction 00000000-0000-4000-8000-d624f5690fe3` as `subject-262144` and then, swapped, the same literal as `subject-262145` (`a.sends.txt`). That is the issue's literal byte for byte.
   - `wrong-state` resends its literal the same way. The interpreter passes all 5 (`a.run.out`), because it keeps nothing under an identity a refused command named.
   - The requester proposes (theirs): redraw every identity input the swapped run sends, or drop the swapped run where a forced external arrangement creates the subject.
2. Class: defect. The swapped run is documented to stay apart from the first: "renames every instance and instant it binds" (`docs/design/caller-values.md:138`). Each fresh identity is drawn "apart from every other row a target the scenarios share may hold" (`crates/verify/ess-conformance/src/synthesize/caller.rs:1061-1064`). The code applies this only to creating commands:
   - `Identities::of` collects identity inputs only where `existence::identity_input` finds a `creates:` branch (`caller.rs:1089-1110`; `existence.rs:51-74`).
   - `Identities::sent` skips every other command (`caller.rs:1152`).
   - So a literal sent to a non-creating command (`ResolvedInstance::Supplied`, `ess-compiler/src/ir.rs:782-786`) is copied verbatim into the second run.
   - `configure_external_outcome` is a test adapter control whose means the target chooses (`scenario.rs:2107-2116`). A target that arranges real state under the sent identity is within the contract, and the shared literal is what breaks it.
3. Existing idiom: none. The adopter's only way out is a target reusing a record it arranged earlier, which hides the case (the issue's Workaround). Making the subject's creation a suite step is not available to the adopter: synthesis does not arrange a subject for an external refusal that touches no entity (probe: the first run sends the literal with no `PrepareBatch`).
4. Fit: the change is synthesis only, in `caller.rs`, with no key, IR or format change.
   - (a) `Identities` also collects, per command, every input an outcome names as a `Supplied` instance (moves, updates, deletes, refusals beside them). `sent` returns those literals too, and `types` (therefore `keys`, `derived` and `member_parameters`) widens to their types.
   - (b) `drawn`/`draw` already keep each sending step's branch through `keeps_branch` and draw struct identities member-fresh (#430). Replacement is by value (`redraw`, :1439-1484), so a literal that also names a row the same run created is replaced consistently.
   - (c) Exhaustion keeps today's answer. A one-value type goes to `one_row_acted_on` (:835-876), which exists for acting on a singleton the command did not create. Otherwise the run is dropped with `Note::UnswappedCallers`.
   - Rows established by setup or seeds are already excluded (`established`, :805-810; `append`, :773-788), so a seeded identity is never redrawn.
   - The requester's second option, dropping the swapped run, is refused on evidence. Synthesis cannot see what a target arranges, so it would have to drop the swapped half of every external scenario on a non-creating command. That loses the two-caller check (`caller-values.md:131-135`) the run exists for.
   - Targets: no runner or interpreter change (the interpreter passes both shapes).
5. Second adopter: a fulfilment service whose `ShipOrder {order_id}` has an external `carrier-rejected`, for which its target arranges a packed order under the sent id. Also a ticketing service whose `Escalate {ticket_id}` has an external `pager-unreachable`.
6. Cost: no source format, suite format, keyword or diagnostic. Suite bytes change, at every source format, only for caller-reading models whose swapped run sends a literal identity to a non-creating command. This follows #275 and #430, which changed the swapped run at every format: `caller.rs` has no format gate. A scalar creating-only model keeps its bytes (`tests/caller_fresh_identity.rs:495-507` pin). Measure with before/after synthesis of every repository model.
7. Alternatives:
   - (a) Change nothing and tell targets to reuse an arranged record. That hides a real duplicate-arrangement defect, and the swapped run then proves less.
   - (b) Drop the swapped run where an external outcome is forced on a non-creating command (the requester's second option). This loses coverage broadly (Q4).
   - (c) Chosen: the requester's first option, scoped to identity inputs: inputs an outcome names as its supplied instance, not every input.

## Decisions
accept, redesigned. The caller-swapped run draws a fresh value for every literal identity it sends, whether the command creates or addresses it. "Identity input" means an input a branch names as its supplied instance. Each value keeps its step's branch, follows every copy, and stays member-fresh for a struct. Exhaustion keeps today's singleton route or note. The swapped run is not dropped. No format bump: neither ess/23 nor ess-conformance/44-45 is touched. Suite bytes change at every source format, decided here as a synthesis defect fix, as #275 and #430 did.

## Acceptance
- swapped_run_redraws_addressed_identity: probe `a`'s `CommitTransaction/outcome/invalid-reference` and `/wrong-state` send different `transaction` literals in the two runs, and the interpreter passes all scenarios.
- arranging_target_passes_both_runs: a fixture target that arranges a prepared batch under the sent identity on `configure_external_outcome` and refuses a second arrangement of one identity passes all three external refusals. It errors on the 0.53.0 suite.
- addressed_identity_follows_its_copies: the swapped run's event payload and view expectations that copy the addressed identity carry the fresh value.
- addressed_struct_identity_member_fresh: a non-creating command addressed by a struct identity gets a swapped value fresh in every member (#430 rule).
- created_then_addressed_stays_consistent: in `committed`, the swapped run's `PrepareBatch` and `CommitTransaction` name the same fresh row.
- singleton_addressed_identity_keeps_one_row_route: a one-value identity sent only to a non-creating command takes `one_row_acted_on` or the `UnswappedCallers` note, never a duplicate send.
- scalar_identity_bytes_unchanged: `tests/caller_fresh_identity.rs` keeps its pinned digest.
- repository_model_suites_change_only_swapped_literals: before/after synthesis of every repository model differs only in swapped-run literals of non-creating identity inputs and their copies.

## Scope
- crates/verify/ess-conformance/src/synthesize/caller.rs  cited — `Identities::of` :1089-1144, `sent` :1146-1177, `drawn`/`draw` :1191-1295, `one_row_acted_on` :835-876
- crates/verify/ess-conformance/src/synthesize/existence.rs  cited — `identity_input` :51-74 read, not changed (a sibling helper for supplied instances is inferred to live in `caller.rs`)
- crates/verify/ess-conformance/tests/caller_fresh_identity.rs  cited — byte pin :495-507 must stay green
- crates/verify/ess-conformance/tests/caller_addressed_identity.rs  inferred — new cases and the arranging fixture target
- crates/verify/ess-conformance/tests/fixtures/caller-addressed-identity.yaml  inferred — brand-free fixture (probe `a` shape)
- docs/design/caller-values.md  cited — :138 gains the sentence on literal identities
