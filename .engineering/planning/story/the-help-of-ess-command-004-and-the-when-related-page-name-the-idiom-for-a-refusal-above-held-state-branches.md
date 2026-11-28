---
format: aep.planning-md/3
id: story:the-help-of-ess-command-004-and-the-when-related-page-name-the-idiom-for-a-refusal-above-held-state-branches
kind: story
status: draft
title: The help of ESS-COMMAND-004 and the when_related reference page name the idiom for a refusal above held-state branches
summary: 'A documentation change decided by RS-D1 A: drop when_subject_state: from the ordinary branch, keep wrong_state:, declare the related-state refusal above it'
tags:
- documentation
- ess-compiler
relations:
- serves: vision:O2
- informed_by: story:a-refusal-reading-a-related-rows-state-may-answer-before-held-state-branches
revision: 1
---
## Need

A command whose ordinary effect is selected by `when_subject_state:` and which must do nothing while a related row rests in a state
cannot declare that refusal above the branch: validation answers `ESS-COMMAND-004`. RS-D1 is decided A (keep the order of the one
precedence rule), taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET. A states the idiom instead: drop
`when_subject_state:` from the ordinary branch, keep `wrong_state:`, and declare the related-state refusal above it. The help of
`ESS-COMMAND-004` and the `when_related` reference page name that idiom; nothing names it today (a search of the store for
`ESS-COMMAND-004` finds no documentation story).

## Fit review

1. **Class.** A documentation change: no format, no surface, a help text and a page.
2. **Existing stories.** `story:a-refusal-reading-a-related-rows-state-may-answer-before-held-state-branches` is the case this answers and
   borders it; `decision-blocker:related-state-refusal-and-the-held-state-step` holds RS-D1.
3. **Not covered.** A command with several held-state ordinary branches: the idiom does not serve it (RS-D1 A's recorded cost).
4. **Release train:** next minor (a help text and a page).

## Decisions

- RS-D1: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET; reversible until a release carries it.

## Acceptance

- the_help_names_the_idiom: `ess` help for `ESS-COMMAND-004` says to drop `when_subject_state:` from the ordinary branch, keep
  `wrong_state:` and declare the related-state refusal above it.
- the_reference_names_the_idiom: the `when_related` reference page carries the same idiom with one example that validates.
- the_limit_is_stated: both say the idiom does not serve a command with several held-state ordinary branches.

## Scope

- inferred: the diagnostic table of `ess-compiler` (`ESS-COMMAND-004`), `website/docs/reference/` (the `when_related` page), `CHANGELOG.md`
