---
format: aep.planning-md/3
id: story:metric-error-state-on-failed-read
kind: story
status: active
title: A view answered with an error renders as an error state in a metric, never as 0
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/metric.tsx.tmpl
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:04Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:05Z", actor: "human:timo", revision: 4}
---

Every claim is marked verified (seen in a run or in the file named) or inferred.

## Observed

- A header metric over a view (`component: metric`, `reads: {view: ...}`, `aggregate: count` or `sum`) renders `0` when the served view answers 501, 404 or 500. Verified in headless Chrome against a static server that answers the view route with a chosen status (ess 0.90.0, `transcript.txt`):

  | view answers | "Items on the shelf" (count) | "Units on the shelf" (sum of qty) |
  |---|---|---|
  | 200, two rows | 2 | 95 |
  | 501 | 0 | 0 |
  | 404 | 0 | 0 |
  | 500 | 0 | 0 |

- The path, in the generated project (verified, `transcript.txt`):
  - `src/runtime/data.ts:308` the read helper throws `DataError("<status> <view>", status)` on a non-2xx answer.
  - `src/runtime/data.ts:512-513` `useRead` turns it into `{status: "failed" (or "forbidden" for 403), rows: [], error}`.
  - `src/runtime/composites/metric.tsx:21,26-27` `MetricView` takes `data.rows` and never reads `data.status` or `data.error` (the file has no `status` token).
  - `src/runtime/composites/metric.tsx:51` `count` of no rows is `0`; `:57` `sum` of no rows is `0`. `min`, `max`, `avg` and a `match` read give `undefined`, drawn as `–`, which is also not an error state.
- The list section of the same page does draw a failed state (`src/runtime/core.tsx:431-435`, role `alert`, "Could not load." and Retry). Verified. It does not name the status either, although `error` holds `501 store.shelf.Items`.
- Inferred, not run: while a read is `loading`, `rows` is `[]` (`data.ts:483`, `525`), so the same metric draws `0` before the first answer.
- Re-run on 0.92.0: the same table, `transcript-092.txt` (501, 404 and 500 each draw `0` and `0`). Verified. Template lines: `crates/ui/ess-ui-react/templates/runtime/composites/metric.tsx.tmpl:51,57`.

## Expected

A metric whose read has failed or is forbidden draws an error state that names the status (for example `501`), and a metric whose read is loading draws a loading state; neither draws `0` or `–`. A zero is shown only for an answered, empty view.

## Reproduction

1. `sh run.sh` in `.engineering/repro/metric-error-state-on-failed-read/` (ess 0.90.0, node, npm with network, a headless Chrome and `playwright-core`; paths in the script header). It writes `transcript.txt`.
2. Inputs: `spec/` (system `store`, domain `store.shelf`, entities Box and Item, view `store.shelf.Items`), `ui/shelf.ui.yaml` (two header metrics and a list), `ui/fixtures/items.yaml`.
3. `check.mjs` serves `react-out/www`, answers `/shelf/views/Items` with 200, 501, 404, 500 in turn, and prints the metric values and the section alerts.

## Acceptance

For a generated React page whose header metric reads a view, when the served view answers 501 the metric's element contains the text `501` in an alert and not the text `0`, and when it answers 200 with two rows the same metric shows `2`.

## Scope

- cited: crates/ui/ess-ui-react/templates/runtime/composites/metric.tsx.tmpl

confidence: medium
