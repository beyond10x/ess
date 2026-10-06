---
format: aep.planning-md/3
id: story:feature-request-228
kind: story
status: implemented
title: No way to declare a multi-field key unique within a scope (equality, one arranged row)
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#228
relations:
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T15:27:22Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-04T15:27:22Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-06T09:42:55Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

No way to declare a multi-field key unique within a scope (equality, one arranged row) (beyond10x/ess#228).

## Status

Feature request, triaged 2026-09-29 as outside the 0.41 and 0.42 defect batches. Not scheduled; acceptance is written when it is.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: feature-request-228 (multi-field unique within a scope)

Tested with `ess 0.44.0`; rules re-read in the 0.48.0 read tree (`gaps-270`, `e3bc9a2ff`).

1. **Need.** A command creating (or updating) a record is refused when another record of the same entity already carries the same values in several fields within a scope: inside one tenant, no two users bound to the same `{sub, org_id}`. The record's identity is a different field (`user_id`), and `existing_instance:` already answers a second bind of that.
   - *Requester's syntax:* entity-level `unique: [{fields: [sub, org_id], per: tenant_id, error: ...}]`. The comment's variant is `unique: [account_id, sub, org_id]`.
2. **Class: gap.** A partial idiom exists. `repro-228/a-claim-entity.yaml` makes the claim triple a struct-typed identity of its own entity, with `existing_instance:`. It validates and synthesizes 2 scenarios, 0 refusals, including "sent twice for one identity takes `taken`" (0.44.0, `a.suite.json`). But then a record has only one key, so the user's own `user_id` collision is lost. Binding both needs one command creating two records, which is `story:related-record-effects` (#229 second half, draft).
3. **Already expressible?** Only via that idiom.
   - `when_related` looks up by identity only (`cross-record-and-stored-field-guards.md:517-520`).
   - Rule 2 keeps set constraints out of scope (`:441-496`).
4. **Fit of the requester's design: fails as written.**
   - An entity-level `unique:` is an invariant over pairs of rows. `cross-record...md:456-458` rejects that shape because it "names no command": every creating or updating command would gain an implicit refusal that its outcomes do not list. Here it carries `error:`, but which commands answer it would still be implicit.
   - `per:` is new vocabulary for a scope that a filter conjunct already says.
   - Redesigned as family F **B** on the command:
     ```yaml
     when_related: {entity: demo.binding.Identity, where: {all: [tenant_id == input.tenant_id, sub == input.sub, org_id == input.org_id]}, exists: true}
     ```
     with an `error:`.
   - It composes with `existing_instance:` (precedence step 1, `cross-record...md:611`). It is the same witness the requester describes: one arranged row with the values, then a second create. Decoys (one conjunct refuted each) come from set-effect arrangement (`set-effects-over-filtered-instances.md:55-70`).
   - An entity-level `unique:` that only checks every writer of those fields declares such a refusal is a possible later lint, not a semantic.
5. **Second adopter.** Already one, in the issue comment (`account_id, sub, org_id`, 409). Others: one booking per `(room, slot)` per venue; one active API key name per project.
6. **Cost.**
   - Shared with #237 (2): new `when_related` keys under the ess/20 bundle and an `ess-diff` classification.
   - No suite format.
   - Entity Runtime keeps `RelatedGuardUnsupported`; the interpreter reports `unsupported`.
   - Concurrency (two racing creates) stays unmodelled, as for every selection-time read (`cross-record...md:469-473`). The design note says so.
7. **Considered.**
   - (a) Change nothing: the struct-identity idiom plus `story:related-record-effects`. Partial, as Q2 shows.
   - (b) The requester's entity-level `unique:`. Rejected (Q4).
   - (c) A B row-set test on each command. Chosen: one spelling with #237 and the #229 follow-up, explicit per command, witnessable with one arranged row.

## Decisions

- **accept, redesigned (proposed):**
  - Uniqueness within a scope is stated per command as family F part B, `when_related: {entity, where: <all-equal conjuncts over input.>, exists: true}` with an `error:`. It is witnessed by one arranged matching row plus one decoy per refuted conjunct.
  - The requester's entity-level `unique:`/`per:` is dropped. It is a pair invariant that names no answering command, which `cross-record-and-stored-field-guards.md:456-458` already rejects. An entity-level lint can be a labelled later milestone.
  - Partial idiom today: a struct-typed identity plus `existing_instance:` (`repro-228/a-claim-entity.yaml`).
  - Overlaps: #237 (2) (same construct); `story:related-record-effects` (#229) for the two-record idiom; #75 rule 2 (out of scope, unchanged).
  - Not already fixed (tree `e3bc9a2ff`).

- Coordinator (2026-10-01): adopted as family F below. Format: ess/21, because ess/20 ships alone in 0.49.0 and family F lands as one bump. Family F (one design across #225, #228, #233, #237, #244): A1 a bare right-hand root names a field, input or binder; A2 one `± constant` offset (Integer or Timestamp); A3 `now` in `when_subject`/`when_related`; A4 `input.<dotted path>` in values; B `when_related: {entity, where, exists | count | forall}` over row sets; C `distinct` over lists and `.utf8_bytes`.

## Current design disposition

Second independent review approved the complete shared row-set/filtered-value contract at 8606b103c (review-result:filtered-related-read-design-20261003-r2). All seven first-round findings were fixed. Design review is complete; implementation and target acceptance remain pending. This supersedes the earlier pending-design-review wording, not the predecessor gates or draft implementation status. The shared normative page is docs/design/filtered-related-reads.md and the syntax allocation is source22. No source21 family allocation remains current. #299 depends on #285 and #228/#237; the shared page also binds the latter stories' row-set implementation.
