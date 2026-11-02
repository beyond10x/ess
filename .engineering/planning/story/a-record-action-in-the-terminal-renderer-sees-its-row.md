---
format: aep.planning-md/3
id: story:a-record-action-in-the-terminal-renderer-sees-its-row
kind: story
status: active
title: A record action in the terminal renderer sees its row
summary: 'ess-ui-tui leaves row out of scope: a row.<key> bind sends null and a row-guarded action does not act; React differs (0.77.0)'
tags:
- ess-batch-4
- ess-next
- ess-ui
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/record_action.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:53Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:53Z", actor: "human:timo", revision: 10}
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

- Over a toy `ui-spec` whose record action binds `row.<key>` and is visible on `row.state == <S>`, an
  `ess ui test` script run against the terminal renderer sends the row's key, and `act` on a
  visible action sends its command.
- The same script gives the same commands against the React target. One test runs both renderers
  over the same document and compares the commands they send.

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

In the terminal renderer (`crates/ui/ess-ui-tui`, behind `ess ui test`), a record action cannot see
its row:

- With `bind: {<key>: row.<key>}` the action sends null for that input.
- With `visible: "row.state == <S>"` the key hint shows the action, but `act` does nothing.

The React target puts the row in scope for the same action, so the two renderers disagree on one
`ui-spec` document.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/ui/ess-ui-tui`, the record composite's key handling — cited.
- **Files:** `crates/ui/ess-ui-tui/src/app.rs:2557-2582`: the `Composite::Record` key arm builds `Ctx` without `row` and calls `run_action_at(&ui, &action, None)` at 2581 — cited.
- **Symbols:** `record_actions` (view.rs:2096, callers view.rs:865 and app.rs:2575); `Scope::resolve` `"row"` arm (app.rs:3848) returns `Null` when `ctx.row` is unset — cited.
- **Also likely:** `crates/ui/ess-ui-tui/src/view.rs:806-812` (the record draw computes the row the hint uses; a shared helper would move there); `crates/ui/ess-ui-test/tests/record_action.rs` (new, the acceptance script and React parity) — inferred.
- **Documents:** none; a `CHANGELOG.md` line — inferred.
- **Confidence:** high — the story names the surface and the tree shows both symptoms on one path (the key arm drops the row, the draw arm keeps it); read, not run.
- **Would collide with:** any unit editing the `ess-ui-tui` record key arm (app.rs:2557-2582), `record_reads` (app.rs:1302-1328), the `row` scope (app.rs:3848) or record drawing (view.rs:806-866) — cited.
- **Safety fact:** `record_actions` has two callers (view.rs:865, app.rs:2575) — cited.
- **Not established:** the React "row in scope" claim is from the story and `ess-ui-react/templates/runtime/composites/record.tsx.tmpl:20`, not run; a record inside a collection's `expand` was not traced — inferred.
- **Size:** medium, about 35 production lines and 120 test lines; no format change (`ess-ui/1`, `ess-ui-test/1`) — cited.
