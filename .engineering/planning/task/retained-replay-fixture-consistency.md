---
format: aep.planning-md/3
id: task:retained-replay-fixture-consistency
kind: task
status: active
title: Make retained Go replay fixtures honor read-your-writes tokens
relations:
- derived_from: story:feature-request-312
- serves: vision:O2
revision: 5
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

## Execution custody admitted after owned cache recovery

The completed413A run's owner verified and retired only its exclusive Go compiler cache through go clean -cache. Receipt19a43826cb1b086233b9d5a425e05f255a39cb42da1ce62a5792fc06c08134c1 records2109902848allocated bytes reclaimed, unchanged source/logs/fixtures/binaries/archives, and an honest limited process audit. The user's three read-only release/backlog candidates were untouched. Fresh coordinator available space13452419072bytes is above the12884901888floor; this is an admission observation, not a forecast.

The existing implementor now owns the sole ESS execution lane. Before each expensive command, record another fresh floor receipt. Preserve a command inventory and immutable original failure packages. Run exactly the adversary projection and full retained generated Go packages after the two-result-token change, then the same two after the exact query-token assertion; each command is bounded180seconds. Only after those counterfactual probes pass, apply the validated one-file correction and run the whole retained_replay Rust target once, strict affected-target lint, owning format and task fmt-check. Rust target/lint commands are bounded900seconds. No redundant separate reruns of the two Rust wrappers and no whole-package run are assigned here. Unexpected failure or insufficient capacity holds the next start. Independent whole-unit review and integration still precede the refreshed413A full-package run. No acceptance success is claimed at dispatch.

## Frozen candidate and proven Go counterfactuals

Local bot candidate8e7c21a0c5055635e771c2c5b2b4fa9181bc909e, tree5de15cb4deb3c32dbaa2020f264471a8f4fc9332, parent3f5b7524633cfb20f178887e2c1a5f54d9eb313b, changes exactly the scoped fixture by6additions/3removals. PatchSHAa1040c6ac78ed660f554232d53dc475f561130e5b1711a001a4449681eff9ba3; fixtureSHA2fe25f862f9256caf4c0f529e3646456bf867ac483206f5c9bd5a1d8e9106699. Author and committer are both the bot. This is a pending candidate, not accepted integration.

The token-only counterfactual passed both actual Go commands: exact adversary projection3pass/0fail/0skip (loge9cd0764b777363aaea9b4cc5d32128e67bb06618ce721babf60824604119314) and whole generated package186pass/0fail/0skip (log2c2f26a8e3b1e683e7ad90cd418b95fea3c94566ddce448a74d2f26fd7f2f726). Adding the exact request-token assertion also passed3/0/0 (log4edd5e74341b709570489e3a047c88d570ab507f57f30dd09a3b1f9c3563f651) and186/0/0 (log80f4c2009c50f4ab0672b831792a117d7d72f82d3e8335ef706bbcbc75cc4e59). Root independently counted the raw JSON events. Original generated failures remain immutable, and the repository fixture byte-matches the final probe. These results prove the token mismatch caused the retained Go failures; the generated runtime itself was unchanged. Owning format and task fmt-check passed.

Whole retained_replay Rust wrapper execution and strict affected-target lint have NOT started: freshfree again fell below12884901888bytes. No native target exists for this unit. Review1 preparation and source audit are pending final author checks; no verdict or review-budget reset is claimed. The existing exclusive exact-request-token Go cache is retained for the later Rust wrapper's generated execution. Source-clean managed tree and live author lease remain handed to the same implementor; no publication or integration.
