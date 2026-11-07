---
format: aep.planning-md/3
id: story:a-list-identity-guard-beside-an-updates-subject-is-witnessed
kind: story
status: active
title: Synthesis witnesses a list-of-identity guard beside an updates or creates subject
summary: 'Issue 761: list guard scenarios fail or are refused ESS-SYNTH-019; queued first for 0.90.0'
tags:
- backlog
- from-issue
- queue-0.90.0
- rank-1
refs:
- provider: github
  reference: beyond10x/ess#761
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:18:51Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-06T22:18:52Z", actor: "human:timo", revision: 4}
---
## Need

Issue https://github.com/beyond10x/ess/issues/761, reproduced on ess 0.88.0 with a three-file model
whose list-of-identity guard (`exists p in permissions: (p == telephony)`) sits beside an `updates:`
subject and, in a variant, a `creates:` command. Queued first for 0.90.0.

- D1: `UpdateRolePermissions/outcome/unknown-permission` arranges the Permission rows but no Role,
  so the target answers `not-found` instead of `unknown-permission`.
- D2: `UpdateRolePermissions/outcome/updated-with-telephony` sends two generated ids, never the
  literal the branch's own guard needs, so the target answers `updated` with the flag false.
- D3: with the guarded outcome in the creating command, the planner sends `permissions: []` to
  every step that needs the subject and synthesis refuses with ESS-SYNTH-019, whose help text calls
  it a defect in ess.

## Acceptance

- The issue's model is a fixture. `synthesize` on it gives no ESS-SYNTH-019 refusal, and `run`
  against the interpreted target passes every scenario it emits.
- D1's scenario arranges the subject before sending one known and one unknown member.
- D2's scenario sends a list containing the guard's literal, arranged as a row.
- D3's creating step is sent a list that satisfies its guard when a later step needs that branch,
  or the branch is not required for a step that needs only the subject.

## Scope

- inferred: `crates/generate/ess-synth/src/` (scenario planner, guard witnesses for list-of-identity
  membership), `crates/verify/ess-conformance/tests/` (new fixture case).
