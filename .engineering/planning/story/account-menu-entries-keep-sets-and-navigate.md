---
format: aep.planning-md/3
id: story:account-menu-entries-keep-sets-and-navigate
kind: story
status: implemented
title: Account menu entries keep sets and navigate
summary: account_menu entries silently drop sets and navigate, so the documented theme switch does nothing and ess ui check is silent (0.77.0)
tags:
- ess-batch-5
- ess-next
- ess-ui
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/lib.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/tests/checks.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/shell.tsx.tmpl
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:48:52Z", actor: "human:timo", revision: 12}
- {from: "proposed", to: "active", at: "2026-11-02T20:48:52Z", actor: "human:timo", revision: 13}
- {from: "active", to: "implemented", at: "2026-11-03T02:29:26Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
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

- An `account_menu` entry with `sets` changes the named state, and one with `navigate` changes the route, in the React renderer. A toy document covers the theme switch.
- `ess ui check` refuses any declared key that a renderer drops, with a named code, so no accepted key goes undrawn without a word.

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

Reported 2026-10-29 on ESS 0.77.0 by a downstream session, through the orchestrator. Not yet reproduced in this repository. ess-ui/1 accepts look-and-feel that the React renderer does not draw.

`account_menu` entries silently drop `sets` and `navigate`. The documented theme switch in the account menu does nothing, and `ess ui check` reports nothing.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/ui/ess-ui-react` (the React account menu) — cited.
- **Files:** `crates/ui/ess-ui-react/src/emit.rs:2457`: the `RegionKind::AccountMenu` branch copies only `name`, `label`, `opens`, `does` into each entry, so `sets` and `navigate` are dropped before the browser — cited.
- **Files:** `crates/ui/ess-ui-react/templates/runtime/shell.tsx.tmpl:128-170`: `MenuEntry` has no `sets` or `navigate`; the `AccountMenu` click handler runs only `opens` or `does`, in both `//+plain` and `//+bound` variants — cited.
- **Symbols:** `RegionKind::AccountMenu` (`crates/ui/ess-ui/src/model.rs:334`); `runAction` (`crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl:151`, not exported; applies `sets` and `navigate` at `:197`) — cited.
- **Also likely:** `templates/runtime/actions.tsx.tmpl` (export `runAction`); `crates/ui/ess-ui-check/src/model.rs:1172` (`region_names`, reads `does` only), `crates/ui/ess-ui-check/src/lib.rs:114` (`CHECKS`), `schemas/ui/ess-ui.schema.yaml:1645` (`checks.list`) for acceptance 2's named code; tests in `crates/ui/ess-ui-react/tests/` and `crates/ui/ess-ui-check/tests/checks.rs` — inferred.
- **Documents:** `website/docs/reference/ess-ui.md` is generated from the schema by `ess-ui-docs` (`crates/ui/ess-ui-docs/src/doc.rs:51`) and moves only if the schema changes — cited.
- **Confidence:** medium — the drop is read from source, not reproduced; the key list `ess ui check` would refuse against has no registry today (inferred); region `props` are opaque to the expander (`crates/ui/ess-ui/src/expand.rs:29`) — cited.
- **Would collide with:** `crates/ui/ess-ui-react/src/emit.rs` (region and action serializers, `:650`), `ess-ui-check`'s `CHECKS` list and the schema's `checks.list`; the theme switch is state-only until `story:light-and-dark-themes-render-differently` lands — inferred.
- **Safety fact:** the TUI parses `props.actions` on its own (`crates/ui/ess-ui-tui/src/app.rs:3439-3446`), so a React-only fix leaves it unchanged — cited.
- **Size:** medium, about 60 lines for the React path and 120 for the check path; no format change (`ess-ui/1`; region `props` stay `json`, `schemas/ui/ess-ui.schema.yaml:288`) — cited.

## Delivered

W1 (D394), 2026-11-02: account menu entries keep sets and navigate; ess-ui-check parses each entry as an Action, refuses sets beside does, navigate or opens, and the emitter refuses an entry it cannot parse. Unit c942ffcd1, with 097f53e06 and 9d2643a5a (review cases); merged cce755258 and 2324f9c1b.
