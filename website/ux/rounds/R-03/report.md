# UX round R-03: ESS documentation website

Date 2026-11-22. Engine: commit `7db25740980e`, digest `bc3e06f44faa130b`. Gating viewports desktop-1280, mobile-390; measured only desktop-1440, desktop-1024, tablet-768; themes light and dark. Hermetic: every request off the local origin aborted.

Cards: anchors `first-conformance-run`, `install-linux-x86_64`; rotating `go-runner`. No person ran them this round: the screen-only user played every task (L3), the questionnaire has no answers, and no session was recorded.

## Coverage

| what | measured of expected |
| --- | --- |
| pages (every configuration measured) | 139 of 139 |
| page configurations (page × gating viewport × theme) | 556 of 556 |
|   desktop-1280/dark | 139 of 139 |
|   desktop-1280/light | 139 of 139 |
|   mobile-390/dark | 139 of 139 |
|   mobile-390/light | 139 of 139 |
| task runs | 22 of 22 |
| measured-only configurations | 417 of 417 |

Unmeasured: none.

## Gates

| gate | result | counts |
| --- | --- | --- |
| 1 no horizontal overflow | pass | 0 of 139 measured pages; 0 with page scroll; 0 spills |
| 2 always visible | fail | 2 of 139 measured pages; docs sidebar: out of viewport ×2; blog sidebar: out of viewport ×2; measured only, not gated: docs sidebar: out of viewport at end ×136; blog sidebar: out of viewport at end ×136 |
| 3 unmissable completion | pass | completion_miss 0, false_complete 0 |
| 4 screen-only user | fail | 18 of 18 pointer runs completed, 10 over the card's action limit |
| 5 accessibility (axe) | fail | 31 serious/critical nodes on 3 of 139 measured pages; label-content-name-mismatch (serious) 6 nodes on 2 pages; color-contrast (serious) 22 nodes on 2 pages; nested-interactive (serious) 2 nodes on 1 pages; scrollable-region-focusable (serious) 1 nodes on 1 pages |
| 5 keyboard only | pass | 4 of 4 keyboard runs completed |
| truncation | fail | 6 texts on 2 of 139 measured pages |
| reduced motion | pass | 0 of 139 measured pages animate or scroll smoothly under reduce |
| 6 cue truth | dropped | no guidance cue on a docs site |

Task runs over their card's limit:

- diagnostic-type-009/mobile-390: 7 actions, limit 5
- ess-31-release/desktop-1280: 25 actions, limit 6
- ess-31-release/mobile-390: 33 actions, limit 6
- first-conformance-run/mobile-390: 4 actions, limit 3
- limitations/mobile-390: 4 actions, limit 3
- page-a-view/mobile-390: 11 actions, limit 5
- target-process/desktop-1280: 5 actions, limit 4
- target-process/mobile-390: 12 actions, limit 4
- theme-dark-and-back/desktop-1280: 3 actions, limit 2
- theme-dark-and-back/mobile-390: 4 actions, limit 2

## Runs

Every run of this round, with its own time against the 120 s step budget of the gate lane. A project that ran no test is not listed. "Known" failures are the regression assets of open findings, expected to fail; "new" failures are every other failed test and every open finding's asset that passed: a run exits 1 on any new failure.

| run | lane | engine | workers | wall s | status | failed: known, new | projects (tests, wall s) | pages | task runs | unmeasured |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| slow-l2-1 | slow | 7db25740980e bc3e06f44faa130b | 4 | 56.2 | passed | 3, 0 | l2-1 47, 55.4 | 47 of 47 | 0 of 0 | 1 |
| slow-l2-2 | slow | 7db25740980e bc3e06f44faa130b | 4 | 64.4 | passed | 3, 0 | l2-2 47, 63.4 | 47 of 47 | 0 of 0 | 1 |
| slow-l2-3 | slow | 7db25740980e bc3e06f44faa130b | 4 | 134.1 | failed | 0, 1 | l2-3 46, 131.6 | 46 of 46 | 0 of 0 | 1 |
| slow-l2-4 | slow | 7db25740980e bc3e06f44faa130b | 4 | 225.3 | passed | 0, 0 | l2-4 46, 221.6 | 46 of 46 | 0 of 0 | 1 |
| slow-l2-5 | slow | 7db25740980e bc3e06f44faa130b | 4 | 312.6 | failed | 1, 2 | l2-5 46, 309.9 | 44 of 46 | 0 of 0 | 4 |
| slow-l2-6 | slow | 7db25740980e bc3e06f44faa130b | 4 | 260.7 | failed | 0, 2 | l2-6 46, 256.1 | 44 of 46 | 0 of 0 | 4 |
| slow-l3 | slow | 7db25740980e bc3e06f44faa130b | 4 | 139 | failed | 4, 6 | l3 22, 132.8 | 0 of 0 | 22 of 22 | 0 |
| slow-measured | slow | 7db25740980e bc3e06f44faa130b | 4 | 599.4 | failed | 0, 2 | measured 139, 595.3 | 0 of 0 | 0 of 0 | 6 |
| rerun-heavy | slow | 7db25740980e bc3e06f44faa130b | 4 | 104.9 | failed | 0, 4 | l2-5 2, 60.1; l2-6 2, 75.9 | 0 of 2 | 0 of 0 | 9 |
| rerun-l3 | slow | 7db25740980e bc3e06f44faa130b | 4 | 92.4 | passed | 10, 0 | l3 22, 86.3 | 0 of 0 | 22 of 22 | 0 |
| rerun-tags | slow | 7db25740980e bc3e06f44faa130b | 4 | 13.4 | passed | 1, 0 | l2-3 1, 7.4 | 1 of 1 | 0 of 0 | 1 |
| rerun-measured | slow | 7db25740980e bc3e06f44faa130b | 4 | 51.5 | passed | 0, 0 | measured 2, 41.1 | 0 of 0 | 0 of 0 | 1 |
| rerun-heavy2 | slow | 7db25740980e bc3e06f44faa130b | 4 | 72.9 | failed | 0, 2 | l2-5 2, 60.3; l2-6 2, 60.1 | 1 of 2 | 0 of 0 | 3 |
| rerun-essui | slow | unknown+ bc3e06f44faa130b | 2 | 51.9 | passed | 0, 0 | l2-5 1, 47; l2-6 1, 45 | 1 of 1 | 0 of 0 | 1 |
| rerun-shift | slow | unknown+ bc3e06f44faa130b | 2 | 20.2 | passed | 0, 0 | l2-4 1, 17.6 | 1 of 1 | 0 of 0 | 1 |
| gate-4 | gate | 7db25740980e bc3e06f44faa130b | 4 | 119.6 | timedout | 8, 0 | l1 36, 14.6; l2 19, 76.8; l3 7, 29.5 | 9 of 9 | 7 of 8 | 1 |
| gate-5 | gate | 7db25740980e bc3e06f44faa130b | 4 | 100.7 | passed | 8, 0 | l1 36, 21.1; l2 19, 52.6; l3 8, 25.6 | 9 of 9 | 8 of 8 | 0 |
| gate-6 | gate | 7db25740980e bc3e06f44faa130b | 4 | 93.6 | passed | 8, 0 | l1 36, 14.9; l2 19, 64.5; l3 8, 11.2 | 9 of 9 | 8 of 8 | 0 |

Ten slowest tests of the measurement:

| s | status | run | test |
| --- | --- | --- | --- |
| 60.2 | timedOut | rerun-heavy | L2 /ess/docs/reference/cli mobile-390 |
| 60.1 | timedOut | rerun-heavy | L2 /ess/docs/reference/ess-ui desktop-1280 |
| 60.1 | timedOut | rerun-heavy2 | L2 /ess/docs/reference/ess-ui mobile-390 |
| 60.1 | timedOut | rerun-heavy2 | L2 /ess/docs/reference/ess-ui desktop-1280 |
| 60 | timedOut | slow-l2-5 | L2 /ess/docs/reference/cli desktop-1280 |
| 60 | timedOut | slow-l2-6 | L2 /ess/docs/reference/cli mobile-390 |
| 60 | timedOut | slow-l3 | L3 ess-31-release mobile-390 pointer |
| 60 | timedOut | rerun-heavy | L2 /ess/docs/reference/ess-ui mobile-390 |
| 58.3 | passed | rerun-heavy2 | L2 /ess/docs/reference/cli mobile-390 |
| 58 | passed | rerun-heavy2 | L2 /ess/docs/reference/cli desktop-1280 |

Where the slowest pages spent their time (ms per step; -1: the step did not end):

- /ess/docs/reference/cli at mobile-390: goto 3675, hydrate 3119, theme light 819, layout light 326, visible light 2344, axe light 19559, theme dark 9027, layout dark 755, visible dark 2804, axe dark 15145
- /ess/docs/reference/cli at desktop-1280: goto 3036, hydrate 5367, theme light 688, layout light 830, visible light 2980, axe light 22707, theme dark 6222, layout dark 562, visible dark 2092, axe dark 12584
- /ess/docs/reference/formats at mobile-390: goto 4045, hydrate 2445, theme light 480, layout light 553, visible light 1841, axe light 23225, theme dark 2878, layout dark 278, visible dark 1570, axe dark 18919

## Screen-only user (L3)

| task | viewport | mode | done | actions (limit) | steers | looks | glance p95 | time to act max | reading load max | note |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| diagnostic-type-009 | desktop-1280 | pointer | yes | 4 (5) | 2 | 3 | 3 | 3 | 256 |  |
| diagnostic-type-009 | mobile-390 | pointer | yes | 7 (5) | 5 | 3 | 3 | 5 | 142 |  |
| ess-31-release | desktop-1280 | pointer | yes | 25 (6) | 23 | 3 | 3 | 24 | 291 |  |
| ess-31-release | mobile-390 | pointer | yes | 33 (6) | 31 | 3 | 3 | 30 | 142 |  |
| first-conformance-run | desktop-1280 | pointer | yes | 2 (3) | 0 | 1 | 2 | 1 | 190 |  |
| first-conformance-run | desktop-1280 | keyboard | yes | 23 (–) | 0 | 2 | 3 | 1 | 190 |  |
| first-conformance-run | mobile-390 | pointer | yes | 4 (3) | 2 | 2 | 3 | 2 | 108 |  |
| first-conformance-run | mobile-390 | keyboard | yes | 130 (–) | 2 | 2 | 3 | 2 | 108 |  |
| go-runner | desktop-1280 | pointer | yes | 2 (3) | 0 | 2 | 2 | 1 | 199 |  |
| go-runner | mobile-390 | pointer | yes | 3 (3) | 1 | 2 | 3 | 2 | 145 |  |
| install-linux-x86_64 | desktop-1280 | pointer | yes | 2 (4) | 0 | 1 | 2 | 1 | 190 |  |
| install-linux-x86_64 | desktop-1280 | keyboard | yes | 21 (–) | 0 | 2 | 3 | 1 | 190 |  |
| install-linux-x86_64 | mobile-390 | pointer | yes | 4 (4) | 2 | 2 | 3 | 2 | 120 |  |
| install-linux-x86_64 | mobile-390 | keyboard | yes | 128 (–) | 2 | 2 | 3 | 2 | 120 |  |
| limitations | desktop-1280 | pointer | yes | 2 (3) | 0 | 1 | 2 | 1 | 165 |  |
| limitations | mobile-390 | pointer | yes | 4 (3) | 2 | 2 | 3 | 2 | 106 |  |
| page-a-view | desktop-1280 | pointer | yes | 4 (5) | 0 | 3 | 2 | 1 | 230 |  |
| page-a-view | mobile-390 | pointer | yes | 11 (5) | 8 | 4 | 3 | 6 | 147 |  |
| target-process | desktop-1280 | pointer | yes | 5 (4) | 3 | 2 | 3 | 4 | 252 |  |
| target-process | mobile-390 | pointer | yes | 12 (4) | 9 | 2 | 3 | 8 | 122 |  |
| theme-dark-and-back | desktop-1280 | pointer | yes | 3 (2) | 0 | 0 | 1 | 1 | 215 |  |
| theme-dark-and-back | mobile-390 | pointer | yes | 4 (2) | 1 | 2 | 3 | 2 | 194 |  |

## Metrics against the baseline and the last round

All lower is better. 556 page configurations and 22 task runs hold values; the page values are summed per viewport and theme here, every value is in round.json.

The baseline column is `baseline.json` as committed with this round, after the ratchet took the round in: UX ratchet baseline of the ESS documentation website, lower is better; written by cargo xtask ux-ratchet --update: lane full from round R-03, lane gate from round R-03; 3556 values in 578 scenario(s)

| scenario | key | this round | baseline | last round |
| --- | --- | --- | --- | --- |
| pages @desktop-1280/dark (sum) | layout.spills | 0 | 0 | 0 |
| pages @desktop-1280/dark (sum) | layout.truncated_texts | 0 | 0 | 0 |
| pages @desktop-1280/dark (sum) | always_visible.failures | 2 | 2 | 2 |
| pages @desktop-1280/dark (sum) | axe.serious_critical_rules | 2 | 2 | 104 |
| pages @desktop-1280/dark (sum) | axe.serious_critical_nodes | 2 | 2 | 685 |
| pages @desktop-1280/dark (sum) | layout_shift.load | 0.015 | 0.013 | 0.013 |
| pages @desktop-1280/dark (sum) | motion.running | 0 | 0 | 0 |
| pages @desktop-1280/light (sum) | layout.spills | 0 | 0 | 0 |
| pages @desktop-1280/light (sum) | layout.truncated_texts | 0 | 0 | 0 |
| pages @desktop-1280/light (sum) | always_visible.failures | 2 | 2 | 2 |
| pages @desktop-1280/light (sum) | axe.serious_critical_rules | 5 | 5 | 105 |
| pages @desktop-1280/light (sum) | axe.serious_critical_nodes | 13 | 13 | 687 |
| pages @mobile-390/dark (sum) | layout.spills | 0 | 0 | 0 |
| pages @mobile-390/dark (sum) | layout.truncated_texts | 3 | 3 | 3 |
| pages @mobile-390/dark (sum) | always_visible.failures | 0 | 0 | 0 |
| pages @mobile-390/dark (sum) | axe.serious_critical_rules | 2 | 2 | 104 |
| pages @mobile-390/dark (sum) | axe.serious_critical_nodes | 2 | 2 | 679 |
| pages @mobile-390/dark (sum) | layout_shift.load | 0.039 | 0.028 | 0.028 |
| pages @mobile-390/dark (sum) | motion.running | 0 | 0 | 0 |
| pages @mobile-390/light (sum) | layout.spills | 0 | 0 | 0 |
| pages @mobile-390/light (sum) | layout.truncated_texts | 3 | 3 | 3 |
| pages @mobile-390/light (sum) | always_visible.failures | 0 | 0 | 0 |
| pages @mobile-390/light (sum) | axe.serious_critical_rules | 6 | 6 | 105 |
| pages @mobile-390/light (sum) | axe.serious_critical_nodes | 14 | 14 | 681 |
| diagnostic-type-009/desktop-1280 | screen_only.actions | 4 | 4 | 4 |
| diagnostic-type-009/desktop-1280 | screen_only.looks | 3 | 3 | 3 |
| diagnostic-type-009/desktop-1280 | screen_only.steers | 2 | 2 | 2 |
| diagnostic-type-009/desktop-1280 | screen_only.time_to_act_steps.max | 3 | 3 | 3 |
| diagnostic-type-009/desktop-1280 | glance_cost.median | 2 | 2 | 2 |
| diagnostic-type-009/desktop-1280 | glance_cost.p95 | 3 | 3 | 3 |
| diagnostic-type-009/desktop-1280 | reading_load.max | 256 | 256 | 256 |
| diagnostic-type-009/desktop-1280 | reading_load.median | 202 | 202 | 202 |
| diagnostic-type-009/desktop-1280 | completion_miss | 0 | 0 | 0 |
| diagnostic-type-009/desktop-1280 | false_complete | 0 | 0 | 0 |
| diagnostic-type-009/mobile-390 | screen_only.actions | 7 | 7 | 7 |
| diagnostic-type-009/mobile-390 | screen_only.looks | 3 | 3 | 3 |
| diagnostic-type-009/mobile-390 | screen_only.steers | 5 | 5 | 5 |
| diagnostic-type-009/mobile-390 | screen_only.time_to_act_steps.max | 5 | 5 | 5 |
| diagnostic-type-009/mobile-390 | glance_cost.median | 3 | 3 | 3 |
| diagnostic-type-009/mobile-390 | glance_cost.p95 | 3 | 3 | 3 |
| diagnostic-type-009/mobile-390 | reading_load.max | 142 | 142 | 142 |
| diagnostic-type-009/mobile-390 | reading_load.median | 120 | 120 | 120 |
| diagnostic-type-009/mobile-390 | completion_miss | 0 | 0 | 0 |
| diagnostic-type-009/mobile-390 | false_complete | 0 | 0 | 0 |
| ess-31-release/desktop-1280 | screen_only.actions | 25 | 25 | 25 |
| ess-31-release/desktop-1280 | screen_only.looks | 3 | 3 | 3 |
| ess-31-release/desktop-1280 | screen_only.steers | 23 | 23 | 23 |
| ess-31-release/desktop-1280 | screen_only.time_to_act_steps.max | 24 | 24 | 24 |
| ess-31-release/desktop-1280 | glance_cost.median | 2 | 2 | 2 |
| ess-31-release/desktop-1280 | glance_cost.p95 | 3 | 3 | 3 |
| ess-31-release/desktop-1280 | reading_load.max | 291 | 291 | 291 |
| ess-31-release/desktop-1280 | reading_load.median | 202 | 202 | 202 |
| ess-31-release/desktop-1280 | completion_miss | 0 | 0 | 0 |
| ess-31-release/desktop-1280 | false_complete | 0 | 0 | 0 |
| ess-31-release/mobile-390 | screen_only.actions | 33 | 33 | 33 |
| ess-31-release/mobile-390 | screen_only.looks | 3 | 3 | 3 |
| ess-31-release/mobile-390 | screen_only.steers | 31 | 31 | 31 |
| ess-31-release/mobile-390 | screen_only.time_to_act_steps.max | 30 | 30 | 30 |
| ess-31-release/mobile-390 | glance_cost.median | 3 | 3 | 3 |
| ess-31-release/mobile-390 | glance_cost.p95 | 3 | 3 | 3 |
| ess-31-release/mobile-390 | reading_load.max | 142 | 142 | 142 |
| ess-31-release/mobile-390 | reading_load.median | 117 | 117 | 117 |
| ess-31-release/mobile-390 | completion_miss | 0 | 0 | 0 |
| ess-31-release/mobile-390 | false_complete | 0 | 0 | 0 |
| first-conformance-run/desktop-1280 | screen_only.actions | 2 | 2 | 2 |
| first-conformance-run/desktop-1280 | screen_only.looks | 1 | 1 | 1 |
| first-conformance-run/desktop-1280 | screen_only.steers | 0 | 0 | 0 |
| first-conformance-run/desktop-1280 | screen_only.time_to_act_steps.max | 1 | 1 | 1 |
| first-conformance-run/desktop-1280 | glance_cost.median | 1 | 1 | 1 |
| first-conformance-run/desktop-1280 | glance_cost.p95 | 2 | 2 | 2 |
| first-conformance-run/desktop-1280 | reading_load.max | 190 | 190 | 190 |
| first-conformance-run/desktop-1280 | reading_load.median | 189 | 189 | 189 |
| first-conformance-run/desktop-1280 | completion_miss | 0 | 0 | 0 |
| first-conformance-run/desktop-1280 | false_complete | 0 | 0 | 0 |
| first-conformance-run/desktop-1280/keyboard | screen_only.actions | 23 | 23 | 23 |
| first-conformance-run/desktop-1280/keyboard | screen_only.looks | 2 | 2 | 2 |
| first-conformance-run/desktop-1280/keyboard | screen_only.steers | 0 | 0 | 0 |
| first-conformance-run/desktop-1280/keyboard | screen_only.time_to_act_steps.max | 1 | 1 | 1 |
| first-conformance-run/desktop-1280/keyboard | glance_cost.median | 3 | 3 | 3 |
| first-conformance-run/desktop-1280/keyboard | glance_cost.p95 | 3 | 3 | 3 |
| first-conformance-run/desktop-1280/keyboard | reading_load.max | 190 | 190 | 190 |
| first-conformance-run/desktop-1280/keyboard | reading_load.median | 189 | 189 | 189 |
| first-conformance-run/desktop-1280/keyboard | completion_miss | 0 | 0 | 0 |
| first-conformance-run/desktop-1280/keyboard | false_complete | 0 | 0 | 0 |
| first-conformance-run/mobile-390 | screen_only.actions | 4 | 4 | 4 |
| first-conformance-run/mobile-390 | screen_only.looks | 2 | 2 | 2 |
| first-conformance-run/mobile-390 | screen_only.steers | 2 | 2 | 2 |
| first-conformance-run/mobile-390 | screen_only.time_to_act_steps.max | 2 | 2 | 2 |
| first-conformance-run/mobile-390 | glance_cost.median | 3 | 3 | 3 |
| first-conformance-run/mobile-390 | glance_cost.p95 | 3 | 3 | 3 |
| first-conformance-run/mobile-390 | reading_load.max | 108 | 108 | 108 |
| first-conformance-run/mobile-390 | reading_load.median | 99 | 99 | 99 |
| first-conformance-run/mobile-390 | completion_miss | 0 | 0 | 0 |
| first-conformance-run/mobile-390 | false_complete | 0 | 0 | 0 |
| first-conformance-run/mobile-390/keyboard | screen_only.actions | 130 | 130 | 130 |
| first-conformance-run/mobile-390/keyboard | screen_only.looks | 2 | 2 | 2 |
| first-conformance-run/mobile-390/keyboard | screen_only.steers | 2 | 2 | 2 |
| first-conformance-run/mobile-390/keyboard | screen_only.time_to_act_steps.max | 2 | 2 | 2 |
| first-conformance-run/mobile-390/keyboard | glance_cost.median | 3 | 3 | 3 |
| first-conformance-run/mobile-390/keyboard | glance_cost.p95 | 3 | 3 | 3 |
| first-conformance-run/mobile-390/keyboard | reading_load.max | 108 | 108 | 108 |
| first-conformance-run/mobile-390/keyboard | reading_load.median | 99 | 99 | 99 |
| first-conformance-run/mobile-390/keyboard | completion_miss | 0 | 0 | 0 |
| first-conformance-run/mobile-390/keyboard | false_complete | 0 | 0 | 0 |
| go-runner/desktop-1280 | screen_only.actions | 2 | 2 | 2 |
| go-runner/desktop-1280 | screen_only.looks | 2 | 2 | 2 |
| go-runner/desktop-1280 | screen_only.steers | 0 | 0 | 0 |
| go-runner/desktop-1280 | screen_only.time_to_act_steps.max | 1 | 1 | 1 |
| go-runner/desktop-1280 | glance_cost.median | 2 | 2 | 2 |
| go-runner/desktop-1280 | glance_cost.p95 | 2 | 2 | 2 |
| go-runner/desktop-1280 | reading_load.max | 199 | 199 | 199 |
| go-runner/desktop-1280 | reading_load.median | 190 | 190 | 190 |
| go-runner/desktop-1280 | completion_miss | 0 | 0 | 0 |
| go-runner/desktop-1280 | false_complete | 0 | 0 | 0 |
| go-runner/mobile-390 | screen_only.actions | 3 | 3 | 3 |
| go-runner/mobile-390 | screen_only.looks | 2 | 2 | 2 |
| go-runner/mobile-390 | screen_only.steers | 1 | 1 | 1 |
| go-runner/mobile-390 | screen_only.time_to_act_steps.max | 2 | 2 | 2 |
| go-runner/mobile-390 | glance_cost.median | 2 | 2 | 2 |
| go-runner/mobile-390 | glance_cost.p95 | 3 | 3 | 3 |
| go-runner/mobile-390 | reading_load.max | 145 | 145 | 145 |
| go-runner/mobile-390 | reading_load.median | 99 | 99 | 99 |
| go-runner/mobile-390 | completion_miss | 0 | 0 | 0 |
| go-runner/mobile-390 | false_complete | 0 | 0 | 0 |
| install-linux-x86_64/desktop-1280 | screen_only.actions | 2 | 2 | 2 |
| install-linux-x86_64/desktop-1280 | screen_only.looks | 1 | 1 | 1 |
| install-linux-x86_64/desktop-1280 | screen_only.steers | 0 | 0 | 0 |
| install-linux-x86_64/desktop-1280 | screen_only.time_to_act_steps.max | 1 | 1 | 1 |
| install-linux-x86_64/desktop-1280 | glance_cost.median | 1 | 1 | 1 |
| install-linux-x86_64/desktop-1280 | glance_cost.p95 | 2 | 2 | 2 |
| install-linux-x86_64/desktop-1280 | reading_load.max | 190 | 190 | 190 |
| install-linux-x86_64/desktop-1280 | reading_load.median | 182 | 182 | 182 |
| install-linux-x86_64/desktop-1280 | completion_miss | 0 | 0 | 0 |
| install-linux-x86_64/desktop-1280 | false_complete | 0 | 0 | 0 |
| install-linux-x86_64/desktop-1280/keyboard | screen_only.actions | 21 | 21 | 21 |
| install-linux-x86_64/desktop-1280/keyboard | screen_only.looks | 2 | 2 | 2 |
| install-linux-x86_64/desktop-1280/keyboard | screen_only.steers | 0 | 0 | 0 |
| install-linux-x86_64/desktop-1280/keyboard | screen_only.time_to_act_steps.max | 1 | 1 | 1 |
| install-linux-x86_64/desktop-1280/keyboard | glance_cost.median | 3 | 3 | 3 |
| install-linux-x86_64/desktop-1280/keyboard | glance_cost.p95 | 3 | 3 | 3 |
| install-linux-x86_64/desktop-1280/keyboard | reading_load.max | 190 | 182 | 190 |
| install-linux-x86_64/desktop-1280/keyboard | reading_load.median | 182 | 182 | 182 |
| install-linux-x86_64/desktop-1280/keyboard | completion_miss | 0 | 0 | 0 |
| install-linux-x86_64/desktop-1280/keyboard | false_complete | 0 | 0 | 0 |
| install-linux-x86_64/mobile-390 | screen_only.actions | 4 | 4 | 4 |
| install-linux-x86_64/mobile-390 | screen_only.looks | 2 | 2 | 2 |
| install-linux-x86_64/mobile-390 | screen_only.steers | 2 | 2 | 2 |
| install-linux-x86_64/mobile-390 | screen_only.time_to_act_steps.max | 2 | 2 | 2 |
| install-linux-x86_64/mobile-390 | glance_cost.median | 3 | 3 | 3 |
| install-linux-x86_64/mobile-390 | glance_cost.p95 | 3 | 3 | 3 |
| install-linux-x86_64/mobile-390 | reading_load.max | 120 | 120 | 120 |
| install-linux-x86_64/mobile-390 | reading_load.median | 99 | 99 | 99 |
| install-linux-x86_64/mobile-390 | completion_miss | 0 | 0 | 0 |
| install-linux-x86_64/mobile-390 | false_complete | 0 | 0 | 0 |
| install-linux-x86_64/mobile-390/keyboard | screen_only.actions | 128 | 128 | 128 |
| install-linux-x86_64/mobile-390/keyboard | screen_only.looks | 2 | 2 | 2 |
| install-linux-x86_64/mobile-390/keyboard | screen_only.steers | 2 | 2 | 2 |
| install-linux-x86_64/mobile-390/keyboard | screen_only.time_to_act_steps.max | 2 | 2 | 2 |
| install-linux-x86_64/mobile-390/keyboard | glance_cost.median | 3 | 3 | 3 |
| install-linux-x86_64/mobile-390/keyboard | glance_cost.p95 | 3 | 3 | 3 |
| install-linux-x86_64/mobile-390/keyboard | reading_load.max | 120 | 120 | 120 |
| install-linux-x86_64/mobile-390/keyboard | reading_load.median | 99 | 99 | 99 |
| install-linux-x86_64/mobile-390/keyboard | completion_miss | 0 | 0 | 0 |
| install-linux-x86_64/mobile-390/keyboard | false_complete | 0 | 0 | 0 |
| limitations/desktop-1280 | screen_only.actions | 2 | 2 | 2 |
| limitations/desktop-1280 | screen_only.looks | 1 | 1 | 1 |
| limitations/desktop-1280 | screen_only.steers | 0 | 0 | 0 |
| limitations/desktop-1280 | screen_only.time_to_act_steps.max | 1 | 1 | 1 |
| limitations/desktop-1280 | glance_cost.median | 1 | 1 | 1 |
| limitations/desktop-1280 | glance_cost.p95 | 2 | 2 | 2 |
| limitations/desktop-1280 | reading_load.max | 165 | 165 | 165 |
| limitations/desktop-1280 | reading_load.median | 164 | 164 | 164 |
| limitations/desktop-1280 | completion_miss | 0 | 0 | 0 |
| limitations/desktop-1280 | false_complete | 0 | 0 | 0 |
| limitations/mobile-390 | screen_only.actions | 4 | 4 | 4 |
| limitations/mobile-390 | screen_only.looks | 2 | 2 | 2 |
| limitations/mobile-390 | screen_only.steers | 2 | 2 | 2 |
| limitations/mobile-390 | screen_only.time_to_act_steps.max | 2 | 2 | 2 |
| limitations/mobile-390 | glance_cost.median | 3 | 3 | 3 |
| limitations/mobile-390 | glance_cost.p95 | 3 | 3 | 3 |
| limitations/mobile-390 | reading_load.max | 106 | 106 | 106 |
| limitations/mobile-390 | reading_load.median | 86 | 86 | 86 |
| limitations/mobile-390 | completion_miss | 0 | 0 | 0 |
| limitations/mobile-390 | false_complete | 0 | 0 | 0 |
| page-a-view/desktop-1280 | screen_only.actions | 4 | 4 | 4 |
| page-a-view/desktop-1280 | screen_only.looks | 3 | 3 | 3 |
| page-a-view/desktop-1280 | screen_only.steers | 0 | 0 | 0 |
| page-a-view/desktop-1280 | screen_only.time_to_act_steps.max | 1 | 1 | 1 |
| page-a-view/desktop-1280 | glance_cost.median | 2 | 2 | 2 |
| page-a-view/desktop-1280 | glance_cost.p95 | 2 | 2 | 2 |
| page-a-view/desktop-1280 | reading_load.max | 230 | 230 | 230 |
| page-a-view/desktop-1280 | reading_load.median | 214.5 | 214.5 | 214.5 |
| page-a-view/desktop-1280 | completion_miss | 0 | 0 | 0 |
| page-a-view/desktop-1280 | false_complete | 0 | 0 | 0 |
| page-a-view/mobile-390 | screen_only.actions | 11 | 11 | 11 |
| page-a-view/mobile-390 | screen_only.looks | 4 | 4 | 4 |
| page-a-view/mobile-390 | screen_only.steers | 8 | 8 | 8 |
| page-a-view/mobile-390 | screen_only.time_to_act_steps.max | 6 | 6 | 6 |
| page-a-view/mobile-390 | glance_cost.median | 3 | 3 | 3 |
| page-a-view/mobile-390 | glance_cost.p95 | 3 | 3 | 3 |
| page-a-view/mobile-390 | reading_load.max | 147 | 147 | 147 |
| page-a-view/mobile-390 | reading_load.median | 104.5 | 104.5 | 104.5 |
| page-a-view/mobile-390 | completion_miss | 0 | 0 | 0 |
| page-a-view/mobile-390 | false_complete | 0 | 0 | 0 |
| target-process/desktop-1280 | screen_only.actions | 5 | 5 | 5 |
| target-process/desktop-1280 | screen_only.looks | 2 | 2 | 2 |
| target-process/desktop-1280 | screen_only.steers | 3 | 3 | 3 |
| target-process/desktop-1280 | screen_only.time_to_act_steps.max | 4 | 4 | 4 |
| target-process/desktop-1280 | glance_cost.median | 2 | 2 | 2 |
| target-process/desktop-1280 | glance_cost.p95 | 3 | 3 | 3 |
| target-process/desktop-1280 | reading_load.max | 252 | 252 | 252 |
| target-process/desktop-1280 | reading_load.median | 213 | 213 | 213 |
| target-process/desktop-1280 | completion_miss | 0 | 0 | 0 |
| target-process/desktop-1280 | false_complete | 0 | 0 | 0 |
| target-process/mobile-390 | screen_only.actions | 12 | 12 | 12 |
| target-process/mobile-390 | screen_only.looks | 2 | 2 | 2 |
| target-process/mobile-390 | screen_only.steers | 9 | 9 | 9 |
| target-process/mobile-390 | screen_only.time_to_act_steps.max | 8 | 8 | 8 |
| target-process/mobile-390 | glance_cost.median | 3 | 3 | 3 |
| target-process/mobile-390 | glance_cost.p95 | 3 | 3 | 3 |
| target-process/mobile-390 | reading_load.max | 122 | 122 | 122 |
| target-process/mobile-390 | reading_load.median | 113 | 113 | 113 |
| target-process/mobile-390 | completion_miss | 0 | 0 | 0 |
| target-process/mobile-390 | false_complete | 0 | 0 | 0 |
| theme-dark-and-back/desktop-1280 | screen_only.actions | 3 | 3 | 3 |
| theme-dark-and-back/desktop-1280 | screen_only.looks | 0 | 0 | 0 |
| theme-dark-and-back/desktop-1280 | screen_only.steers | 0 | 0 | 0 |
| theme-dark-and-back/desktop-1280 | screen_only.time_to_act_steps.max | 1 | 1 | 1 |
| theme-dark-and-back/desktop-1280 | glance_cost.median | 1 | 1 | 1 |
| theme-dark-and-back/desktop-1280 | glance_cost.p95 | 1 | 1 | 1 |
| theme-dark-and-back/desktop-1280 | reading_load.max | 215 | 215 | 215 |
| theme-dark-and-back/desktop-1280 | reading_load.median | 215 | 215 | 215 |
| theme-dark-and-back/desktop-1280 | completion_miss | 0 | 0 | 0 |
| theme-dark-and-back/desktop-1280 | false_complete | 0 | 0 | 0 |
| theme-dark-and-back/mobile-390 | screen_only.actions | 4 | 4 | 4 |
| theme-dark-and-back/mobile-390 | screen_only.looks | 2 | 2 | 2 |
| theme-dark-and-back/mobile-390 | screen_only.steers | 1 | 1 | 1 |
| theme-dark-and-back/mobile-390 | screen_only.time_to_act_steps.max | 2 | 2 | 2 |
| theme-dark-and-back/mobile-390 | glance_cost.median | 2.5 | 2.5 | 2.5 |
| theme-dark-and-back/mobile-390 | glance_cost.p95 | 3 | 3 | 3 |
| theme-dark-and-back/mobile-390 | reading_load.max | 194 | 194 | 194 |
| theme-dark-and-back/mobile-390 | reading_load.median | 174.5 | 174.5 | 174.5 |
| theme-dark-and-back/mobile-390 | completion_miss | 0 | 0 | 0 |
| theme-dark-and-back/mobile-390 | false_complete | 0 | 0 | 0 |

## Measured-only viewports

| viewport | pages measured | with page scroll | with a spill | with an always-visible element out of view, covered or missing (in the states gate 2 gates) | truncated texts |
| --- | --- | --- | --- | --- | --- |
| desktop-1440 | 139 of 139 | 0 | 0 | 4 | 0 |
| desktop-1024 | 139 of 139 | 0 | 0 | 2 | 0 |
| tablet-768 | 139 of 139 | 0 | 0 | 0 | 0 |

## Worst moments

- /ess/docs/visualise at mobile-390, light: axe color-contrast×8, label-content-name-mismatch×1, nested-interactive×1, scrollable-region-focusable×1; 1 truncated (magenta outline: the first axe nodes)
  ![worst-1-mobile-390-light.png](shots/worst-1-mobile-390-light.png)
- /ess/docs/getting-started at desktop-1280, dark: always-visible: docs sidebar out of viewport at middle (magenta outline: the first axe nodes)
  ![worst-2-desktop-1280-dark.png](shots/worst-2-desktop-1280-dark.png)
- /ess/releases/tags at desktop-1280, dark: always-visible: blog sidebar out of viewport at middle (magenta outline: the first axe nodes)
  ![worst-3-desktop-1280-dark.png](shots/worst-3-desktop-1280-dark.png)

## Contact sheets

![contact sheet, light theme](shots/contact-light.png)

![contact sheet, dark theme](shots/contact-dark.png)

## Missing inputs

- L6 recorded live rounds and the questionnaire: no person took part in this round.
- Recordings (rrweb): none; the screen-only user's states are in the runs' raw.json.
- L4 keyed replay, L5 discoverability probe, L7 review agents: run by the coordinator after this report.

## Notes of the round (written by hand after the report stage)

- Host load. The machine ran at a 1-minute load of 40 to 165 on 20 cores during the round (other
  sessions' builds). The gate runs `gate-4` to `gate-6` took 119.6 (cut at the 120 s budget), 100.7
  and 93.6 s at load 61 to 92. A same-load comparison at load 41 to 48 ran the gate's L2 and L3
  projects in 35.8 and 36.4 s with the R-02 engine and 45.2 s with this one; L1 took 10 to 21 s,
  most of it the three nested Playwright runs of the known-failure tests. Before the last engine
  commit, three runs at load 105 were cut at 119 s, and one without the cut took 220.7 s.
- Reruns. The heavy reference pages (`cli`, `ess-ui`, `formats`) ran into the 60 s test bound or
  the hydration wait at load 90 to 160; `rerun-*` measured them again, each replacing only its own
  configurations. `ess-ui` passed at load 49 (47 and 45 s). `rerun-essui` and `rerun-shift` ran
  from a copy of the engine at `7db25740980e` outside the Git tree (same engine files, digest
  `bc3e06f44faa130b`, so commit "unknown"), because the tree had moved on to a fix of the round
  stage's test selection that does not touch measurement.
- `/ess/releases/outcome-shapes@mobile-390/dark` measured `layout_shift.load` 0.0253 in `slow-l2-4`
  (baseline 0) and 0.00004 in `rerun-shift`; the rerun's value is in `round.json`.
- Known failures. Six open findings named one regression asset where their failure is at both
  gating viewports or on two pages (UXF-006, UXF-010 to UXF-013, UXF-022), so the first runs showed
  those tests as new failures; the findings name every such test now (a finding may name several).
  UXF-023 is closed as not reproduced: `/ess/` reports no `color-contrast` node.
