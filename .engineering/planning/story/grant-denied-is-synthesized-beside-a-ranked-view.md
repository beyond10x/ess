---
format: aep.planning-md/3
id: story:grant-denied-is-synthesized-beside-a-ranked-view
kind: story
status: draft
title: grant/denied is synthesized beside a ranked view
summary: 'Synthesize withholds grant/denied for a command on an existing row when a view of its entity has order_by: the grant template reads the scenario''s last send (#712)'
tags:
- ess-batch-5
- ess-next
refs:
- provider: github
  reference: beyond10x/ess#712
revision: 4
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

- Over both toy specs of #712, synthesize writes the moving command's `grant/denied` scenario, with or
  without `order_by`. The scenario passes the interpreted target and fails a target that skips the
  grant check.
- The grant template is the last send of the command under audit. Later sends that a view check
  appends are dropped from it, as `denied_from` already does.
- Suites that already had their `grant/denied` scenarios keep their bytes. The bytes-table rows that
  gain a scenario are named in the unit and re-pinned on purpose.

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

beyond10x/ess#712, a defect found 2026-10-30 while reproducing #710, on ESS 0.77.0. On a toy spec with
a creating command, a command that moves an existing row, and an actor granted nothing:

- With a view of the entity ranked by `order_by`, synthesize writes 6 scenarios. It notes that no
  scenario sends the moving command into a branch that accepts it, so its `grant/denied` is not
  synthesized, although its `outcome/...` scenario does exactly that.
- Without the `order_by` line, it writes 7 scenarios, `grant/denied` included.

Cause: `refused_from_template` in `synthesize/grant.rs` takes the scenario's last send of any
command. The `ranked` view check appends a second creating send after the command's own, so the
template is rejected and `denied` and `denied_since` push `GrantDeniedUnwitnessed`. This is not the
deliberate baseline drop of #710.
