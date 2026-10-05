---
format: aep.planning-md/3
id: story:feature-request-447
kind: story
status: active
title: 'Multi-source views: joins and computed columns'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#447
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-441
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/tests/read_api_view_idioms.rs
- confidence: cited
  path: docs/design/aggregate-views.md
- confidence: inferred
  path: docs/design/read-api-view-idioms.example/joined.yaml
- confidence: inferred
  path: docs/design/read-api-view-idioms.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:27Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:27Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome
Resolve beyond10x/ess#447: Multi-source views: joins and computed columns.

## Origin
beyond10x/ess#447, filed 2026-10-05; an adopter's metrics read API, where two reads combine a call record with its queue and add computed label columns.

## Fit review
1. Need, in two parts:
   - (a) a row of one entity carries a field of the row its reference names: a call's queue label;
   - (b) a column derived from other values: a label.
   Minimal reproduction, written fresh (`<fit-review scratch>/probe-447/`): `Queue {label}`, `Call {queue_id}` with `references` to `Queue`. The requester's syntax, not adopted: a view over a declared relation path that projects fields of both ends (an equi-join), and computed columns.
2. Class:
   - (a) is a convenience where the referenced value is fixed when the row is written, because copy-at-write states it (question 3). It is a gap only for a value that changes after the row is written, a live join.
   - (b) is local policy where the label is presentation, and a convenience where it is an enum value's label (question 3).
   - Joins were placed out of scope when aggregate views were designed: "`source:` stays one entity" (docs/design/aggregate-views.md:1033).
3. Existing idiom, validating on the 0.53.0 debug build (probe-447):
   - copy at write: `sets: {queue_label: {related: {via: input.queue_id, field: label}}}` (ess/16, values-and-views.md:106-129). Views then project or group by `queue_label`: "A view still reads one entity: group by the copied field rather than by a field of another entity" (values-and-views.md:150-151).
   - Synthesis checks the copy against decoy rows, and re-runs an `updates:` of the read field just before the branch (values-and-views.md:153-162). A target that reads the wrong row or copies early fails.
   - live value: a second view over `Queue` exposing `label`. The client joins on `queue_id`.
   - enum label: a variant's `display:` (wire-names.md:47-50).
   - A view field naming the other entity is refused: `queue.label` is "invalid field name identifier … contains '.'" (probe-447 n1).
   - A field with neither `aggregate:` nor a source field is refused with `ESS-VIEW-001` (probe-447 n4).
4. Fit:
   - A join view would be the first view with two sources. `ViewSpec.source` is one entity (`crates/specify/ess-domain/src/view.rs:911-913`), and the field check requires every field to be a field of the source (docs/design/aggregate-views.md:31-32).
   - It would need path syntax in fields, a rule for a missing or absent referenced row (`{related:}` already has one: values-and-views.md:144-149), and join evaluation in the interpreter, the generated Rust and Go queries, the runners, the UI read and Entity Runtime.
   - Synthesis would need arrangements that change the referenced row after the referencing row, which is the contrast that separates a live join from copy-at-write.
   - Computed columns are #441's construct under another name.
5. Second adopter: an order list showing the customer's name. It is served by copy-at-write when the name at order time is the fact, and by two views when the current name is. The live-join case is real but uncommon in the requests so far: issue #447 is the first to ask (inferred from the issue list given to this review).
6. Cost if accepted: multi-source views, path-typed fields, a missing-row rule, ess/23, a suite pair, diff classes and every target. Cost of the idiom: one section of the shared idioms note and its example.
7. Alternatives:
   - (a) Change nothing.
   - (b) The requester's relation-path view with computed columns.
   - (c) A narrower `field: {related: {via: <reference field>, field: <field>}}` on a view field: the existing value source reused at read time, one hop, absent where the reference is. It is the smallest live join and is recorded for a second adopter.
   - Chosen: decline. The note section says when copy-at-write is the right fact and when two views are, and that joins stay out of scope.

## Decisions
decline, with the idiom — The story body is the decline record. The section `## A view has one source` in `docs/design/read-api-view-idioms.md` documents three idioms, with their models in `docs/design/read-api-view-idioms.example/joined.yaml`, domain `idioms.joined`: entities `idioms.joined.Agent` (`agent_id`, `label`) and `idioms.joined.Call` (`call_id`, `agent_id`, `agent_label` copied at write), the aggregate view `idioms.joined.CallsByAgentLabel`, and the two-view alternative `idioms.joined.Agents` and `idioms.joined.Calls`:
- copy-at-write with `{related: {via, field}}`, then project or group by the copy, where the fact is the value when the row was written;
- two views joined by the consumer, where the fact is the current value of the other row;
- variant `display:` for enum labels, with other labels left to the consumer.

It states that a view has one source (aggregate-views.md:1033) and records alternative (c) as the starting shape if a live join is needed. Depends on #441 (edge recorded), which creates the note, the example directory and the test. No format bump. Reply to the requester with the idiom.

## Acceptance
- one_source_section_states_the_idiom: `docs/design/read-api-view-idioms.md` has the heading `## A view has one source`. The section contains "`{related: {via`", "value when the row was written", "current value", "two views", "`display:`", "`source:` stays one entity" and "`field: {related:`" (alternative (c)). The case reads the note and fails on a missing heading or phrase.
- join_idiom_examples_validate: the example directory with `joined.yaml` validates, holding `idioms.joined.CallsByAgentLabel` (grouped by the copied label) and the two-view alternative `idioms.joined.Agents` and `idioms.joined.Calls`.
- copied_related_field_synthesizes_against_decoys: `idioms.joined.Call.agent_label` synthesizes its `{related:}` scenario with no refusal.
- multi_source_view_fields_stay_refused: a view field naming another entity and a field with no source stay refused with today's messages.

All four cases live in `crates/verify/ess-conformance/tests/read_api_view_idioms.rs`.

## Scope
- docs/design/read-api-view-idioms.md  inferred — adds `## A view has one source` (file created by #441)
- docs/design/read-api-view-idioms.example/joined.yaml  inferred — domain `idioms.joined`: copy-at-write and two-view examples (directory created by #441)
- crates/verify/ess-conformance/tests/read_api_view_idioms.rs  inferred — section, validation, refusals, synthesis
- docs/design/aggregate-views.md  cited — joins out of scope, 1033
