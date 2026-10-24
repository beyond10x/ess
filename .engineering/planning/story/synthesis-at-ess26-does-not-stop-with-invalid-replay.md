---
format: aep.planning-md/3
id: story:synthesis-at-ess26-does-not-stop-with-invalid-replay
kind: story
status: draft
title: Synthesis at ess/26 does not stop with InvalidReplay
summary: 'Adopter report (2026-10-22, ess 0.68.0): synthesis stops at ess/26 with InvalidReplay; not yet reproduced.'
tags:
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

An adopter reported on 2026-10-22, against ess 0.68.0: "synthesis stops at ess/26 with InvalidReplay". No specification came with the report, and no store artifact or `CHANGELOG.md` entry names `InvalidReplay`, so 0.69.0 is not known to fix it.

`InvalidReplay` is raised in two places, both while admitting a suite:

- `crates/verify/ess-conformance/src/replay.rs:592` (`admit_suite`), when `validate_steps` refuses a scenario's retained-result steps: a `capture_command_result` not bound to the invocation identity, a capture overwritten, or an `expect_replay_result` with no matching original capture.
- `crates/verify/ess-conformance/src/admission.rs:673`, when a `capture` field does not decode as a replay observation.

Format `ess/26` made an external refusal beside an existing-row branch read the addressed row back and require it unchanged (#587). A `replays:` outcome is external, so the hypothesis (inferred) is that the read-back steps synthesized at `ess/26` land between the original command and its capture or replay, and `validate_steps` then refuses the scenario.

Not reproduced on 0.69.0. The fixtures `retained-replay.yaml` and `retained-commit.yaml` (under `crates/verify/ess-conformance/tests/fixtures/`), raised to `ess/25`, `ess/26` and `ess/31`, synthesize with 0 refusals and admit their `capture_command_result` and `expect_replay_result` steps; `retained-commit` gives byte-identical suites at `ess/25` and `ess/26`.

Priority: this adopter's reports go first; this story is ahead of the other 0.70.0 adopter-report stories.

## Acceptance

- First, a reproduction: the reporter's specification, or a neutral one built from it, that stops `ess verify conform synthesize` with `InvalidReplay` on 0.68.0, checked in as a fixture.
- `a_replay_beside_an_ess26_read_back_is_admitted`: that fixture synthesizes at `ess/26` and later, and every synthesized scenario passes `admit_suite`.
- `a_synthesized_suite_never_fails_its_own_admission`: synthesis that would write a suite its own admission refuses reports a typed refusal naming the scenario and the step, instead of stopping.
- Models below `ess/26` keep their suite bytes.
