---
format: aep.planning-md/3
id: story:the-sequential-and-concurrent-explorers-exclude-alike
kind: story
status: draft
title: The sequential and concurrent explorers exclude alike
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The generated sequential and concurrent explorers exclude a command for the same reason, and report
the same first reason, except where the concurrent design needs a different answer and names it
(`preserves`; a delete, whose deleted row a workload drawn before any call runs can still name).
Measured over the 39 explorer fixtures at 0.60.0, about 25 commands differ today: the concurrent
explorer excludes every condition read from the stored row; only the sequential one excludes "no
actor may invoke it", `affects:` and `replays`; and the two name a different first reason for the
same command.

## Acceptance

- `sequential_and_concurrent_explorers_exclude_alike`: for every fixture model the explorers
  explore, Go and TypeScript, the two report the same exclusions with the same first reason, except
  the named concurrent-only ones.
- A stored-row condition is decided by the concurrent explorer against the row each linearization
  holds, or excluded by both with one reason.
- Tests are modules of existing test binaries; no new `tests/*.rs` file.
