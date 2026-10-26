---
format: aep.planning-md/3
id: story:a-set-effect-bound-to-a-fresh-input-composes-after-a-disclosure
kind: story
status: draft
title: A set effect bound to a fresh input composes after a one-time-response disclosure
summary: ESS-SYNTH-001 refuses the disclosure cell before an instances-where follow-up; 9 / 1, 11 / 0 with instance:.
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, neutral repro (format ess/25, `demo.*` names; committable as a fixture). Counts are scenarios / refusals from `ess verify conform synthesize --suite-format 5 --compact`; reproduced here on 0.69.2 and on the 0.70.0 integration head 3f7602f05 (same refusal counts).

A creator with `one_time_response: [secret]` and a follow-up command whose outcome selects rows with `instances: {where: owner == input.owner}`: the disclosure cell `…/disclosure/issued/secret/command/…/parked/as/anonymous` is refused (ESS-SYNTH-001, "the follow-up assertions cannot compose with the retained origin state"): 9 / 1. With `instance: owner` instead, 11 / 0. Reporter hypothesis (unverified): `synthesize/caller.rs` `reads_every_row` treats any set effect as reading every row, so the appended second run is refused, although the selector is bound to a fresh `input.owner` per run.

Classification: defect. The message is also misleading: nothing in the retained origin state conflicts, the selector is what is refused.

## Acceptance

- `a_set_effect_bound_to_a_fresh_input_composes_after_a_disclosure`: the repro as a fixture synthesizes the disclosure cell; the second run's set effect selects only its own rows.
- A set effect whose selector does not bind a per-run value stays refused, with a message naming the selector.
