---
format: aep.planning-md/3
id: story:a-view-answer-takes-its-protocols-list-shape
kind: story
status: draft
title: 'A view answer takes its protocol''s list shape: rows member, cursor paging, row-shaping parameters, state wire values'
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22). The adopter's list operation answers `{"tasks": [...], "nextPageToken": "…", "pageSize": n, "totalSize": n}`, takes `pageSize` and `pageToken` (cursor paging, required by the protocol), takes `historyLength` and `includeArtifacts`, which shape each row, and writes the task state as `TASK_STATE_*` values. The generated view route answers none of that, so the adopter hand-writes the list route.

## Reproduction

Minimal spec on 0.68.0 (a view over one entity with an optional parameter, bound `get /jobs`, generated with `--target rust --layout crate --http`):

- The answer is always `{"rows": [...]}`: `json::member(&mut body, "rows")` in the generated route. No binding or view key renames the member.
- `paging:` (ess/16) is by page number and size and answers no continuation token, so cursor paging is not expressible.
- The adopter's specification notes that no view field can be omitted by a parameter (`include_artifacts`, `history_length`).
- The lifecycle state is written by its state name (`"Done"`, `"Open"`). Not tested: whether a lifecycle state takes a `wire:` name the way an enum variant does since `ess/5`.
- Wire names of view fields and parameters do work: `wire: ownerName` is read and written as `ownerName` (see `story:a-view-parameter-carries-its-wire-name-on-every-http-reader`).

## Where it lives

- The `rows` member: `crates/generate/ess-synth/src/rust/http.rs:1413`, traced through the generated output.
- Paging: `crates/specify/ess-domain/src/view/paging.rs:39-49` (`Paging { page, size, first_page, total }`), traced.
- Row-shaping parameters and state wire values: inferred, not located.

## Acceptance (provisional)

- An `ess-http/1` view route may name the member that carries the rows and the members that carry paging facts.
- A view may declare cursor paging, a page size and an opaque continuation token, answered beside the rows; conformance checks that following the tokens reads every row once.
- A view parameter may omit or truncate a field of each row, and conformance holds it.
- A lifecycle state may carry a wire value, used by the view row, view parameters and the event payloads.
- Split this story before dispatch: these are four constructs.
