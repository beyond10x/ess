---
format: aep.planning-md/3
id: review-result:counter-reachability-413-design-r1
kind: review-result
status: active
title: ESS413 bounded arithmetic and separate seed design review
relations:
- reviews: story:counter-reachability-arithmetic-completeness
revision: 1
---
approve

Read-only design assessment of ESS413 revision 2, SHA256 `5916502a20b823b4b4bec949cb0101e77462f8f8c3b3129143946593c3c27f3e`. No unresolved design finding. This approves the proposed design for owner planning; it does not authorize source dispatch or claim an implementation, generated witness, passing gate, or delivered adopter capability.

I inspected the cited ESS base `2f554561bef25125a93a1fb1d6517d50cb24ed20` and then-current integration commit `8a94f4a31c29c7b695fd5dbc7b1f0732cb228726` through Git objects. Proposed APIs and formats are new obligations, not APIs present in either inspected source. No builds, synthesis, mutations, model validation or executable probes ran. Earlier reproduction results remain attributed to the retained source assessment and counter controls, not newly verified results.

A is appropriately independent. In the current integration's `synthesize/subject_fact.rs:2529–2552`, chaining an absent checked literal bound can shrink the enumeration and still return a supposedly complete set. `ess-primitives/src/facts.rs:304–316` intentionally rejects an integral result beyond signed i64. Returning the existing incomplete/unknown result when a necessary bound cannot be constructed is the narrow correction; primitive arithmetic and search limits stay intact. All required bound operations, including negative magnitude/widest-step construction, must obey that rule. The proposed MAX/MIN/padding regressions and finite limit-2 CAS control distinguish arithmetic completeness from ordinary input grounding. A alone owes honest refusals, not zero refusals at unreachable extreme states. Shared related-counter behavior needs the specified regression rather than speculative sibling edits.

B is a coherent additional capability, necessarily broader than A and still outside pinned50. Explicit FILE/INSTANCE selection preserves the distinction between compiling an authored scenario and supplying one literal initial row. Existing `authored.rs` Arrangement/EntitySetup types and its setup compilation can be reused; current `input.rs:340–381` validates identity, declared lifecycle state, full typed fields and invariants and returns unresolved validation as an error. Compiling the whole source before selecting its setup, preserving absence versus null, binding the IR, and refusing dependent/multi-row/historical arrangements avoid an alternate permissive grammar. Selection does not import timeline effects, caller authority, lifecycle reachability or external temporal authorization.

The witness contract is substantive: ordinary arrangement first; seeds only at a still-unmet typed obligation; real EstablishEntity, observation, checked command and outcome/preservation assertions afterward. Successful ordinary steps cannot be replaced, and a different authored assertion or matching scenario name cannot erase a generated refusal. A stable internal obligation handle may be new work; diagnostic-string matching would violate this design. Literal identity collisions, related-row dependencies and unsupported target setup remain refusals. MAX and MAX−1 must be executed exactly through Rust and emitted Go/TS readers/targets; setup acknowledgements and metadata alone do not constitute coverage. The proposed wrong-row/no-op/Unsupported and mutation controls are appropriate acceptance obligations, not results established here.

The provenance design keeps generated and authored acquisition separate and handles unused selections honestly. Source digests, full typed selections and application-to-step bindings provide inspectable provenance, not authentication of absent source documents. Admission must reject dangling or forged membership before callbacks. Ordinary and declared-coverage paths, caller specialization, component filtering, selected-input/report carriers and parent metadata are included; this prevents a superficially successful ordinary-only implementation from discarding an existing consumer path. Deterministic ordering, a shared bounded seed-search allowance and exact numeric representations are specified; no pass/fail total is inflated by selected or unused rows.

Current-base compatibility details must remain in the implementation brief. At `8a94f4a31`, `scenario.rs:157–169` selects fresh ordinary format34 and establishes empty logical initial state; `admission.rs:175–218` admits only1–35, requires that empty-state authority for34/35, and enumerates coverage-bearing majors. `SuiteProvenance` has no seed field. Go and TS have their own closed version/metadata admission. The owner-reserved proposed42/43 pair therefore requires the explicit coordinated writer/reader migration described in V2, preserving empty-state authority and preventing a later fresh-format pass from lowering a seeded suite back to34. Allocation36–41 is owner coordination evidence, not implemented support in this inspected commit. Reconcile the actual dispatch base before fixing exact paths or API names; do not reinterpret any older contract. Seed-free output stays on that base's normal selection, while explicit nonempty admitted selections, even unused ones, use42/43. Old readers refuse the new pair; new readers refuse seed metadata on old majors. Existing source/authored document formats and production model APIs need no new construct.

No source-selection, invariant, integer, provenance, minimality or format contradiction remains in the proposed design. Required execution and mutation acceptance remains outstanding. The ESS hardening design-review procedure informed this comparison; executable validation and planted-defect exercises were intentionally outside this read-only proposal review. Original reports were preserved unchanged.

```findings
[]
```
