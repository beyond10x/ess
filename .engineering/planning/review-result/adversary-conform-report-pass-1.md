---
format: aep.planning-md/3
id: review-result:adversary-conform-report-pass-1
kind: review-result
status: active
title: Adversary pass 1, ess verify conform report
relations:
- reviews: story:external-results-become-a-report
revision: 1
---
unit: story:external-results-become-a-report, tree ess-048 (release/0.48.0)
verdict: CONFIRMED, red 2
cases: executed 1918→1924, red 2
origin: introduced 4

| file:line | severity | message |
|---|---|---|
| ess-conformance/src/results.rs:76 | warning | an entry refused for its structure was also reported as MissingResult |
| ess-conformance/src/results.rs:277 | warning | admit_suite depth-limited parse refused a suite ESS admits (INFEASIBLE in practice) |
| ess-cli/src/release_evidence.rs:174 | warning | release qualification printed passed without saying the results were supplied |
| ess-conformance/src/counts.rs:400 | note | CountReport had no producer_profile or runner accessor |
| story external-results-become-a-report | note | Outcome named ess-conformance-run/2; the unit writes report/2 |

Tests: crates/verify/ess-conformance/tests/adversary_results_pass1.rs.

## Correction 1 (coordinator-verified)

All five addressed; the story Outcome now names ess-conformance-report/2. Coordinator rerun: adversary_results_pass1 6 passed, external_results 16, conform_report 3, delivery_trust 42, 0 failed. Implementor gate: ess-conformance 1925 passed, 0 failed; clippy and fmt exit 0.
