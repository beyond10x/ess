---
format: aep.planning-md/3
id: review-result:adversary-ui-test-pass-1
kind: review-result
status: active
title: Adversary pass 1, ess-ui wave unit ui-spec-test-language
relations:
- reviews: story:ui-spec-test-language
revision: 1
---
unit: story:ui-spec-test-language, tree ess-ui-test (branch impl/ui-test)
verdict: NEEDS-CHANGE, red 5
cases: executed 27→33, red 5
origin: introduced 4 / pre-existing 1

| file:line | severity | message |
|---|---|---|
| ess-ui-test/src/runner.rs:812 | blocker | rows recognised by unique shown values; identical rows not counted |
| ess-ui-test/src/runner.rs:574 | blocker | text at a column/child/action path read the whole row, box or screen |
| ess-ui-test/src/playwright.rs:207 | warning | page to N clicked next N-1 times from the current page |
| ess-ui-tui/src/live.rs:157 | warning | duration parse multiplied unchecked (pre-existing) |
| ess-ui-test/src/runner.rs:123 | warning | advance added durations unchecked; looping scripts exhausted memory |
| ess-ui-test/src/lib.rs:197 | note | run directories left behind on panic |
| ess-ui-test/src/playwright.rs:158 | note | select on a row emitted a click |
| ess-ui-test/src/playwright.rs:124 | note | view route prefix-matched other views |

Tests: crates/ui/ess-ui-test/tests/adversary_pass1.rs.

## Correction 1

Rows by row key through the read-only ess-ui-tui App::regions(); text scoped to the node's own cells; absolute page; checked durations and MAX_CYCLES_PER_ADVANCE = 10000; TempDir run directories; select focuses; exact view routes. 85 passed, 0 failed. The coordinator added data-ui-path on React column headers.
