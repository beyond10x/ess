---
format: aep.planning-md/3
id: story:external-results-become-a-report
kind: story
status: implemented
title: A runner's per-scenario results become a canonical conformance report
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T03:18:18Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-30T03:18:19Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-30T10:30:12Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":2,"review_outcome":1,"verification":1}}}
---
## Outcome

A runner written in any language records a canonical `ess-conformance-report/2` for a suite
ESS admits, by handing ESS its per-scenario results; the report says the results were supplied by
the runner, so evidence built on it cannot be mistaken for a run ESS executed.

## Acceptance

- `ess verify conform report --suite <suite.json> --results <results.json> --implementation <name>
  --report-out <path>` admits the suite (any format ESS admits, e.g. `ess-conformance/26`), reads a
  results document (`{format: ess-conformance-results/1, completed_at, results: [{scenario_id,
  status: passed|failed|error|unsupported, message?}]}`), and writes report/2 with coverage, suite
  reference and policy computed from ESS's own admission of the suite.
- `producer_profile` records that the results were supplied externally (runner name and version if
  given); report consumers (`aep plan artifact evidence --from`) keep working.
- Refused, with a message: a result for a scenario the suite does not contain, a scenario of the
  suite with no result, duplicate results, an unknown status, a results document for another suite
  digest (if the results carry one).
- The results format is documented and tracked (`FORMAT_RELEASES`, formats.md).

## Origin

A downstream implementation's hand-written runner passes its 670-scenario suite but cannot produce
report/2: report/1 refuses its suite format, and coverage, suite reference and policy come from
ESS's admission of the suite.
