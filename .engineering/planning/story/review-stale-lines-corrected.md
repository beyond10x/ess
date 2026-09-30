---
format: aep.planning-md/3
id: story:review-stale-lines-corrected
kind: story
status: draft
title: Stale lines found by the fit review are corrected
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

Documentation lines the review found contradicting shipped behaviour are corrected (review section 3).

## Acceptance

- CO:156, CO:220, GP:139-140 and CL:25 ("unreleased" for a released format) state shipped behaviour
- `ess-gen/src/http.rs:93-100` (`UNFINISHED`) describes every case it now covers

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (review-stale-lines-corrected). Drafted, not scheduled; a fit review precedes dispatch.
