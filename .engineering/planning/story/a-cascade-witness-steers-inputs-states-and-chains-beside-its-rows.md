---
format: aep.planning-md/3
id: story:a-cascade-witness-steers-inputs-states-and-chains-beside-its-rows
kind: story
status: archived
title: A cascade witness steers per-input guards, copy-source states and relation chains beside its rows
summary: Five shapes the batch-3 adversary found still refused ESS-SYNTH-001 after the cascade witness; no wrong scenario
tags:
- ess-issues-3
relations:
- informed_by: story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed
revision: 5
transitions:
- {from: "draft", to: "archived", at: "2026-10-31T23:45:01Z", actor: "human:timo", revision: 5}
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

- Each of shapes 1-4 gets the cascade outcome's scenario. The scenario passes the interpreted target
  and fails a target that skips the move. Each shape's adversary case from the batch-3 pass is
  restored as a test in `crates/verify/ess-conformance/tests/related_record_effects.rs`.
- Shape 5 either gets its scenario, with the rows arranged before the subject leaves the state their
  creator needs, or keeps ESS-SYNTH-001 with a cause that names the ordering rather than the guard.
- No row of the synthesis bytes tables moves.

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

The batch-3 adversary pass over the cascade witness (U14) found five shapes that synthesis still refuses
as ESS-SYNTH-001. In each one, every scenario the new last-try path does produce passes the
interpreted target, so these are refusals that remain, not wrong scenarios. The six red cases are
kept in a repro outside the repository.

1. A creator that reads a second related row and also compares the subject with the input
   (`limit < input.cost`) is refused. Without the second-row guard it is witnessed. The beside pass
   passes no per-input steering (`crates/verify/ess-conformance/src/synthesize.rs`, the
   `CreatorPass::Beside` call of `arrange_toward_with`).
2. A copy source row whose creator is guarded by that row's own state is refused (a tier that is
   not `Published` is refused). `referenced_held` accepts on `holds` only and takes the cheapest
   state; `present_beside` then rejects it, and no other state is tried
   (`synthesize/row_set/copied.rs`).
3. A second related row that is checked against the moving subject (`run_id != input.run_id`) is
   refused, because `beside_bound` arranges strictly "whatever the input"
   (`synthesize/related_guard.rs`).
4. A key copied through a chain of two relations is refused: `referenced_held` arranges through
   `arrange_toward_beside`, which never recurses into `steered_held`.
5. Judgement. When the subject must leave a state before the cascade (cancel only a Paused run) and
   the rows are created only in that state, the refusal's new "answered first ... by" text points
   at a correct guard. The real gap is ordering: the rows are not created before the subject
   leaves the state. This predates U14.

## Triage (D338)

Declined at the 2026-10-31 findings triage (orchestrator D338, operator approval relayed by host-monitor D53): no wrong scenario and no downstream model named; reopen when a model hits it.
