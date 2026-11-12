---
format: aep.planning-md/3
id: story:two-shells-share-the-chosen-theme
kind: story
status: draft
title: Two shells share the chosen theme
summary: 'Follow-up of story:light-and-dark-themes-render-differently: a local_storage chosen_by state is keyed by shell path, so two shells keep different themes although the design says they share one value (adversary pass 1, backlog)'
tags:
- backlog
- defect
- ess-ui
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

The `theme:` design (design § Themes) says that two shells which each declare the `chosen_by`
state "then share one value". The React renderer keys a `local_storage` state by its shell path
(`placement_call` and `storageKey` in `storage.ts`), so each shell keeps its own value: a theme
picked in one shell is not the theme of the other.

Found by adversary pass 1 of story:light-and-dark-themes-render-differently (pre-existing, not
introduced by that story).

## Acceptance

1. With two shells that each declare the `chosen_by` state, a value chosen in one is the value the
   other reads after navigation and after a reload; a test in `crates/ui/ess-ui-react/tests` pins
   it in a generated project.
2. A `local_storage` state that is not the `chosen_by` state keeps its per-shell key and its bytes.
