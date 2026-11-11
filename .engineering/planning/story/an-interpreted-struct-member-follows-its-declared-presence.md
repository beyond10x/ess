---
format: aep.planning-md/3
id: story:an-interpreted-struct-member-follows-its-declared-presence
kind: story
status: draft
title: An interpreted struct member follows its declared presence
summary: A payload struct member declaring null_when_absent is left out, not null, in the interpreted event (turn-2 adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:zi-t2-interp-adv-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

The interpreted target writes an absent top-level event field as `null` under `null_when_absent`
since `story:the-interpreted-target-publishes-an-absent-event-field-in-its-declared-presence`. A
payload struct literal built member by member (`detail: {kind: input.kind, note: input.note}`)
whose member `note` declares `null_when_absent` still leaves the member out
(`crates/verify/ess-conformance/src/interpret/execute.rs`, the `Struct` arm of the payload
evaluation): the step's event holds `{"kind": "k"}`, not `{"kind": "k", "note": null}`. The suite
does not observe it yet. Found by the turn-2 adversary (`review-result:zi-t2-interp-adv-1`); the
story scoped its rule to top-level fields and the base behaves the same.

## Acceptance

- `adv_struct_member_null_when_absent_step_spells_null` in
  `crates/verify/ess-conformance/tests/adversary_t2_interp_presence_pass1.rs` runs (no
  `#[ignore]`) and passes.
- A struct member under `omitted_when_absent` or no presence is still left out.
- Interpreted history bytes of specifications declaring no `presence:` are unchanged.

## Scope

- `crates/verify/ess-conformance/src/interpret/execute.rs`
- `crates/verify/ess-conformance/tests/adversary_t2_interp_presence_pass1.rs`
