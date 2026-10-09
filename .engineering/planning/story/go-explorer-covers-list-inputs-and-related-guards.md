---
format: aep.planning-md/3
id: story:go-explorer-covers-list-inputs-and-related-guards
kind: story
status: draft
title: The Go explorer covers List inputs and related guards, and a skip counts
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#512
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The generated Go explorer covers commands with `List` inputs and with `related` / `related_set`
guards, and a command it still skips counts against the run's result.

## Evidence

GitHub https://github.com/beyond10x/ess/issues/512 (ess 0.56.0): `Explore` excludes every command with a `List` input
("input `aud` is a `list`") and every command with a `related` condition; 4 of 6 commands in the
requester's model, and a planted defect in `ReceiveTokenResponse` went uncaught.

## Acceptance

- The explorer generates `List` input values the way the synthesizer does, including empty and
  several-element lists.
- Commands with `related` / `related_set` guards are explored with fixture rows that satisfy and
  refute each guard.
- A skipped command is reported with its reason and fails the run unless the caller names it.
- The requester's planted `aud` defect is caught by a seeded explorer run in a test.
