---
format: aep.planning-md/3
id: approval-record:g-u33-row-shape-in-0700
kind: approval-record
status: approved
title: G-U33 ships the hint; the row shape check moves to 0.70.0
summary: 'Option D: ESS-AUTHOR-042 hint in 0.69.0; view-read row shape as its own story with a design page for 0.70.0.'
relations:
- decides: story:a-view-read-checks-its-rows-shape
- decides: story:a-view-read-checks-every-rows-shape
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-23T17:12:25Z", actor: "human:timo", revision: 2}
---
## Decision

Relayed by the orchestrator on 2026-10-23, option D: "Line 1 (ESS-AUTHOR-042 hint) ships in 0.69.0; line 2 (row shape check on view reads) becomes a story plus design page for 0.70.0. Reason: 0.69.0 gates loom 0.40.0 on the ADR 0098 path; same split as the ess-diff line."

Options weighed: A, a shape on every view read (about 128 pinned fixtures move); B, only view reads from `ess/31` specifications (a new suite format pair and row checks in the Rust, Go and TypeScript runners); C, only fields filled by a minted value (partial); D, the split above.

## Effect

`story:a-view-read-checks-its-rows-shape` lands in 0.69.0 with its hint line only. `story:a-view-read-checks-every-rows-shape` holds the row shape check, draft, tagged ess-0.70.0, and needs a design page under `docs/design/` before code.

Approved by session b10x-chef-orchestrator under its grant.
