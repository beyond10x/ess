---
format: aep.planning-md/3
id: review-result:mutation-audit-design-20261003-r2
kind: review-result
status: active
title: Mutation audit and known failures final design review
relations:
- reviews: story:feature-request-294
- reviews: story:feature-request-295
- reviews: story:feature-request-296
- reviews: story:feature-request-236
revision: 1
---
needs-revision

`docs/design/mutation-scope-and-known-failures.md:152-166` requires actual generated Go/TypeScript results to be accounted through `verify conform report --suite --results`, but that command admits only `ess-conformance-results/1` (`crates/edge/ess-cli/src/main.rs:647-673`; `crates/verify/ess-conformance/src/results.rs:33-34,313-327`). The generated runners do not emit that document: their lossless artifact is `ess-conformance-report/2` with producer profile `go-scenario-status/2` (`crates/verify/ess-conformance/src/go/runtime.go:5136-5162`; `crates/verify/ess-conformance/src/ts/runtime.ts:7436-7484`). This is not a mechanical conversion. Results/1 has external/Rust status semantics and admits only passed, failed, error and unsupported; generated report/2 deliberately preserves a fifth `skipped` category (`results.rs:23-25`; `counts.rs:29-48`). Therefore the required actual-runner accounting and native/external status equivalence have no lossless CLI reader path, and a generated execution/1 sidecar would bind a report/2 that `report --results` cannot consume. Specify an accounting mode that admits the existing exact-suite report/2 plus its execution/1 sidecar and declaration, or define a new generated results format/output and producer profile that preserves all five categories. Add actual generated Go and TypeScript failed/skipped controls; mapping skipped to another status or hand-constructing results/1 would not satisfy the contract.

The three pass-one findings are otherwise closed. Component mutant selection now remains based on typed scenario body/presence differences, whole-system refusals cannot establish scope, selected accepted commands with attributable refusals become explicit unavailable obligations, and unrelated refusals remain visible outside scoring. Manifest/4 requires exact-suite report/2 while legacy manifests retain report/1. Protected target identity remains redacted; the independently supplied public build digest is frozen before target calls, bound to exact report and suite bytes in execution/1, compared with the declaration/manifest, and never derived from target-returned data. The declaration, baseline eligibility, unavailable-site, ordinary strict-failure, accounting partition, versioning and exit contracts are coherent after the remaining generated-accounting path is defined.

Reviewed the complete design and current stories #294/#295/#296, accepted stories #212/#236, all pass-one corrections, and the relevant generated report, supplied-results, report admission, mutation collection, component synthesis and protected identity seams. Design/planning bytes are those frozen at `3dfaba0eea3108103c6e46099a56f0851f91e190`; integration source was inspected from `f24dafb1e560c8df50721f32591fd2a7e2824e37` through `267130977132849602a524e03a28f4a90d2601d8`, with no reviewed source-seam or design/planning diff. This was a bounded read-only design review; no repository/AEP files were changed and no compiler or executable test was run.

```findings
[
  {
    "file": "docs/design/mutation-scope-and-known-failures.md",
    "line": 152,
    "category": "compatibility",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "actual generated Go/TypeScript runners emit lossless report/2 with the five-category go-scenario-status/2 profile, while the required report --suite --results accounting path accepts only results/1 with external/Rust semantics and no skipped status; define an accounting reader for report/2 plus execution/1 or a new lossless generated results contract"
  }
]
```
