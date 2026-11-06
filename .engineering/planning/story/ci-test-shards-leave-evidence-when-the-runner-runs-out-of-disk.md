---
format: aep.planning-md/3
id: story:ci-test-shards-leave-evidence-when-the-runner-runs-out-of-disk
kind: story
status: implemented
title: CI test shards leave evidence when the runner runs out of disk
summary: Three shards crashed with No space left on device about 2 min into the test step on 2026-11-03 (runs 41703340966, 41721030813); cause unknown, no logs (from-finding, backlog)
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T08:56:06Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-04T08:56:07Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-05T17:37:15Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
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

Ride-along of the 0.87.0 wave (approval-record:ess-0870-wave), on its integration branch.

Two more shards crashed the same way on main run 41783353819 (2026-11-04, 8d26c81f74): Test 8/20 (job
126426194652) 2m10s and Test 18/20 (job 126426194659) 2m05s after their "Workspace tests" step started. Neither
left a log.

Measured on green Test 6/20 of the same run (job 126481404659):

| point | `/` used | `/` free | `target/` |
|---|---|---|---|
| job start | 59G | 86G | |
| after "Free runner disk" | 24G | 121G | |
| both archives downloaded (10,660 MiB workspace + 7,099 MiB feature-off) | | about 103G | 17G |
| workspace archive extracted: 1,800 binaries in 2m43s | | | |
| after the workspace tests | 90G | 55G | 64G |
| after dropping the workspace archive and its binaries | 34G | 111G | |
| after the feature-off tests | 65G | 81G | 38G |

The extraction alone takes 2m43s, so both crashes fell inside it, before any test ran. The extracted binaries are
about 46G (64G less the 17G of archives), so a shard with 121G free ends the extraction with about 55G to spare.
To crash there, a runner would need about 45G less free space than this one after "Free runner disk". One
explanation is a runner with a smaller root disk, but nothing has observed one: the crashed runners left no log.
The archive build already strips symbols and debug information (`the_archive_jobs_build_the_test_binaries_without_debug_information`),
so a smaller debug profile cuts nothing.
