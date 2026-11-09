---
format: aep.planning-md/3
id: story:a-group-named-like-a-top-level-command-of-its-component-is-refused
kind: story
status: draft
title: A group named like a top-level command of its own component is refused
summary: Inside one component a group and a top-level command may share a word and the clap tree defines it twice (W2 GEN2 review finding, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-high
- zero-open-issues
relations:
- informed_by: review-result:adversary-w2-gen2-pass-1
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-synth/tests/adversary_w2_gen2_adversary.rs
- confidence: cited
  path: crates/specify/ess-domain/src/component.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/
revision: 5
---
## Need

Inside one component, a command group named like one of that component's own top-level command or
view words validates: `validate_command_placement`
(`crates/specify/ess-domain/src/component.rs:1043-1118`) checks that each command has one place and
is accepted, but never compares a group name with a top-level word. The generated clap tree then
defines that word twice and one of the two cannot be reached. Found by the W2 GEN2 adversary pass
(review-result:adversary-w2-gen2-pass-1, F2) as the control case of a cross-component finding the
unit fixed in `validate_shared_binaries` (`component.rs:1426`); pre-existing at 4a0d58c611.

Reproduction on 0.92.0: two domains `desk.audit` and `desk.note`, each with one command
`RecordEntry` (wire words `record` and `jot`), and one component:

```yaml
components:
  - component: desk-service
    owns: {domains: [desk.audit, desk.note]}
    accepts: {commands: [desk.audit.RecordEntry, desk.note.RecordEntry]}
    reached_by: command_line
    cli:
      binary: desk
      commands: [desk.audit.RecordEntry]
      groups:
        - name: record
          commands: [desk.note.RecordEntry]
```

```text
$ ess specify validate --path spec
desk v1 — 4 file(s), valid
exit=0
$ ess generate synthesize --path spec --target clap --out out
$ grep -rn 'Command::new("record")' out
out/crates/desk-cli/src/tree.rs:15:            ::clap::Command::new("record")
out/crates/desk-cli/src/tree.rs:24:            ::clap::Command::new("record")
```

## Acceptance

- `ess specify validate` on the specification above exits non-zero with a `DuplicateDeclaration`
  error at `component desk-service` naming the group `record` and the command
  `desk.audit.RecordEntry`.
- The same holds for a group named like a top-level view word of the same component, naming the
  view.
- A group whose name differs from every top-level word of its component validates as before, and
  the committed clap fixtures keep their bytes.
- The adversary case
  `adv_gen2_control_one_component_naming_a_group_like_its_own_top_level_command_is_refused` is
  committed in `crates/generate/ess-synth/tests/adversary_w2_gen2_adversary.rs` and passes.

## Scope

- cited: crates/specify/ess-domain/src/component.rs
- cited: crates/generate/ess-synth/tests/adversary_w2_gen2_adversary.rs
- inferred: crates/specify/ess-domain/tests/

confidence: high

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
