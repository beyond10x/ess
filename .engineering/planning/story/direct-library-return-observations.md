---
format: aep.planning-md/2
id: story:direct-library-return-observations
kind: story
status: active
title: Assert typed direct library returns without invented events
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: schemas/generated/ess.schema.json
revision: 5
---
# Direct library return observations

## Accepted scope

The approved ER executable-contract plan requires actual pure-library return observations. ESS 0.37.0 refused authored `response:` with ESS-AUTHOR-001 and could not assert a silent return without invented events. ER retains that minimal reproducer in `ess/reproducers/pure-return/`. The existing typed command response and SemanticCommandResult.response own these values; this work adds their direct observation, without a persistence or event claim.

## Delivered contract

Source `ess/16` declares `returns: true`; authored `ess-scenario/4` supplies literal response fields. Ordinary suite/26 and inventory suite/27 carry the new observation. The Rust runner validates actual complete response shape, nested presence, exact typed values, ordered lists and duplicate values, then compares the authored literals. Partial top-level assertions do not weaken full response shape validation. Independent direct-return resource limits preserve legacy selection behavior. Go and TypeScript generators explicitly refuse the new vocabulary. Native ESS Binary64 conformance remains explicitly unsupported; this extension does not relax it.

## Acceptance and evidence

`crates/verify/ess-conformance/tests/direct_returns.rs` names the executable acceptance cases: correct and wrong literal returns, absent/extra/nested fields, source coordinates, old-reader refusal before effects, preserved legacy bytes, exact collections, generated shape validation, exact suite/report association and resource bounds. Schema regeneration is retained in `schemas/generated/ess.schema.json`.

`docs/design/direct-library-returns.md` defines the format boundary. `docs/evidence/direct-library-returns/` retains the initial red results, old 0.37.0 reader refusals, suite bytes, targeted tests, strict Clippy, MSRV and task ci-lint evidence. Independent adversary review found nested presence and opaque Json resource-limit defects; both received red regressions and fixes. The final review and seven independent rechecks are retained beside the sixteen direct-return regression cases. The final complete conformance-crate run passed; historical ignored cases remain identified in its output.

The targeted pre-push lane is green. The complete repository Gate is still a CI integration requirement; no release or main integration is asserted here. AEP report admission for new ESS suites belongs on the consuming side and is tracked separately in ER.
