---
format: aep.planning-md/3
id: story:the-a16-socket-case-binds-under-any-tmpdir
kind: story
status: draft
title: The a16 socket case binds under any TMPDIR
summary: The ess-cli a16 input-discovery case fails with 'path must be shorter than SUN_LEN' when TMPDIR is long; it fails the same at the turn-1 base (turn 1 gate, pre-existing, backlog)
tags:
- backlog
- defect
- severity-low
- zero-open-issues
revision: 1
---
## Need

The ess-cli test `authored_scenarios input_discovery::a16_selected_links_kinds_and_manifest_links_refuse_unlisted_ones_are_ignored`
binds a Unix socket through a symlink in `tempfile::tempdir()`
(`crates/edge/ess-cli/tests/support/input_discovery_cases.rs:1065-1072`, `fn socket`). The comment
says this keeps the bind path below the socket length limit, but the temporary directory is
created under `TMPDIR`, so a long `TMPDIR` still makes the bind fail with "path must be shorter
than SUN_LEN". The turn-1 gate of the zero-open-issues plan ran the tests with `TMPDIR` set to a
build scratch path. The test failed there both on the integration head and on the turn's base
`ab09f5b48d`, so the defect predates the turn. Under nextest, at load 90, it also hit the 30 s
slow-timeout first.

## Acceptance

- The a16 case passes with `TMPDIR` set to a directory whose path is 100 bytes long.
- The socket is still created in the destination filesystem of the selected path, as the
  case requires.

## Scope

- `crates/edge/ess-cli/tests/support/input_discovery_cases.rs` (`fn socket`)
