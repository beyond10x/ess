---
format: aep.planning-md/3
id: story:ci-test-shards-leave-evidence-when-the-runner-runs-out-of-disk
kind: story
status: draft
title: CI test shards leave evidence when the runner runs out of disk
summary: Three shards crashed with No space left on device about 2 min into the test step on 2026-11-03 (runs 41703340966, 41721030813); cause unknown, no logs (from-finding, backlog)
tags:
- backlog
- from-finding
revision: 2
---
## Need

On 2026-11-03, three CI test shards crashed. In each, the hosted runner's worker died with
"System.IO.IOException: No space left on device" while writing its own diagnostic log, about 2m08s
after the "Workspace tests N/20" step started:

- main push run 41703340966, Test 20/20 (job 126162441554), 20:22:50 to 20:24:58Z;
- PR #724 run 41721030813, Test 6/20 (job 126222359226), 21:51:09 to 21:53:17Z;
- PR #724 run 41721030813, Test 19/20 (job 126222359469), 22:13:24 to 22:15:33Z.
- main push run 41752885466 (2026-11-04, the 0.85.0 tag commit 2b5fa98db3), Test 11/20 (job
  126329057413), 01:54:45 to 01:56:52Z;
- the same run, Test 6/20 (job 126329057620), 01:57:44 to 01:59:50Z. This is the second crash of
  shard 6 in two runs.

None of the crashed runners uploaded a log, so no failing test is known. The same tree
(51b439e784, at a5824dd176) was green on shards 6, 19 and 20 in run 41690627394. On the 18 green
shards of run 41721030813, df reported 145G size and 121G free after "Free runner disk", and 112G
free after "Drop the workspace archive". What fills about 110G within about two minutes is not
known. Candidates (inferred, not observed): a test that writes without bound, or a runner with
less disk than 145G.

## Acceptance

- When a test shard runs out of disk, the run still leaves evidence of what was filling it: a
  background `df`/`du` sample over `target/` and the temp directories during the workspace and
  feature-off test steps, written somewhere that survives the worker crash (for example step
  output flushed at intervals, or an artifact uploaded from an `always()` step).
- The cause the evidence names is fixed, or the shard leaves room for it, so that the 20 test
  shards of one run are not lost to "No space left on device".

## Schedule

Backlog. Found during the 0.85.0 release follow-up (PR #724) and recorded at the orchestrator's
request. A second crash of the same kind on the rerun of run 41721030813 makes the sampler the
next step.
