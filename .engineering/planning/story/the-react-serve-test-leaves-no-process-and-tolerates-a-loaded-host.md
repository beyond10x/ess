---
format: aep.planning-md/3
id: story:the-react-serve-test-leaves-no-process-and-tolerates-a-loaded-host
kind: story
status: draft
title: The React serve-without-stdin test leaves no process behind and holds on a loaded host
summary: adversary_react_plain_pass1 leaked an esbuild serve process and missed its 5 s deadline under load (0.89.0 gate; from-finding)
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
revision: 1
---
## Need

Seen during the ESS 0.89.0 local gate (wave-2026-11-06-ess). Pre-existing, outside every unit of
that wave.

- `crates/ui/ess-ui-react/tests/adversary_react_plain_pass1.rs` (`serves_without_stdin`) starts the
  generated project's `dev` and `preview` scripts and waits five seconds for an HTTP answer. Under a
  loaded host both cases failed with "never listened". Run alone, they passed 4 of 4.
- After that test ran, an esbuild serve process stayed alive with its working directory inside the
  worktree. `worktree discard-cache` refused the tree as in use until that process was killed by
  PID. The `exec` in the test is meant to leave no orphan, so something it starts outlives the drop.
  Which process spawns the survivor is not yet established.

## Acceptance

- After `cargo test -p ess-ui-react --test adversary_react_plain_pass1`, no process started by the
  test is alive. A case checks this, for example by asserting the server's process group is gone
  after the drop.
- The listen deadline holds on a loaded host: the cases pass while another test binary saturates
  the cores, or the deadline is derived from the first answer rather than fixed at five seconds.

## Scope

- inferred: `crates/ui/ess-ui-react/tests/adversary_react_plain_pass1.rs` (`Server` drop,
  `serves_without_stdin`); possibly the generated `dev` and `preview` scripts in `ess-ui-react`.
