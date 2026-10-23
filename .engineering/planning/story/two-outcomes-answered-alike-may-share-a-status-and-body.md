---
format: aep.planning-md/3
id: story:two-outcomes-answered-alike-may-share-a-status-and-body
kind: story
status: draft
title: Two outcomes answered alike may share a status and body
summary: 'Adopter report S3: Two outcomes answered alike cannot share a status and body (ess#652).'
refs:
- provider: github
  reference: beyond10x/ess#652
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

ESS-HTTP-025 refuses two outcomes on one status with bodies a reader cannot tell apart. Where the service answers both alike (one status, the bare response), the only valid binding is the `envelope` body, which projects `{outcome, response, published}` the service does not send. Adopter report S3, ess 0.68.0.

## Acceptance

- `outcomes_declared_alike_share_a_status_and_body`: an `ess-http/1` route declaring `fetched` and `inline` answered alike at `201` with the bare response validates, and the projection has one `201` response.
- `outcomes_with_different_responses_are_still_refused`: two outcomes declared alike whose responses differ in shape are refused by name.
- `conformance_accepts_either_outcome_for_the_shared_answer`: a scenario expecting `fetched` passes on an answer that is `201` with the response, and the report says the outcome was not distinguishable on the wire.
- `existing_bindings_are_unchanged`: bindings without the new key validate and project byte for byte as before.
