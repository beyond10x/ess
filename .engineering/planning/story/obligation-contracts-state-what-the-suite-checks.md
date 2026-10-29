---
format: aep.planning-md/3
id: story:obligation-contracts-state-what-the-suite-checks
kind: story
status: draft
title: Obligation contracts state what the suite checks
summary: Generated Rust obligation contracts omit view order_by, misprint an empty-string guard, omit refusal precedence and the ASCII-only trimmed rule (0.77.0)
tags:
- ess-batch-7
- ess-next
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

- Over a toy model, the view-query obligation in `PLAN.md`, `plan.json` and the trait doc states
  the view's `order_by` keys and directions exactly as the IR holds them.
- An `any_of: [""]` guard under `trimmed` is printed as the IR means it (empty after trimming). The
  contract states the precedence the suite pins: input-guarded refusals answer before not-found and
  before wrong-state.
- Recommended: the contract states that `trimmed` strips ASCII whitespace only. Synthesized
  scenarios then avoid inputs on which ASCII-only and Unicode trimming disagree, or else name the
  rule they rely on. An authored scenario may state the no-break-space case. Alternatively, the fit
  review decides to make `trimmed` Unicode-aware, with its D135 handling.
- One test per item compares the printed contract against what the suite checks, so the two cannot
  drift apart again.

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
reproduced in this repository. Nothing blocks, but each finding misleads whoever implements an
obligation from the contract that `ess generate synthesize --target rust` writes.

1. A view-query obligation's contract drops the view's `order_by`. The IR holds
   `order_by: ["<field> asc", "<id> asc"]`, but `PLAN.md`, `plan.json` and the generated trait's
   doc state only the filter, while the suite checks the order.
2. `PLAN.md` prints an empty-string guard as `trimmed(<field> in [])` where the IR holds
   `any_of: [""]`. As printed, the guard reads as one that never holds. The contract also leaves out
   that input-guarded refusals answer before not-found and wrong-state, though the suite pins that
   order.
3. `trimmed` strips ASCII whitespace only, and the contract does not say so. A synthesized outcome
   scenario sends `"  "`-like input with a non-ASCII space and expects acceptance, while Rust's
   `str::trim` treats it as empty. An authored scenario for the no-break-space case is refused with
   ESS-AUTHOR-041.
