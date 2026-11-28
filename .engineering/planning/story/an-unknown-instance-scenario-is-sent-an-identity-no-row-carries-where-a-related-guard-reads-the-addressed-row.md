---
format: aep.planning-md/3
id: story:an-unknown-instance-scenario-is-sent-an-identity-no-row-carries-where-a-related-guard-reads-the-addressed-row
kind: story
status: draft
title: An unknown_instance scenario is sent an identity no row carries where a related guard reads the addressed row
tags:
- defect
- issues-2026-11-27
relations:
- serves: vision:O2
- informed_by: story:a-not-found-branch-over-the-addressed-row-beside-a-row-set-is-refused-by-synthesis-and-not-sent-the-row-it-arranged
- informed_by: story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent
revision: 1
---
## Need

A downstream control plane reported on 0.99.7 (2026-11-26) a conformance suite that failed against
the interpreted target on a scenario synthesis wrote, and `ess verify conform mutate` on the same
suite refused with `ESS-MUTATE-001` because the unmutated suite did not pass.

Reproduction, generic: an `Item` with an `owner_id`, and a command `Claim` that moves an item
(`moves:` with `instance: item_id`), declares `unknown_instance:` and has a `when_related:` guard
over the row its own input addresses,
`when_related: {via: input.item_id, predicate: owner_id != input.owner_id}`. Synthesis wrote the
`unknown_instance:` scenario (`Claim/outcome/no-such-item`) by sending the row it had arranged for
the predicate refusal, so the send named an existing row and the command answered the predicate
refusal, not the unknown-instance answer. Observed on 0.99.7 with `ess verify conform run --target
interpreted` (the scenario fails) and `ess verify conform mutate` (`ESS-MUTATE-001`).

The bug is old: every release from 0.63.0 to 0.99.7 carries it. It is a sibling of
`story:a-not-found-branch-over-the-addressed-row-beside-a-row-set-is-refused-by-synthesis-and-not-sent-the-row-it-arranged`
(the row-set path) and of `story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent`.

A second shape of it, found in the same work: a command that also reads a row of another entity
through its input (`when_related: {via: input.owner_id, predicate: active == false}`) got a
related-guard branch whose setup arranged the addressed row twice, once as the subject and once as
the related row, and sent the related one, so a refusal over the other row was answered by the
predicate over the addressed row and the scenario failed on the interpreter.

## Outcome

The `unknown_instance:` scenario of such a command is sent an identity no row carries, and the
second send naming a missing related row is left out where that row is the addressed record, here
and in the held-state refusal's scenario. A branch arranged around the other row below a predicate
over the addressed row is refused `ESS-SYNTH-008`, as it is where the command reads the addressed
row alone; a branch arranged around the addressed row, or with no such predicate declared before
it, keeps its scenario. A suite never holds a scenario its own model answers differently.

## Acceptance

- The red test of the regression (`01b0c4b277`) fails on 0.99.7 and passes with the fix
  (`400385903a`).
- `ess verify conform mutate` on the reduced model no longer refuses `ESS-MUTATE-001`.
- The variants an adversary pass added (`6baf3977f8`, one red) pass; the three repository-wide byte
  tables pin the new fixture and only its rows and the header counts move (`93da79b08d`); the
  several-row search refuses a command reading the addressed row beside another row
  (`47e08e38dc`).
- No committed suite in the repository moves.

## Fit review

1. Need: a suite written by synthesis must pass the model it was written from. Reproduction above.
2. Class: defect. Synthesis writes a scenario the model's own interpreter answers differently; the
   related-guard sibling path already refuses the same shape.
3. Already expressible: no author action avoids it except dropping the `unknown_instance:` branch.
4. Fit: the check the sibling paths have, here applied to the `unknown_instance:` scenario. The
   code is the existing `ESS-SYNTH-008`.
5. Second adopter: any command that moves or updates a row and also guards on a predicate over it.
6. Cost: a scenario the suite no longer holds where it failed before; no format key, no new
   diagnostic. A dependency change in the same release is the `nix` crate for the signal fix of
   `story:the-sigint-and-sigterm-during-the-handshake-test-decides-its-signal-without-a-race`, which
   made the release take the full gate.
7. Considered: change nothing; arrange the scenario with an identity no row carries (chosen for
   `unknown_instance:`); refuse the branch (chosen for the multi-row variant, with its follow-up in
   `story:a-branch-arranged-around-the-other-row-below-a-predicate-over-the-addressed-row-is-written-as-a-scenario-its-model-accepts-not-refused-ess-synth-008`).

## Decisions

- None open. Taken as a defect.
- D163 = A (the operator session, 2026-11-26 19:36 CET): the push run of `main` (45868681122) is
  cancelled to free runner capacity; the pull-request runs stay.
- D164 = A (the operator session, 2026-11-26 19:53 CET): the fix ships as the hotfix 0.99.8 now,
  from the tag 0.99.7 by the Hotfix rule of `AGENTS.md`, and does not wait for the wave.

## Delivery

0.99.8 (branch `hotfix/0.99.8` from the tag 0.99.7): red test `01b0c4b277`, fix `400385903a`,
release preparation `80d2a73cf1`, adversary variants `6baf3977f8`, byte tables `93da79b08d`,
several-row refusal `47e08e38dc`. Pull request 843 (https://github.com/beyond10x/ess/pull/843)
merged into `main` on 2026-11-27 09:23 CET as `e29c029fab`; the tag 0.99.8 is on `e29c029fab`
(2026-11-27 09:26 CET). Release run 45965734005 pending when this was written; the release is not
published yet.

## Left open

Validation still requires an `exists: false` branch over the addressed row, which `unknown_instance:`
answers first; lifting that would admit documents older releases refuse, so 0.99.8 does not change it.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/src/synthesize/row_set.rs
- inferred: crates/verify/ess-conformance/tests/related_guard.rs
- inferred: CHANGELOG.md
