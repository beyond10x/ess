---
format: aep.planning-md/3
id: story:ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views
kind: story
status: draft
title: ess-http/1 admits text bodies, route refusals, empty bodies and array views
summary: 'Adopter reports N21, N26, N27, N29: four answer forms refused by ess-http/1 (ess#673).'
refs:
- provider: github
  reference: beyond10x/ess#673
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

`ess-http/1` refuses four answer forms an adopter binding served operations needs: a text body (N21, `ESS-HTTP-024`), a per-route `refusals:` (N26), a body on `empty` (N27) and a view answered as a bare array (N29, `ESS-HTTP-024`). Each was a narrow stage-2 choice in `docs/design/http-presentation-binding.md` ("Still refused", answers 17 and 20, `rows`/`single` only), recorded there as relaxable without a format change. ess#673, ess 0.68.0.

## Acceptance

- `a_command_outcome_answers_a_text_body`: `body: text` validates, projects a `text/plain` string response in OpenAPI, and conformance reads the body as text.
- `a_route_refusal_overrides_the_binding_refusal`: a route-level `refusals: {not_granted: ...}` answers that route with its own status and body; other routes keep the top-level answer.
- `an_empty_answer_may_carry_a_body`: `empty: {status: 404, body: ...}` validates and projects; `ESS-HTTP-025` still refuses two outcomes answered alike.
- `a_view_route_answers_a_bare_array`: `body: array` answers the rows as a top-level JSON array in OpenAPI, the generated route and conformance.
- `the_design_page_records_the_relaxation`: "Still refused", answers 17 and 20 and the view answer table are amended; the bytes of every existing binding fixture are unchanged.

Related: `story:a-view-answer-takes-its-protocols-list-shape` (rows member renamed, cursor paging) stays separate.
