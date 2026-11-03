---
format: aep.planning-md/3
id: story:unreadable-git-tests-report-a-privileged-skip
kind: story
status: implemented
title: Unreadable-.git tests report a skip on a privileged runner instead of passing
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T20:41:38Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-02T20:41:38Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-02T20:41:38Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
# Story: unreadable-git-tests-report-a-privileged-skip

## Outcome

A test run on a privileged runner says that the three unreadable-`.git` tests checked nothing,
instead of reporting them as passed.

## Context

Review 2026-10-21, finding 3. `crates/edge/ess-cli/src/git_checkout.rs` (test
`an_unreadable_git_directory_is_an_error_not_an_absence`),
`crates/edge/ess-cli/tests/observed_bindings.rs`
(`live_output_below_an_unreadable_git_directory_is_refused_before_collection`) and
`crates/edge/ess-xtask/src/infra_acceptance.rs`
(`a_scratch_below_an_unreadable_git_directory_is_refused`) skip their only assertion when
`read_dir` succeeds through mode `0o000`, and pass.

## Acceptance

- On a runner where the directory stays readable and `CI` is unset, each test prints
  `skipped: a privileged runner reads through mode 0o000` to stderr.
- On such a runner with `CI` set, each test fails, so a privileged CI runner cannot report it
  as passed.
- On an unprivileged runner each test still asserts the refusal.

## Out of Scope

Running CI as root.

## Ambiguities

None.

## Open Questions

None.
