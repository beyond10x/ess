---
format: aep.planning-md/3
id: story:view-filters-bind-identity-and-link-fields
kind: story
status: active
title: Synthesis cannot bind an identity or link-field view filter (ESS-SYNTH-005 / -003 / -017)
refs:
- provider: github
  reference: beyond10x/ess#193
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:46Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:46Z", actor: "human:timo", revision: 3}
---
# Story: Synthesis cannot bind an identity or link-field view filter (ESS-SYNTH-005 / -003 / -017)

## Why

beyond10x/ess#193. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #193 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

**Verdict: needs-design.** The defect is still present on this tree (`e932781965`, 0.40.0), and nothing has fixed it. It needs one representation decision before a fix, described below.

**1. Does it reproduce, and is it already fixed? Yes, and no.**
- cited: `crates/verify/ess-conformance/src/synthesize.rs:5951` `shows()` binds only `state` (line 5974), the literal `settled` fields (`ScenarioValue::Literal` only, line 5987), and literal params (line 6005). It never binds the entity's identity field, so the left side of `id == param.id` is unbound.
- cited: `synthesize.rs:5914` `bound()` takes params only from `settled` by name. The identity is not in `settled`: it lives in `Run::instance` or in an observed event field (see `identifying()`, `synthesize.rs:5878`). So `param.id` is unbound too, and `filter.evaluate` returns `Unknown` (line 6019). That becomes `RefusalCause::ViewUndecidable` (lines 448 and 4272), rendered as ESS-SYNTH-005 at 851–854. This matches the issue text exactly.
- inferred: part 2, the link field. For an owned subject, the owner link (e.g. `branch_id`) is settled as `ScenarioValue::Instance`, not `Literal`. `shows()` drops it at line 5987, so a comparison on it stays `Unknown`.
- cited: aggregate grouping. `synthesize/aggregate.rs:1266` `held()` returns `None` for any non-`Literal` settled value (line 1277). A link-field group key is therefore unknown, and the module doc (`synthesize/aggregate.rs:14-19`) says such a row is refused as ESS-SYNTH-017. The identity case already gets a synthetic `row-{index}` (line 1270).
- inferred: I did not locate the ESS-SYNTH-003 path for a comparison on a link field. It is probably the same literal-only binding in `synthesize/subject_fact.rs` `row_truth_with` (line 242).
- cited: `git log` of the last 300 commits has no commit referencing #193 or by-id filters.

**2. Where the fix lands**
- cited: `synthesize.rs` `shows` (5951), `bound` (5914), `identifying` (5878). `bound` has about 10 callers: 4268, 4318, 4339, 4454, 4521, 7205, 7903, 8246, 8330, and `synthesize/paging.rs:55`. Each must pass the run's instance, or the arranged instance for companion rows.
- cited: `synthesize/aggregate.rs` `held` (1266). Map an `Instance` link value to a per-instance token.
- inferred: `synthesize/subject_fact.rs` `row_truth_with` (242), for the ESS-SYNTH-003 case.
- inferred: `crates/verify/ess-conformance/tests/synthesis.rs` and `tests/aggregate_views.rs` for regressions. Also agentplugins `ess:retrofitting`/`syntax.md` and the `later-formats.md` workaround, to be reverted after release.

**3. Collisions**
- inferred, high: #198, #199 and #209 all touch arrangement search in `synthesize.rs`. The file is shared, but the symbols are mostly different (`route_from`, `arrange_unbound` 7051).
- inferred, medium: #204 (`subject_fact.rs` row truth), and #211, which would add a cross-entity guard in `synthesize/existence.rs` or `subject_fact.rs`.
- inferred, low: #196 and #202 (`witness.rs`/`input.rs`), #203 and #210 (`mutate.rs`), #201 (domain command validation), #205 (preconditions/explorer), #195 (bindings).

**4. Design decision for the implementor**
- inferred: how to represent an identity that is only known at runtime as a fact. Smallest option:
  - Give each `InstanceName` or observed identity an opaque token fact such as `instance:<name>`.
  - Bind the entity's identity field and every `Instance`-valued settled field (the link fields) to their tokens.
  - When a param's type equals the identity's type, and the filter compares it to the identity or to a link field, have `bound()` supply `ScenarioValue::instance`/`observed` for it.
  - Companion rows get their own tokens, so `==` excludes them.
  - Only `==` and `!=` are decidable on tokens. Ordering comparisons stay refused.
  - Aggregate `held()` reuses the same tokens as group keys.

**Confidence: medium-high** for part 1 (the code path is read end to end). **Medium** for part 2 (the ESS-SYNTH-003 site was not located).

Paths:
- crates/verify/ess-conformance/src/synthesize.rs
- crates/verify/ess-conformance/src/synthesize/aggregate.rs
- crates/verify/ess-conformance/src/synthesize/paging.rs
- crates/verify/ess-conformance/src/synthesize/subject_fact.rs (inferred)
- crates/verify/ess-conformance/tests/synthesis.rs (inferred)
- crates/verify/ess-conformance/tests/aggregate_views.rs (inferred)

Verdict: needs-design
