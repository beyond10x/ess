---
format: aep.planning-md/3
id: story:feature-request-443
kind: story
status: implemented
title: A maintained per-key aggregate record (fold)
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#443
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-441
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/read_api_view_idioms.rs
- confidence: cited
  path: docs/design/aggregate-views.md
- confidence: inferred
  path: docs/design/read-api-view-idioms.example/fold.yaml
- confidence: inferred
  path: docs/design/read-api-view-idioms.md
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:25Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:25Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-06T17:48:28Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#443: A maintained per-key aggregate record (fold).

## Origin
beyond10x/ess#443, filed 2026-10-05; a downstream specification that folds session-end rows into per-key totals (ESS version not stated). Reproduced minimally in `<fit-review scratch>/probe-443/`.

## Fit review
1. Need: per (customer, session, label), the total duration of that session's end rows, recomputed as rows arrive, with no total where it would be zero. The total is non-negative. Then, quoted: "one combined row per session is created or updated with that value". Whether "that value" is each label's total or the session's total is unclear; ask them. Requester's proposals (theirs): "an aggregate view with `materialized: true` and an update rule", or "a create-or-update effect keyed by a natural key on an outcome". The reported block: `ESS-COMMAND-018` refuses an invariant over the folded field, because the creating outcome does not set it.
2. Class: convenience. A total over stored rows is an observation, and ESS states it as an aggregate view (`docs/design/aggregate-views.md:20-21`: "An aggregate is a view … not a new declaration kind"). "Materialized" is how an implementation serves a view, not a domain fact. The ESS-COMMAND-018 refusal is documented behaviour, not a defect (`website/docs/guides/specify/fields-and-invariants.md:47-64`).
3. Existing idiom, run on installed ess 0.52.0 (not the 0.53.0 tree):
   - The end rows are the entity (`customer, session, label, duration`) with the invariant `duration >= 0`. Two views: `group_by: [customer, session, label]` and `group_by: [customer, session]`, each `{name: duration_sum, type: Integer, aggregate: {sum: duration}}` with `filter: duration > 0`. `ess specify validate` printed `demo v1 — 1 file(s), valid` (probe `system3.yaml`).
   - Zero total means absent. Empty groups are absent (aggregate-views.md:24-25). With non-negative durations, a group sums to 0 exactly when every row in it is 0, so `filter: duration > 0` leaves that group out. There is no `having` (aggregate-views.md:139-140), and none is needed.
   - Non-negative total. The domain fact is the row invariant `duration >= 0`, so a sum of non-negatives needs no invariant of its own, and the suite checks the exact sum.
   - Without the filter, synthesis writes both `…/aggregate` scenarios: `5 scenario(s) (0 authored), 0 refusal(s)` (`system5.yaml`). With the filter, it refuses them: `refusal[ESS-SYNTH-017] … cannot be arranged: no reachable state leaves row x refuted by the filter`. The 0.53.0 arranger only searches lifecycle states for a refuted row (`crates/verify/ess-conformance/src/synthesize/aggregate.rs:2350-2387`), so a value filter cannot be refuted. That is a synthesis coverage gap, not a language one. Record it as follow-up work, not as part of this decision.
   - The 0.52.0 interpreter answers `views are not interpreted yet`, so no scenario was executed. I don't know whether 0.53.0 executes them.
   - A stored per-key record also validates. Use the existing create-or-update pair (commands-and-outcomes.md:93-130) with `duration_sum` set on the creating branch, and `duration_sum >= 0` raises no ESS-COMMAND-018 (`upsert-lit.yaml`, valid). It cannot add an input amount: `{increment: input.duration}` gives `type_mismatch … input.duration is not a non-zero amount` (`upsert.yaml`). `{increment:}` takes a literal number (`website/docs/guides/specify/values-and-views.md:36`). A recompute that replaces "the previous one" from all rows is the view, not this.
4. Fit: the requester's `materialized: true` plus update rule would add a second way to state an aggregate beside `group_by`/`aggregate:` and put storage strategy into the model. That is the red flag "names a concept ESS already spells differently". The natural-key upsert already exists (ess/16) for a key the entity uses as its identity, including a struct identity.
5. Second adopter: a metering service reporting minutes per account per day; the same idiom (row entity + grouped `sum` view) serves it. A stored running total fed by an input amount would need `{increment: input.<field>}`. Nobody has asked for that, so it is not adopted here.
6. Cost of the idiom: none in the format. One note section and its test cases. Accepting the proposal would cost a new view key, a new update-rule grammar, a diff classification, synthesis and every target.
7. Alternatives: (a) `materialized: true` view: refused, as in Q4. (b) An outcome-level create-or-update by natural key: exists already for identity keys; a non-identity natural key would duplicate identity. (c) `{increment: input.<field>}` for stored totals: a plausible small extension, held until a second adopter needs a stored total rather than a view. (d) Chosen: decline with the aggregate-view idiom.

## Decisions
decline, with the idiom — a fold over stored rows is an aggregate view: the rows as the entity with `duration >= 0`, a grouped `sum` view per key, and `filter: duration > 0` so a zero total is absent. Materialization is the implementation's. A stored record keyed by identity is the existing create-or-update pair. The story body is the decline record. The idiom is documented as the section `## A total maintained per key is a grouped view` in `docs/design/read-api-view-idioms.md`, with its model in `docs/design/read-api-view-idioms.example/fold.yaml`, domain `idioms.fold`. It declares the row entity `idioms.fold.SessionEnd` (`customer`, `session`, `label`, `duration`, invariant `duration >= 0`) and four grouped `sum` views, two keys each in two forms: unfiltered `idioms.fold.TotalByLabel` (`group_by: [customer, session, label]`) and `idioms.fold.TotalBySession` (`group_by: [customer, session]`), and filtered `idioms.fold.NonZeroTotalByLabel` and `idioms.fold.NonZeroTotalBySession` (the same keys with `filter: duration > 0`). Synthesis writes group scenarios for the two unfiltered views and refuses the two filtered ones (`ESS-SYNTH-017`), so the zero-total claim is an authored scenario. Depends on #441 (edge recorded), which creates the note, the example directory and the test. No format bump; not part of the `ess/23` bump. Reply to the requester with the idiom and ask what "that value" refers to. Separate follow-up, not filed: the ESS-SYNTH-017 arranger gap for value filters on aggregate views.

## Acceptance
- aggregate_fold_idiom_validates: the example directory with `fold.yaml` validates, and `idioms.fold` holds `SessionEnd` and the four views `TotalByLabel`, `TotalBySession`, `NonZeroTotalByLabel` and `NonZeroTotalBySession`.
- aggregate_fold_idiom_synthesizes_group_scenarios: synthesis writes the `…/aggregate` scenarios of the two unfiltered views, `idioms.fold.TotalByLabel` and `idioms.fold.TotalBySession`, with no refusal naming either.
- aggregate_fold_zero_total_group_is_absent: synthesis refuses the two filtered views (`ESS-SYNTH-017`), so an authored scenario holds the claim instead. It arranges rows of durations 0 and 0 under key A and 0 and 5 under key B, and expects `idioms.fold.NonZeroTotalBySession` to hold exactly one row, B with `duration_sum` 5. It passes against the interpreted target (`crates/verify/ess-conformance/src/interpret/views.rs:236-321`, `project`, groups and sums). A target that keeps a zero-total group fails it.
- per_key_total_section_states_the_idiom: `docs/design/read-api-view-idioms.md` has the heading `## A total maintained per key is a grouped view`. The section contains "`group_by`", "`sum`", "`filter: duration > 0`", "zero total is absent", "`duration >= 0`", "materialized" and "create-or-update". The case reads the note and fails on a missing heading or phrase.

All four cases live in `crates/verify/ess-conformance/tests/read_api_view_idioms.rs`.

## Scope
- docs/design/read-api-view-idioms.md  inferred — adds `## A total maintained per key is a grouped view` (file created by #441)
- docs/design/read-api-view-idioms.example/fold.yaml  inferred — domain `idioms.fold`: the row entity and its four grouped views (directory created by #441)
- crates/verify/ess-conformance/tests/read_api_view_idioms.rs  inferred — adds the four named cases and the authored zero-total scenario
- docs/design/aggregate-views.md  cited — authority for "an aggregate is a view" (20-25, 139-140)
- crates/verify/ess-conformance/src/synthesize/aggregate.rs  cited — arranger gap noted, unchanged here (2350-2387)
