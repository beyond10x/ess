---
format: aep.planning-md/3
id: review-result:adversary-ui-test-pass-2
kind: review-result
status: active
title: Adversary pass 2, ess-ui wave unit ui-spec-test-language
relations:
- reviews: story:ui-spec-test-language
revision: 1
---
unit: story:ui-spec-test-language, tree ess-ui-test (branch impl/ui-test, after correction 1)
verdict: NEEDS-CHANGE, red 6
cases: executed 123→130, red 6
origin: introduced 6

| file:line | severity | message |
|---|---|---|
| ess-ui-react collection.tsx.tmpl:144 | warning | terminal recorded cells for a cards collection with item; React renders none |
| ess-ui-react collection.tsx.tmpl:194 | warning | header path columns/<name> for selectable columns, canonical columns/all/<name> |
| ess-ui-tui/src/view.rs:927 | warning | text at a cell compared the cut terminal cell |
| ess-ui-test/src/runner.rs:614 | warning | text at a row action refused in the terminal, checked in the browser |
| ess-ui-test/src/playwright.rs:328 | note | a move past the cycle limit emitted as a plain runFor |
| ess-ui-test/src/playwright.rs:329 | note | spec generator panicked on a move past the end of time |

Tests: crates/ui/ess-ui-test/tests/adversary_pass2.rs.

## Correction 2 (coordinator-verified)

Wherever the terminal refuses a step, the Playwright spec marks the test test.fixme with the same reason (parity.rs); no cell regions for collections with item; header paths from the column's own data-ui-path; Region.text carries the full value; row-action regions carry labels; checked clock shared by runner and spec. Coordinator rerun: cargo test -p ess-ui-test -p ess-ui-tui -p ess-ui-react 134 passed, 0 failed; clippy -D warnings and fmt --check exit 0.
