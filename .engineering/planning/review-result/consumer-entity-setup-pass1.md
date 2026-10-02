---
format: aep.planning-md/3
id: review-result:consumer-entity-setup-pass1
kind: review-result
status: active
title: Entity setup independent review first pass
relations:
- reviews: story:interpreted-scenario-supplied-facts
revision: 1
---
needs-revision

Mechanical format adaptation of the existing review; no new review or execution. Own test/build executions: 0.

Original report, preserved verbatim:

```json
{
  "verdict": "needs-revision",
  "own_test_or_build_executions": 0,
  "patch_sha256": "18abe42ec3acdac5613eb7f75729e01218ad62a32523958ba8ac2cade2ad2d44",
  "checked_source_manifest_sha256": "51ab895e3e83a08cf9bdd6ebb915ed8510bd278e4f0a48268c6b508a99782e4a",
  "findings": [
    {
      "id": "entity-setup-generated-identity-collision",
      "severity": "blocker",
      "verdict": "NEEDS-CHANGE",
      "title": "An established row can permanently occupy the next generated identity",
      "locations": [
        "crates/verify/ess-conformance/src/interpret/setup.rs:26",
        "crates/verify/ess-conformance/src/interpret/execute.rs:83",
        "crates/verify/ess-conformance/src/interpret/execute.rs:841",
        "crates/verify/ess-conformance/src/interpret/execute.rs:1191"
      ],
      "explanation": "Store::establish inserts a supplied text identity while leaving minted unchanged. With a fresh scenario and a UUID identity, establish 00000000-0000-4000-8000-000000000001, then execute an ordinary creates branch for that entity whose identity is generated. mint uses tick() from zero and returns precisely that UUID. create rejects the already-held identity rather than selecting another fresh generated identity. execute returns Undetermined::Request, and the target never installs the attempted next store, so minted remains zero and repeated creation is stuck on the same collision. This is an introduced interaction with the new setup surface; existing collision rejection is still required for explicitly supplied or recorded identities.",
      "reproduction": "Extend the setup model with an ordinary creator that publishes a generated UUID identity and explicitly sets every required stored field. Begin a scenario; establish the first generated UUID with valid fields; execute the creator twice. Require two fresh distinct generated identities, preservation of the setup row, and three rows total. Also retain a supplied/recorded identity collision refusal control so the fix does not weaken creates uniqueness.",
      "measurement": "Source-traced counterexample only; no test or build was executed by this reviewer. The implementing owner was sent the concrete case for a decisive red probe.",
      "reachability": "The actual ConformanceTarget supports establish_entity followed by execute_command in one active scenario. Authored arrangement supports upstream setup plus command-based arrangement. The checkpoint's positive model explicitly has no commands, so its tests do not cover this combination."
    }
  ],
  "other_assessment": "Verified all five frozen current source hashes. The setup hook checks active correlation and calls the existing complete model validator before mutable borrowing. That validator covers declared entity/state, supplied identity kind, total fields, nested constraints and invariants requiring True. Duplicate rows are not replaced. The insertion then publishes an immediately visible projection snapshot without command/event/invocation fabrication. Begin/end reset clears state and authority. The actual authored two-scenario route has exact row/value/count/order assertions and isolation; direct invalid-input tests use fresh identities to avoid vacuous duplicate refusal. The compiled Integer-identity Unsupported control is honestly described as unfinished typed-store work and is not a finding against the explicitly bounded text-identity checkpoint.",
  "evidence_bounds": "Retained author logs show final-test baseline1/4 and treatment5/0, existing authored13/0 plus scenario-facts5/0 and views6/0. Those are author executions, not independent reviewer evidence. No source/AEP edits or builds were performed. This report does not claim full typed identity support or complete package validation."
}
```

Structured findings:

```json
[
  {
    "file": "crates/verify/ess-conformance/src/interpret/setup.rs",
    "line": 26,
    "category": "correctness",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "Store::establish inserts a supplied text identity while leaving minted unchanged. With a fresh scenario and a UUID identity, establish 00000000-0000-4000-8000-000000000001, then execute an ordinary creates branch for that entity whose identity is generated. mint uses tick() from zero and returns precisely that UUID. create rejects the already-held identity rather than selecting another fresh generated identity. execute returns Undetermined::Request, and the target never installs the attempted next store, so minted remains zero and repeated creation is stuck on the same collision. This is an introduced interaction with the new setup surface; existing collision rejection is still required for explicitly supplied or recorded identities. Related locations: interpret/execute.rs:83, :841 and :1191. Reproduce using valid upstream setup followed by two ordinary generated-identity creates; require three distinct preserved rows and retain supplied/recorded collision refusal. Source-traced counterexample only; own test/build executions 0."
  }
]
```
