---
format: aep.planning-md/3
id: story:a-related-row-optional-reached-through-a-member-or-a-later-command-is-arranged
kind: story
status: archived
title: A related row's Optional reached through a struct member or a later command is arranged
summary: Synthesis refuses a witnessable found branch when the Optional a creator's guard compares with the input is a struct member or is written only by a later update.
tags:
- ess-0.71.0
relations:
- derived_from: review-result:adversary-ess-0700-unit-b-pass-2
revision: 3
transitions:
- {from: "draft", to: "archived", at: "2026-10-31T23:45:01Z", actor: "human:timo", revision: 3}
---
## Finding

Found 2026-10-26 by adversary pass 2 on the 0.70.0 wave's unit B
(`review-result:adversary-ess-0700-unit-b-pass-2`). From 0.70.0, conformance synthesis arranges a
related row past a creator's predicate guard that compares an Optional stored field of the row
with the input, by writing that field. Two shapes still fall through to ESS-SYNTH-001 although the
interpreter witnesses the `found` branch by direct sends:

- the Optional is a member of a required struct field (`profile.tenant`): only root fields are
  considered (`row_set.rs` `namespace()`);
- the Optional is written only by a later `updates:` command, not by the row's creator: the row
  is never arranged through that command (`row_set.rs` `related_row`).

## Acceptance

- Both shapes synthesize a `found` witness that passes on the interpreted target; the pass-2
  cases `adv2_optional_member_of_a_struct_compared_with_the_input` and
  `adv2_optional_written_only_by_a_later_command` (kept in the wave scratch) go green.
- No other model's pinned suite bytes move.

## Triage (D338)

Declined at the 2026-10-31 findings triage (orchestrator D338, operator approval relayed by host-monitor D53): an honest refusal with no wrong scenario and no downstream model named; reopen when a model hits it.
