---
format: aep.planning-md/3
id: story:a-row-actions-confirm-is-addressed-by-the-views-key
kind: story
status: implemented
title: A row action's inline confirm is addressed by the view's key
summary: ess-ui-tui addresses row confirms by live.match or id and ignores reads.key, so a view without id cannot reach its open confirm (0.77.0)
tags:
- ess-batch-5
- ess-next
- ess-ui
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/confirm.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:54Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:54Z", actor: "human:timo", revision: 9}
- {from: "active", to: "implemented", at: "2026-11-03T02:29:28Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
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

- Over a toy view keyed by `reads.key` on a field other than `id`, with no `id` field, an `ess ui
  test` step addressing `rows/<key>/row_actions/<a>/confirm/overlay` finds the open confirm and can
  accept and dismiss it.
- A view that has an `id` field and also declares `reads.key` is addressed by `reads.key`. Views
  without `reads.key` keep their current addresses.

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

Reported 2026-10-29 on ESS 0.77.0 by a downstream session, through the orchestrator. Not yet
reproduced in this repository.

The terminal renderer addresses a row action's inline confirm by `live.match`, else by the row's
`id`, and ignores the view's `reads.key` (`row_scoped` in `crates/ui/ess-ui-tui/src/app.rs`). For a
view with no `id` field, the step address `.../rows/<key>/row_actions/<a>/confirm/overlay` reports
"is not open" while the dialog is on screen.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/ui/ess-ui-tui` — cited (the story names `row_scoped` in `crates/ui/ess-ui-tui/src/app.rs`).
- **Files:** `crates/ui/ess-ui-tui/src/app.rs:3311-3331` keys a row confirm by `section.live.match_field`, else `id`, never `reads.key`; its only caller is `app.rs:3479`; the confirm's `params` at `app.rs:3480-3484` carry only `id`, and its `does` command runs with them at `app.rs:3267` — cited.
- **Symbols:** `Section::row_key()` at `crates/ui/ess-ui/src/model.rs:810-815` prefers `reads.key` over `live.match`; `model.rs:780-785` copies `reads.key` into `live.match_field` only when a `live` block exists — cited.
- **Also likely:** `crates/ui/ess-ui-test/tests/confirm.rs` or `crates/ui/ess-ui-tui/tests/confirm.rs` for the regression test — inferred.
- **Documents:** none; a `CHANGELOG.md` line — inferred.
- **Confidence:** high — the runner (`crates/ui/ess-ui-test/src/runner.rs:597`) and the view (`crates/ui/ess-ui-tui/src/view.rs:1164`) key rows by `row_key()`, and `row_scoped` is the one site that does not; not reproduced, read from code.
- **Would collide with:** any unit editing `crates/ui/ess-ui-tui/src/app.rs` in the inline confirm path (`run_action` 3455-3500, `row_scoped` 3311), or the ess-ui-test confirm tests — cited.
- **Safety fact:** `drawn_at` is set only from `row_scoped` at `app.rs:3479` and `3494` and read only at `view.rs:654` — cited, unproven at runtime.
- **Not established:** whether accept on a key-less view carries the row key to the command's `bind`; shell-hosted sections keep the `id` key (`app.rs:3318`); the live-event path keys by `live.match_field` alone (`app.rs:1808`, `1975`, `4128`), outside this acceptance — inferred.
- **Size:** small, about 10 lines in `app.rs` and a 50-line test; no format change (`FORMAT = "ess-ui/1"`, `crates/ui/ess-ui/src/lib.rs:47`) — cited.

## Delivered

W1 (D394), 2026-11-02: a row action's inline confirm in the terminal renderer is addressed by the view's row key. Unit 642617588; merged cce755258 and 2324f9c1b.
