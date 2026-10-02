---
format: aep.planning-md/3
id: review-result:consumer-one-time-history-cli
kind: review-result
status: active
title: Independent model-aware history CLI review
relations:
- reviews: story:feature-request-389
revision: 1
---
approve

Independent source-only review; own test/build executions0. Verified frozen one-time-history-cli-review.patch SHA256 b9b3ec6f00108269bceb7103e1a10ecb38ab299b1c5abd60af5e367ca45448f4 and crates/edge/ess-cli/tests/one_time_history.rs SHA25695b952a75a43c058f879d03f9e77d74e1aa2077ba4811a84dd4ade7a272f7cce in ess-backlog-next-20261002.

No concrete defect found in this bounded CLI pre-read guard. The import-history dispatcher at main.rs:3185 passes path arguments to import_history without first opening adapter/log inputs. After resolving the source model, main.rs:3256 scans all commands and all outcomes for a nonempty one_time_response list. A marked model returns exit2 with one static diagnostic before output collision inspection, either fs::read call, adapter decoding, recorded::import_for, gap reporting or output writing. The diagnostic contains no input path, log value or adapter content. The unmarked path follows the existing implementation unchanged.

The regression covers both missing inputs and existing malformed sentinel bytes, asserting the named refusal, empty stdout, no private sentinel in stderr, preservation of an existing destination and absence of the gaps sidecar. The unmarked control requires the original adapter-unreadable diagnostic, so the refusal cannot be implemented by indiscriminately blocking history import. Reported producer red1pass1fail and green2pass0fail are not attributed to this reviewer's own execution.

This approves the CLI boundary only. Native record/session/import_for guards and replacing the identical local model predicate with the native owner's shared helper remain separate integration work. The review does not claim a new replayable history representation or support for recording one-time disclosures.

```findings
[]
```
