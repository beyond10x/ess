---
format: aep.planning-md/3
id: story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent
kind: story
status: implemented
title: An identity guard below unknown_instance is decided on the row sent
summary: 'Follow-up of U11 (#645, #693): a branch''s own guard on a subject-addressed identity, and a related row three moves away, still get the wrong row or a wrong refusal'
tags:
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#645
- provider: github
  reference: beyond10x/ess#693
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/repeated_identity.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_u11_batch3_pass1.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T06:49:19Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-01T06:49:19Z", actor: "human:timo", revision: 9}
- {from: "active", to: "implemented", at: "2026-11-01T10:38:00Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
<!-- Starting point for a `story` artifact, seeded by `aep artifact new story <name>`.
     No frontmatter here on purpose: the `---` block is written by the CLI from the id, kind, status
     and relations you gave it, and a second copy in this file would be the one that went stale.
     Delete the italic guidance as you fill each section. -->

# Story: <name>

## Outcome

*What is true for whom once this has shipped, in one sentence. If it names a component rather than a
person, it is a task — say what changes for someone.*

## Context

*Why this is worth doing now, and what it depends on. Link the epic or specification it comes from
rather than restating it; the `derived_from` relation already carries the edge.*

## Acceptance

- A branch's own `when:` guard on the identity input of a subject-addressed row is decided on the
  identity of the row arranged. The creator is steered onto that identity, as `on_identity` does for
  #693. Where no row can meet the guard, the branch is refused with ESS-SYNTH-003, naming the guard.
- With a String identity guard and a related row three moves away, the guarded branch is written and
  passes on the interpreted target, or it is refused with a reason that names the guard. It is never
  refused as a repeated identity when a row of another identity witnesses it.
- Each red case of the batch-3 U11 adversary pass either passes on the interpreted target or is
  refused as above. No row of the synthesis bytes tables moves.

## Out of Scope

*What a reasonable reader would expect to be included and is not — the boundary that stops this
story quietly becoming an epic.*

## Ambiguities

*Each gap this story found and did not close, classified. `inferable` — the answer is already
written down, so give the `path:line` or the artifact id that settles it.
`requires-stakeholder-input` — nobody here can decide it, so name who does, and raise that entry as
a `decision-blocker` with a `blocks` edge to this story, or it is a sentence somebody improvises
later.*

## Open Questions

*Anything still undecided belongs in `## Ambiguities` above, classified and with its citation or its
decider. Keep this section for a question that is neither — an unowned question is a story that
stalls without anybody noticing.*

## Problem

The batch-3 adversary pass over U11 (#645, #693) found shapes still outside it. The models are kept with
the red cases of that pass.

- An accepting branch below `unknown_instance:` whose own `when:` reads the identity input (for example
  `note_id == "keep"`, or an enum `note_id == b`) is sent a stored row of another identity. The
  interpreted target answers the wrong branch. It is the same on 0.77.0.
- Below an accepting guard `note_id != "keep"`, the deleting branch is sent a note opened as
  `"note_id"`, so the guarded branch answers. This is the #693 bug class on a subject-addressed row.
  It is the same on 0.77.0.
- With a String identity guard and a related row three moves away, the guarded branch is refused
  ESS-SYNTH-001 ("creates a second row under a value a row it created still holds"). A row of another
  identity would witness it, as the same model shows with the guard on a different input. On 0.77.0
  a failing scenario was written instead. This is introduced by U11, but it moves a wrong scenario to
  a refusal.

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D343 on 45a1beba7. Every line is cited or inferred.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize.rs` — cited. `arranged` (:5006), `stored_where_unknown_first` (:5415), `addressed_held` (:5439), `held_row_in` (:5497), `arranged_plain` (:5645). The first four are U11's #645 arrangement; they call `arrange_first(.., &[])` with no hint, so the row's creator is not steered by the branch's own `when:`. `held_row_in` sets `bound[identity field]` to the captured row (:5524), so the send repeats whatever identity the creator used.
- **Symbols to reuse:** `on_identity` (`synthesize/related_guard.rs:4543`, private), `created_identity` (:4576), `on_row` (:4529), `reading_only` (:4562); `subject_fact::{hinted:3853, identity_read:4610, search_under:5400}`; `existence::unknown_answers_before` (`existence.rs:48`) — cited.
- **Also likely:** `synthesize/related_guard.rs` (visibility of `on_identity`, or a sibling reading the branch's own `when:`) — inferred.
- **Also likely (acceptance item 2):** `synthesize/repeated_identity.rs:28` `CREATED_TWICE`, `:35` `withdraw_repeated_identities` raise the ESS-SYNTH-001; the fix is probably upstream in identity profiling — inferred.
- **Tests:** the batch-3 U11 red shapes were never committed (4902a0b3e holds 18 green cases); the unit writes them red-first in `crates/verify/ess-conformance/tests/adversary_u11_batch3_pass1.rs` (models `NOTES`, `SLOTS`, `moved_slots` :499). Regression: `accepting_branch_below_unknown_instance`, `related_guard_on_the_identity_input`. Bytes: `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read` stay unchanged (acceptance) — cited.
- **Confidence:** medium. Where item 2's ESS-SYNTH-001 arises needs a run.
- **Would collide with:** a unit editing `synthesize.rs` arrangement functions or `related_guard.rs` `on_identity`/`with_row_where`.

## Decisions

D343, 2026-11-01 (coordinator): accepted as written for wave D343, unit U3. The batch-3 U11 red shapes
were never committed (4902a0b3e holds only green cases), so the unit writes them red-first in
`crates/verify/ess-conformance/tests/adversary_u11_batch3_pass1.rs` from the shapes in `## Problem`.
The synthesis bytes tables must not move (acceptance).

## Delivered

D343, 2026-11-01: unit ef2cc60f1 and correction 3db8d0d84 (merged 679449289, 0459d1eba). Scope check:
`related_guard.rs` gained `onto_identity` and `identity_parts`; the repeated-identity refusal's
cause was `related_boundaries` in `synthesize.rs` (confirmed by one run); steering goes through
`subject_fact::search_within`, not `arrange_first` hints (Scope wrong). Bytes tables unchanged.
Adversary pass 1: F1, F4 fixed (F4 pre-existing, ess 0.77.0 probe); F2, F3 became
`story:identity-guard-decided-on-the-row-sent-on-every-synthesis-path`.
