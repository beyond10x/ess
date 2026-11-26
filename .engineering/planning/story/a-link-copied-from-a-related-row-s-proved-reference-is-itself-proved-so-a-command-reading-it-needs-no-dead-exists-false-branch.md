---
format: aep.planning-md/3
id: story:a-link-copied-from-a-related-row-s-proved-reference-is-itself-proved-so-a-command-reading-it-needs-no-dead-exists-false-branch
kind: story
status: draft
title: A link copied from a related row's field that is itself a proved reference is proved too, so a command reading it needs no exists-false branch that can never answer
summary: 'ESS-COMMAND-005 asks for a dead exists: false branch where a writer copies the link from another row''s field; the ess/30 derived proof covers only a link set from a refused input'
tags:
- defect
- issues-2026-11-26
relations:
- serves: vision:O2
- informed_by: story:enforced-reference-needs-no-missing-target-branch
revision: 1
---
## Need

A downstream application reported on 0.99.4 (received 2026-11-25) a command whose accepting branches read the row the subject links, behind a first branch that refuses when the subject's link differs from an input. The author could not remove the last branch, a dead `exists: false` branch that can never answer: `ESS-COMMAND-005` asks for it.

Two rules meet here and both are right as written:

- `ESS-COMMAND-004` is `docs/design/generic-language-features.md` § G4 "The rule": the subject guards of step 3 answer before the related-row existence of step 4, and the declaration order must agree.
- `ESS-COMMAND-005` (`crates/specify/ess-domain/src/command/related_guard.rs`): a related read named by an input always needs an `exists: false` branch. The lift is `crates/specify/ess-domain/src/command/reference_integrity.rs`, § G2 "Enforced references" (`ess/30`). It covers only a read through the subject's stored field whose every writer sets that field from an input that an `exists: false` branch refuses first. A link a writer copies from another row's field defeats it (`checked()` answers false for that source).

The gap is the scope of the `ess/30` proof, not either rule.

## Reproduction

Five generic reductions, each on the 0.99.5 binary:

1. The branch moved above the subject guard: `ESS-COMMAND-004` ("answers at step 3 ... the earlier branch at step 4").
2. The dead branch removed: `ESS-COMMAND-005`.
3. The read through the stored link at format `ess/29`: `ESS-COMMAND-005`, whose hint names `ess/30`.
4. The same at `ess/33`, where the writer checks its input first: it validates without the branch.
5. The writer copies the link from another row's field: `ESS-COMMAND-005`, "the row can be missing because the writer writes the link without first refusing an identity no row carries".

## Outcome

From `ess/38` on, the derived proof of § G2 covers a link that a writer copies from a related row's field when that field is itself a proved reference to the same entity and that row is refused as missing first. A command reading the row through such a link validates without the dead `exists: false` branch. `ESS-COMMAND-004` stays as it is. No key is added; the change accepts more documents, and only from `ess/38` on.

## Acceptance

- At `ess/38`, a reduction of the reported shape (a writer copying the link from a related row whose field is a proved reference) validates without the `exists: false` branch; the same document at `ess/37` is refused with `ESS-COMMAND-005` as before.
- A link copied from a row whose field is not a proved reference is still refused with `ESS-COMMAND-005`.
- Moving the branch above the subject guard is still refused with `ESS-COMMAND-004`.
- Synthesis of the accepted reduction writes a scenario for each accepting branch; no scenario is refused for a branch that can never answer. The synthesis half is `story:a-synthesized-scenario-whose-setup-its-own-model-refuses-is-written-with-the-rows-its-guards-relate-or-refused-by-synthesis` (group 1).
- § G2 "Enforced references" of `docs/design/generic-language-features.md` states the copied-link condition, and a test pins its first condition's new wording.

## Fit review

1. Need: an author reads a refusal as a request to write a branch that no input can reach; that is a defect of the proof's scope.
2. Class: a rule's scope, found by a downstream report; no new key.
3. Already expressible: the stored-link form with an `exists: false` branch validates, and synthesis refuses that branch as unreachable (option C below).
4. Fit: it extends the derivation of `ess/30` by one source form and keeps both rules.
5. A second adopter: any model that copies a reference from one row into another and reads the target through it.
6. Cost: a wider acceptance behind a format step, so documents below `ess/38` behave as before.
7. Considered: B, lift `ESS-COMMAND-005` for an input-named read where a step-3 guard `<link> != input.<field>` answers first and the link is proved (a second notion of proof); C, keep both rules and document the stored-link form (leaves the dead branch in the author's model).

## Decisions

- D152 = A (the coordinator, 2026-11-26): from `ess/38` on, the derived proof covers a link copied from a related row's field that is itself a proved reference; `ESS-COMMAND-004` stays; no new key. Options were B (drop 005 behind an equality guard) and C (document the stored-link form).

## Delivery

Step 8 of the branch `issues/open-2026-11-26`, in progress when this was written.

## Scope

Cited: `crates/specify/ess-domain/src/command/reference_integrity.rs` (the proof), `crates/specify/ess-domain/src/command/related_guard.rs` (`ESS-COMMAND-005`), `docs/design/generic-language-features.md` § G2 and § G4. The synthesis half is not in this story.
