---
format: aep.planning-md/3
id: story:feature-request-365
kind: story
status: active
title: UI bounded reads support checked row filters in both renderers
refs:
- provider: github
  reference: beyond10x/ess#365
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/ui/ess-ui
- confidence: cited
  path: crates/ui/ess-ui-check
- confidence: cited
  path: crates/ui/ess-ui-react
- confidence: cited
  path: crates/ui/ess-ui-test
- confidence: cited
  path: crates/ui/ess-ui-tui
- confidence: cited
  path: crates/ui/ess-ui/Cargo.toml
- confidence: cited
  path: docs/design/ui-read-filter.md
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T10:09:54Z", actor: "human:timo", revision: 10, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T10:09:54Z", actor: "human:timo", revision: 11, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

A bounded UI read can filter rows by a checked predicate over its row, page parameters and UI state, with the same meaning in React and TUI. This addresses consumer issue365 using the accepted filter vocabulary.

## Fit review

1. Need: an operator console shows only runs belonging to the current objective even when its bounded view has no objective parameter. Issue365 supplies the concrete row.objective_id == params.objective_id example.
2. Class: expressiveness gap. Reads has no predicate at ess-ui/src/model.rs:2186-2216 in published55061600.
3. Existing expression: declared server view parameters are preferred and now served after PR386. They cannot replace every renderer-local narrowing. Dynamic menu filter is existing vocabulary precedent.
4. Fit: implement accepted docs/design/ui-read-filter.md:52-166 completely: filter on eligible listing/choice reads, admitted boolean grammar/roots, fail-closed evaluation, explicit server-paging refusal and model-aware path checks. Do not introduce the proposed alternative where spelling.
5. Second adopter: severity filtering over a bounded recent-events list, already described in the design.
6. Cost: additive ess-ui/1 field with old readers refusing it; model/schema/reference, seven checker rules and coordinated React/TUI semantics. No server authorization or binding filter semantics are implied. Existing choice identity is Reads.key and must survive filtering.
7. Alternatives: retain missing capability; require a server parameter for every local view; selected existing accepted renderer-local design while preserving server parameters as preferred mechanism.

## Decisions

Accept, redesigned as already approved in docs/design/ui-read-filter.md and PR367. Implement as one coherent PR including model/checker/both renderers/parity rather than four remote-gated stages. Refresh stale design source anchors and historical dependency text against55061600 before code changes. Source comments must describe current behavior. Do not modify the already-published PR381 candidate while its gates run.

## Acceptance

All named requirements and cases in docs/design/ui-read-filter.md:97-135,180-199 remain required. Prove red then green for row/page/state predicate filtering; filtered totals and empty states; removal of selected rows; patch-out/patch-back and count_new; server-paging refusal; and cross-renderer admitted expression parity. Named refusals/warnings and unsupported placements are checked. Retain legacy no-filter bytes/behavior where required by the design. Run affected package tests and strict Clippy/fmt; schema/reference generation and projection checks must hold before publication.

## Scope

ess-ui model/schema; ess-ui-check rules/expression/model; React read emission/core/data/live/collection/choice; TUI expression/app/view; parity tests; schema/reference and accepted design clarification. No conformance/server source changes. Coordinator owns AEP, commits and remote publication.

## Numeric parity scope refinement

The shared filter requires React/TUI parity even for admitted mixed scalar comparisons. Rust f64 display and JavaScript String(number) differ at small/large exponent thresholds. The correction uses ryu-js ECMAScript number formatting, adding workspace Cargo.toml and crates/ui/ess-ui/Cargo.toml with the existing Cargo.lock scope. These manifest paths are cited from the implementation dependency seam. Regression controls cover scalar and canonical array/object comparisons and negative zero. This changes only UI filter comparison, not ESS numeric semantics.

## Corrected filter composition and closing source review

Frozen revision contains32paths, patch01b3f527918966d0d6c1393dc4439e102c3f118f21afaa09115703cba3bc0130. Reviewer verified every hash, read the owner-produced baseline/treatment logs and found no further concrete counterexample; reviewer execution count0. Final full six-package run58 reports531 passed/0 failed/0 ignored (69 UI,119 checker,49 docs,121 React,69 UI test,104 TUI). Final strict lint/ci-lint/site checks remain owner work before publication.

The filtered refetch now admits events through only_if first. UI numeric comparisons use ryu-js ECMAScript formatting without changing the comparison grammar or other ESS numeric semantics. Final numeric test bytes fail the old formatter (log54), and the14-case focused suite passes with arbitrary_precision (55). Earlier feature-on numeric failures50/51 were fixture construction failures, not formatter defects.

The source-switch/live composition is measured for setDataAdapter in an unbound generated app (47 red0/1,49 green13/0); direct setAuthorization plus useLive is not an emitted combination because bound apps poll. The correction resets/hides local live rows and totals when the source epoch changes. The first correction also discarded queued coalesced events on ordinary raw-read completion; log53 reproduces that newly introduced ordering failure. Queue clearing is now restricted to actual source changes and55's14/0 includes the regression. Retain both the unreachable direct-auth fixture observation and the reachable adapter result rather than claiming an unexecuted authorization case.

Earlier stale refresh, authorization-aware read cache and stalled-poll corrections remain covered. Immutable reviews pass1/pass2/pass3 and the closing pass4 retain distinct findings and evidence provenance; no source changes are attributed to the reviewer.

## Local committed delivery candidate

Botaabc378c8 commits all32 reviewed source paths after final six-package531/0/0, strictClippy, exactci-lint and tasksite-build passed. It merges without source conflicts into combinedfba30d2af. All32 implementation hashes matched after integration; the three conformance/domain/synthesis crate trees were unchanged by this merge. Final combined lint/projection/site checks are running. Publication will reusePR387 after its single service-contract expectation correction, preserving the UI source commit and saving a separate full remote gate.
