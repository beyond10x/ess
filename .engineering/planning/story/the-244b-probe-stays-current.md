---
format: aep.planning-md/3
id: story:the-244b-probe-stays-current
kind: story
status: draft
title: The window-free byte probe stays current with the byte tables
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by wave D368 U2 and U3: `tests/adversary_244b_window_free_bytes.rs` is `#[ignore]` by default and runs only under `task test-slow-probes`, which neither `task check` nor CI calls (`story:slow-probe-244b-follows-intended-moves` last re-pinned it). With `--ignored` it was red at the D368 base for `text-length-positions` and `binding-arrangement`, pinned by commits before the wave; no digest the wave replaced survives in `fixtures/adversary-244b-base-digests.tsv`.

## Acceptance

- The probe's base digests match the current suites of every model it lists, or the probe is retired with its cases moved to a test that runs by default.
- If it stays ignored, a check in `task check` fails when a byte table moves a model the probe lists without the probe moving with it.
