---
format: aep.planning-md/3
id: story:the-ess31-history-states-the-optional-read
kind: story
status: draft
title: The ess/31 history sentence states how an Optional read behaves
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by wave D368 U1's review passes, pre-existing and out of that story's Acceptance: the `ess/31` history sentence at `system.rs:335` says an `Optional` read is refused as `type_mismatch`, which no longer matches what the language does.

## Acceptance

- The sentence states the current behaviour, checked against a test that reads an `Optional` field at `ess/31`.
- No other change.
