---
format: aep.planning-md/3
id: story:feature-request-428
kind: story
status: active
title: Synthesis binds a view parameter only to a stored field of the same name (ESS-SYNTH-005)
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#428
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/identity.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/view-param-binding.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_copied_view_parameter.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/view_param_binding.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:22Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:22Z", actor: "human:timo", revision: 7}
---
## Outcome
Resolve beyond10x/ess#428: Synthesis binds a view parameter only to a stored field of the same name (ESS-SYNTH-005).

## Origin
beyond10x/ess#428, filed 2026-10-05; a downstream specification (ess/20, ess 0.52.0) whose list-by-owner and list-by-scope views lose their scenarios. Related to #360.

## Fit review
1. Need: a view parameter compared by `==` with a stored field must be sent the value the scenario stored in that field, whatever the parameter is named. This includes a member of a struct field, such as the row's struct identity. Fresh reproduction `<fit-review scratch>/probe-428/` on installed ess 0.52.0 (`synth.out`). `owner == param.owner` is synthesized. `owner == param.who` gives ESS-SYNTH-005 "`param.who` is bound by nothing a scenario knows". `ref.tenant == param.tenant` gives ESS-SYNTH-005 on both `ref.tenant` and `param.tenant`. Requester's proposal, theirs: bind from the equality the filter states, not from the name.
2. Class: defect. `identity::param` already states the rule, "Read off the filter, never off a name" (`crates/verify/ess-conformance/src/synthesize/identity.rs:242-248`). But it returns only `ScenarioValue::Instance` values (:275) and only one-segment row paths (:295). `bound` otherwise falls back to `settled.get(&param.name)` (`src/synthesize.rs:9677-9686`). So a plain-valued field under another name, or a dotted member, is never bound. The predicate is the same in all three rows of the issue; only the binding differs.
3. Existing idiom: rename the parameter to the field's name. That covers row 2 but changes the published query contract. It cannot cover row 3, because no parameter name can be `ref.tenant`. Dropping the filter, the requester's workaround, changes the view's meaning. There is no idiom for row 3.
4. Fit: this extends the existing filter-read binder, with no new vocabulary.
   - (a) `identity::param` returns any settled value of the one field the parameter is compared with, not only an instance, as long as the declared types agree (the type check stays, :265-268).
   - (b) `compared` admits a dotted row path. Its value is projected from the settled struct literal. Synthesis sends struct identities as literals (`probe-428/suite.json`: `"ref":{"kind":"literal","value":{"key":"ref.key","tenant":"ref.tenant"}}`). An instance- or observed-valued struct has no member projection in `ScenarioValue` (`src/scenario.rs:1614-1660`), so it stays refused, with that reason named.
   - (c) The filter-read binding runs before the name fallback. Hypothesis, not reproduced: today a parameter named after one field but compared with another is sent the wrong field's value.
   - The same `bound` feeds the excluded-row, paging and absent-parameter reads (`synthesize.rs:6826,6878,6905,6958,7050,7169`), so they all gain the binding.
   - Coordinated with #430: a member value sent as a parameter must be redrawn in a caller-swapped run.
5. Second adopter: an order history lists orders `placed_by == param.customer`. A parcel locker lists slots `slot.bank == param.bank`, where the identity is `{bank, door}`.
6. Cost: no source or suite format. Bytes change only where a refusal becomes a scenario, or where name-binding and filter-binding disagreed. Measure with the before/after synthesis of every repository model, as `docs/design/input-guard-overlap-precedence.md` did (209 specifications). No new diagnostic.
7. Alternatives: (a) change nothing and document "name the parameter after the field". That does not reach struct members. (b) Add a binding key on the parameter (`binds: owner_subject`). That is new surface, restating what the filter already says. (c) Chosen: the requester's rule, implemented in the binder that already claims it.

#360 check: #360's repair is in the 0.53.0 head `087935463f` (commit `48d5cc77b3`, `crates/verify/ess-conformance/tests/related_copied_view_parameter.rs`). It is not named in `CHANGELOG.md` `## [0.53.0]`, and `story:feature-request-360` is still `active`. It binds a parameter compared with a link field that holds a captured instance (fixture `ByOrder`, `source_order == param.order`) and a same-named copied field (`ByDepot`). It does not cover a plain-valued field under another name or a struct member (identity.rs:275, :295). So #428 is not fixed and not a duplicate. It extends the same seam.

## Decisions
accept as proposed. View parameters are bound from the filter's `==`/`!=` comparison with one row field or struct member. That covers any settled literal value, and member projection from a literal struct. The name fallback is kept only where the filter does not decide. Instance-valued struct members stay refused, by name. No format bump. Sequence after #360 (same files) and with #430.

## Acceptance
- differently_named_param_binds_from_filter: `owner == param.who` synthesizes the outcome's view read with `who` = the stored owner, without ESS-SYNTH-005.
- struct_member_param_binds_from_literal_identity: `ref.tenant == param.tenant` sends the tenant member of the created identity, with a decoy row of another tenant excluded.
- same_named_param_bytes_unchanged: `owner == param.owner` and the #360 fixture keep their bytes.
- filter_binding_wins_over_name: a parameter named after field A but compared with field B is sent B's value.
- ignored_param_mutant_fails: a target ignoring `who` or `tenant` fails a decisive view assertion.
- instance_struct_member_refused_by_name: a member of an observed struct identity is refused with a cause naming the missing member projection.

## Scope
- crates/verify/ess-conformance/src/synthesize/identity.rs  cited — `param` :249-279, `compared` :282-303
- crates/verify/ess-conformance/src/synthesize.rs  cited — `bound` :9667-9690
- crates/verify/ess-conformance/tests/related_copied_view_parameter.rs  cited — #360 regression must stay green
- crates/verify/ess-conformance/tests/view_param_binding.rs  inferred — new regression test
- crates/verify/ess-conformance/tests/fixtures/view-param-binding.yaml  inferred — brand-free fixture
