---
format: aep.planning-md/3
id: decision-blocker:related-state-refusal-and-the-held-state-step
kind: decision-blocker
status: open
title: 'May a refusal that reads a related row''s lifecycle state answer before the held-state branches of its command: RS-D1'
summary: RS-D1 options A to D, an order change of the one precedence rule; architecture decision with the owner, no default
tags:
- architecture
- gap-reports-2026-11-24
relations:
- blocks: story:a-refusal-reading-a-related-rows-state-may-answer-before-held-state-branches
revision: 2
---
RS-D1: may a refusal that reads a related row's lifecycle state answer before the held-state branches of its command? It changes the order of the one precedence rule (`docs/design/generic-language-features.md` "The rule": request alone, existence, held state, rows around the record, acceptance, default) and the formats, so it has no default: the owner decides.

## The question

The one rule answers what the addressed row holds (step 3) before the rows around it (step 4). A command whose ordinary effect is `when_subject_state:`-selected, and which must do nothing while a related row rests in a state, cannot declare the refusal above it: `ESS-COMMAND-004`. Evidence and reproduction are in `story:a-refusal-reading-a-related-rows-state-may-answer-before-held-state-branches`. Where the ordinary effect is a plain accepting branch, the shape already validates and its suite passes.

## RS-D1 options

- A. Keep the order. `ESS-COMMAND-004`'s help and the `when_related` reference page name the idiom: drop `when_subject_state:` from the ordinary branch, keep `wrong_state:`, declare the related-state refusal above it. Cost: no format, no surface; a help text and a page. Does not serve a command with several held-state ordinary branches. The sibling case (related row ahead of input guards) was answered this way.
- B. A marker on a related-row refusal that places it before the held-state step, as `before_row: true` places an external refusal at step 1 (`cross-record-and-stored-field-guards.md`, the `before_row` section). Cost: a new marker, a step in the precedence table, one new step-pair refusal in validation, synthesis, the interpreter, generated Rust and Go, Entity Runtime lowering, docs, diff classification, a source format version.
- C. A declared tie: an input is true exactly when the related row is in a state, checked in scenarios. Cost: a new kind of declaration (a caller-reported fact tied to a state the model owns); the check is weaker than a read of the row (policy 1 of `generic-language-features.md`).
- D. One branch may carry a held-state guard and a related-row guard together (`when_subject_state:` with `when_related:`), answering at the later of the two steps. Cost: two guard kinds on one branch, today `conflicting_declaration`; a rule for which step answers.

No default. A is the only option that adds no surface; B is the one that states the fact exactly for every ordinary shape.

## Not decided here

The spelling of any marker, and whether the accepting branch that changes nothing (`story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view`) is the reporter's "changes nothing".

## Decisions

- RS-D1: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET (he is away); reversible until a release carries it: keep the order of the one precedence rule; the help of `ESS-COMMAND-004` and the `when_related` reference page name the idiom (the short draft `story:the-help-of-ess-command-004-and-the-when-related-page-name-the-idiom-for-a-refusal-above-held-state-branches`).
