---
format: aep.planning-md/3
id: task:retained-replay-fixture-consistency
kind: task
status: active
title: Make retained Go replay fixtures honor read-your-writes tokens
relations:
- derived_from: story:feature-request-312
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:50:01Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-04T00:50:01Z", actor: "human:timo", revision: 3}
---
## Finding and authorization

The accepted bundle's full ess-conformance run on source2255315a4 plus the two-path413A arithmetic patch terminated101 after1128.553625seconds:2609passed,2failed,8ignored,0filtered across337test-result summaries. Only retained_replay failed:34passed,2failed. Its actual generated Go execution reported153pass events,33fail events,0skips; these are nested events, not33independent Rust failures. Terminal logSHAb4ff23325419aaf72d5b84c5247c5b1bc28a007106612e1c80c469ad171c1b0b; the two failing Rust cases are adversary_go_replay_requires_complete_actual_subject_rows and generated_go_replay_runtime_executes_actual_results_and_strict_admission. Strict all-target lint and both format checks passed. The package is red, not integration evidence.

Source diagnosis3fbdda1cd7eb170de53812e9ec9ee71f0452a81e8fce3aeed83d7ec07f64e7ba identifies a concrete contract mismatch: native Backend returns actual-write, while both corresponding Go retainedFixture results omit Consistency despite the model's read_your_writes view. The runtime then correctly suppresses a weaker Current query. This is the leading source-supported explanation; the token-only counterfactual must execute before declaring the fix proven. The diagnosis's whole-log hash was taken while the command was running; the terminal digest above supersedes it for whole-run accounting. The retained failure slice and33-event inventory1249af716974b7e9459b67ac9852f6edb2b015f53c6272f774013edf7aa4c3f5 remain valid.

This is a newly exposed fixture compatibility correction under the accepted312 bundle work. The closed RYW implementation's two reviews remain closed. No production runtime relaxation, third review of that closed unit, arithmetic blame or broader source change is authorized.

## Scope and deciding probes

Only crates/verify/ess-conformance/tests/fixtures/retained-replay-runtime.go may change. Preserve the failed generated packages unchanged. In independent outside-Git copies, first add actual-write to only the seeded and replayed result constructors; run the actual adversary snapshot and complete retained Go packages. Then add the exact request-token check to QueryView and run those same packages. Check all inherited/wrapper routes before choosing the guard so legitimate existing token producers retain their contracts. A failure after tokens are corrected is a new finding to report, not a reason to weaken assertions. Rust remains mandatory for new committed executable tools; the existing generated-runtime Go fixture is the accepted target test surface.

No probe or build starts below12884901888free bytes. Only one ESS execution lane is admitted, jobs1, locked/offline, Rust1.98.1, no wrappers/incremental/debug payload, outside-Git TMP and a unit-exclusive Go cache. The initial assignment is read-only/source preparation until explicit execution custody is granted. No cleanup, publication, AEP write or source integration belongs to the implementor.

## Acceptance

Both failed Rust wrappers and the whole retained_replay target must pass with actual Go execution, no skipped required checks and no assertion removed. Recheck negative controls that previously passed from the unrelated early token failure; complete/incomplete rows, replay mutation, exact integers and retained input callbacks must reach their intended assertions. Exact token propagation must be exercised on original and replay reads. Run strict affected-target lint, owning formatter and task fmt-check, then obtain independent whole-unit review. Retain baseline, treatment, source and execution digests. Integrate serially before refreshing413A; the final full package run is still required on the combined corrected bytes. No issue closure or release claim follows from this task alone.
