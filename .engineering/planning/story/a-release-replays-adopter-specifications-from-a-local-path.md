---
format: aep.planning-md/3
id: story:a-release-replays-adopter-specifications-from-a-local-path
kind: story
status: draft
title: A release replays adopter specifications from a local read-only path
summary: Before tagging, a release-gate step runs adopter specifications from a local read-only path outside the repository, set by an environment variable, and counts a downstream fix done when the adopter refusal count drops.
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

A fix for an adopter-reported synthesis refusal is counted as done when the neutral reproduction passes. On 2026-10-20 seven of seven neutral repros passed while the adopter specification they came from kept 45 refusals in the same categories, so the release said the gap was closed and it was not.

Adopter specifications cannot enter the repository: they are not ours to publish. They exist as read-only local clones outside it.

## Acceptance

- `an_adopter_replay_reads_specifications_from_the_configured_path`: a release-gate step (a `cargo xtask release` subcommand or `task` target) reads a list of specification roots from an environment variable naming a local read-only path outside the repository, runs `ess verify conform synthesize` over each, and prints per root the scenario count and the refusal count by category.
- `an_adopter_replay_without_the_variable_is_skipped_and_says_so`: with the variable unset the step prints that it was skipped and exits 0; CI never sets it.
- `an_adopter_replay_never_writes_inside_the_path_or_the_repository`: the step writes nothing under the configured path and nothing tracked by git; a path inside the repository is refused.
- `an_adopter_replay_compares_with_the_previous_release`: given a previous `ess` binary, the step reports the refusal-count delta per root and category, so a downstream fix counts as done when the adopter count drops.
- The release procedure in `AGENTS.md` runs the step before tagging when the variable is set, and the release announcement carries the counts without naming the specification.
