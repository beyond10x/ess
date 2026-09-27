---
format: aep.planning-md/2
id: story:concurrent-history-format
kind: story
status: draft
title: 'A concurrent history has a format: ess-history/1'
owner: ess
relations:
- serves: vision:O2
- decomposes: epic:concurrent-history-conformance
revision: 1
---
# Story: a concurrent history has a format

## Outcome

`ess-history/1` records a concurrent run: the `spec_digest`, the seed, the client count, and per
operation its client, command, subject identity, invoke instant, return instant and outcome. An
operation with no answer is `Indeterminate`, and its return instant is read as "after every other
operation" (the S2 rule, https://s2.dev/blog/linearizability). Go and TypeScript runners write it;
the Rust checker reads it. The document is the one `models/concurrent-history/` declares.

## Acceptance

- A history written by the Go runner and one written by the TypeScript runner from the same seed
  over `examples/billing` are equal bytes.
- A history whose `spec_digest` differs from the compiled IR is refused with a named diagnostic
  before any check runs.
- An operation marked `Returned` with no return instant is refused by name.
- An operation whose return instant precedes its invoke instant is refused by name.

## Scope

New format module in `ess-conformance`, its JSON Schema under `schemas/`, the Go and TS writers.
