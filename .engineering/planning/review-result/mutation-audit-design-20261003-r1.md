---
format: aep.planning-md/3
id: review-result:mutation-audit-design-20261003-r1
kind: review-result
status: active
title: Mutation audit and known failures design, first review
relations:
- reviews: story:feature-request-294
- reviews: story:feature-request-295
- reviews: story:feature-request-296
- reviews: story:feature-request-236
revision: 1
---
needs-revision

`docs/design/mutation-scope-and-known-failures.md:70-76` makes any component-suite synthesis-refusal inventory delta establish that a mutant is in scope. The existing `synthesize_for` contract deliberately retains the whole system's refusals unchanged (`crates/verify/ess-conformance/src/synthesize.rs:1939-1940,1962,1981-1984`). A mutation wholly unrelated to the selected component can therefore change a global refusal and be retained in that component's audit. This also expands accepted story #236, whose decision keeps mutants where they change a scenario in the component suite (`.engineering/planning/story/feature-request-236.md:61-64`), without defining how a refusal with no executable scenario is attributed to a component. Define a component-attributable refusal inventory before using refusal deltas for scope; a whole-system or unattributable refusal cannot by itself place a mutant in scope. Add a two-component control where only the other component's refusal changes, plus a selected-component refusal with no executable scenario.

`docs/design/mutation-scope-and-known-failures.md:157-168` requires every collected report to match its own exact emitted suite and claims original-byte controls, but it neither requires an exact-suite report nor changes the ordinary report versions. Current collect accepts `ess-conformance-report/1`; that format has specification digest, suite format and scenario count, but no suite-byte digest (`crates/verify/ess-conformance/src/evidence.rs:18-38`). Its admission therefore accepts a report from different suite bytes when the spec digest, suite version, count and reported non-pass IDs coincide (`crates/verify/ess-conformance/src/mutate.rs:2651-2669`). Report/2 already carries and validates the exact `sha256-json-bytes/1` suite digest (`crates/verify/ess-conformance/src/counts.rs:296-306,345-357`). Require report/2 (or another exact-suite-bound result) for manifest/4 collection while retaining report/1 only under legacy manifest semantics, and add a same-spec/same-ID/same-count but changed-body stale-report control.

`docs/design/mutation-scope-and-known-failures.md:91-100` requires an exact identity containing a build identity so an upgrade invalidates the declaration. This is incompatible with protected one-time-response suites: all current runners intentionally replace the target identity with `{name: "one-time-protected-target", version: ""}` so target-returned identity text cannot disclose a protected value (`crates/verify/ess-conformance/src/runner.rs:372-387`; the generated Go and TypeScript runtimes do the same). Such a target upgrade leaves the observed identity unchanged and silently preserves the declaration, contrary to the required build mismatch. Define a privacy-safe public build identity for this path or explicitly refuse known-failure declaration/accounting and mutation eligibility for protected suites, and include that path in the changed-build acceptance control.

Reviewed the complete design, stories #294/#295/#296, accepted stories #212/#236, issue reproductions 6a/6b/7, and the current mutation, component synthesis, report admission, runner identity and CLI seams at integration commit `652b8e7ed8b34c1de015855cc207984cb2965505`. The emit-swap candidate rules, unavailable-site result, bounded #212 operator definitions, strict ordinary failure preservation, exact known-failure IDs, eligible baseline-witness rules, manifest/report4 versioning and external accounting are otherwise coherent. This was a bounded read-only design review; no repository/AEP files were changed and no compiler or executable test was run.

```findings
[
  {
    "file": "docs/design/mutation-scope-and-known-failures.md",
    "line": 71,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "component scope treats a synthesis-refusal inventory delta as sufficient, but synthesize_for retains the whole system's refusals unchanged, so a mutation wholly outside the selected component can enter its audit; define and test component-attributable refusal selection before comparing refusal inventories"
  },
  {
    "file": "docs/design/mutation-scope-and-known-failures.md",
    "line": 159,
    "category": "compatibility",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "manifest/4 promises that every collected report matches its exact emitted suite, but collect remains allowed to consume report/1, which has no suite-byte digest and can accept stale same-spec/same-count suite bytes; require an exact-suite-bound report for /4 or define an equivalent binding"
  },
  {
    "file": "docs/design/mutation-scope-and-known-failures.md",
    "line": 98,
    "category": "compatibility",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the mandatory build-bound implementation identity cannot be satisfied by protected one-time-response suites, whose Rust, Go and TypeScript runners intentionally publish a fixed name and empty version; define a privacy-safe build identity or explicitly refuse this feature for those suites"
  }
]
```
