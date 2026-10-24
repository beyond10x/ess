---
format: aep.planning-md/3
id: story:a-caller-swap-redraw-keeps-unread-members-and-related-ties
kind: story
status: active
title: A caller-swap redraw keeps unread members and related ties
summary: 'Adopter reports N24, N25: the swapped replay redraws a whole struct and breaks related ties (ess#672).'
refs:
- provider: github
  reference: beyond10x/ess#672
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T10:47:53Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-23T10:47:54Z", actor: "human:timo", revision: 3}
---
## Problem

The caller-swapped replay redraws every input another command's guard reads (#580). It cuts a struct member to its root and redraws the whole struct, so optional members of the arranged input come back absent; and it redraws an input a related predicate ties to a stored value without keeping the tie. `crates/verify/ess-conformance/src/synthesize/caller.rs`: `selected`, `draw`, `fresh_everywhere`; `synthesize.rs` `keeps_branch`. Adopter reports N24 and N25 (ess#672), ess 0.68.0.

## Acceptance

- `a_swapped_redraw_keeps_unread_struct_members`: with a row-set guard on another command reading `Book.tags`, the swapped `CreateBook` keeps `settings.cover`; 6 of 6 scenarios pass (fixture from the N24 repro, YAML).
- `a_swapped_redraw_redraws_only_the_read_member`: `settings.tags` is fresh in the swapped half, `settings.cover` equals the first half's value.
- `a_related_predicate_tie_survives_the_swap`: `StartReview/outcome/same-version` sends the arranged book's stored version in both halves; 10 of 10 pass (fixture from the N25 repro, YAML).
- `keeps_branch_checks_the_command_under_test`: a redraw that would move the scenario to another branch of the command under test is refused, and the case is pinned.
