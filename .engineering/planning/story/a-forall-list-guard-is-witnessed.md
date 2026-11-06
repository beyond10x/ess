---
format: aep.planning-md/3
id: story:a-forall-list-guard-is-witnessed
kind: story
status: draft
title: Synthesis witnesses a forall guard over an identity list
summary: Vacuous forall over an empty list is sent two generated rows (0.90.0 U1 adversary; from-finding)
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
revision: 1
---
## Need

Found by the U1 adversary of wave-2026-11-07-ess (ESS 0.90.0). Pre-existing, beside issue 761.

A `forall p in <list>: (p == <literal>)` guard beside an `updates:` subject is vacuously true for
an empty list. Synthesis does not treat the empty list as refuting the guard, so it sends two
generated rows. The guarded scenario then fails on the interpreted target.

The red case is `adversary_761_forall_guard_beside_an_updates_subject_is_witnessed` in
`crates/verify/ess-conformance/tests/adversary_761_list_identity_guard.rs`. It is ignored with
this story's id until fixed.

## Acceptance

- That case passes. The guarded scenario sends a list whose every member satisfies the guard,
  arranged as rows. The unguarded scenario sends one that does not.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize/related_list.rs`,
  `related_guard.rs`.
