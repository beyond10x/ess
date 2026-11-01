---
format: aep.planning-md/3
id: story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field
kind: story
status: draft
title: A parent tenant guard holds across every related row and an absent field
summary: 'U13 adversary: a second related row guarded on tenant still starves found; an absent Optional tenant splits synthesis and interpreter'
tags:
- ess-next
relations:
- informed_by: story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children
revision: 3
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

- Case 1: `found` is witnessed, with every related row the child's creator guards on the input
  created under that input, and the interpreted target passes it. Without the parent guard, no
  scenario that the interpreted target fails is written.
- Case 2: synthesis and the interpreter decide an absent Optional field compared with an input in
  the same way. The main witness either passes or is not written, and the decision is pinned by a
  test on each side.
- The batch-3 adversary's red cases for both shapes are restored as tests.

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

Found by the batch-3 adversary pass over the parent-row guard (U13). Both cases also happen on
0.77.0.

1. The child's creator guards a second related row on the same input as the parent: it reads a
   folder, and refuses one of another tenant. `GetChild/found` is then still refused with the
   guarded-creator ESS-SYNTH-001, because the U13 fallback binds only the parent link
   (`crates/verify/ess-conformance/src/synthesize/row_set.rs`, `prepare`, the held-under attempt).
   Without the parent guard, the same model gets `found` and `child-not-found` scenarios that the
   interpreted target fails, because the child is created under a folder of another tenant.
2. With Optional tenants compared plainly (`tenant != input.tenant`), the deleter's main `deleted`
   witness sends a tenant to a parent created without one. The interpreted target decides the
   guard Unknown and reports the scenario Unsupported. Synthesis treats the absent field as not
   taken, so synthesis and the interpreter disagree.
