---
format: aep.planning-md/3
id: story:a-group-named-like-a-top-level-command-of-its-component-is-refused
kind: story
status: draft
title: A group named like a top-level command of its own component is refused
summary: Inside one component a group and a top-level command may share a word and the clap tree defines it twice (W2 GEN2 review finding, pre-existing, backlog)
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:adversary-w2-gen2-pass-1
revision: 1
---
## Need

Inside one component, a command group named like one of that component's own top-level commands
validates (`validate_command_placement`, `crates/specify/ess-domain/src/component.rs:1043`), so the
generated clap tree defines that word twice and one of the two cannot be reached. Found by the W2
GEN2 adversary pass (review-result:adversary-w2-gen2-pass-1, F2) as the control case of a
cross-component finding the unit fixed; pre-existing at 4a0d58c611.

## Acceptance

- `ess specify validate` refuses a component whose group name equals one of its own top-level
  command or view words, naming the component, the group and the command or view.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.

## Repro

The adversary's case `adv_gen2_control_one_component_naming_a_group_like_its_own_top_level_command_is_refused`
was left out of the committed test file because the finding is outside W2's scope.
