---
format: aep.planning-md/3
id: story:synthesized-and-authored-view-reads-carry-the-rows-shape
kind: story
status: active
title: Synthesized and authored view reads carry the row's shape
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#680
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-view-read-checks-every-rows-shape
- depends_on: story:the-go-runtime-checks-a-view-rows-shape
- depends_on: story:the-typescript-runtime-checks-a-view-rows-shape
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/view_row_shape.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/web_replay.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/view_row_shape.rs
- confidence: inferred
  path: suites/generated/billing/suite.json
- confidence: inferred
  path: suites/generated/gatepass/suite.json
- confidence: inferred
  path: suites/generated/oracle-fixture/suite.json
- confidence: inferred
  path: website/data/billing.presentation.json
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:46:55Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:46:56Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Once the three runners check a view row against a shape (`story:a-view-read-checks-every-rows-shape`,
`story:the-go-runtime-checks-a-view-rows-shape`,
`story:the-typescript-runtime-checks-a-view-rows-shape`), synthesis and authoring must write that
shape on every `expect_view` and `eventually_view` step they emit, or no suite carries it.

On 0.92.0, for `example.order.Orders` (`order_id`, `title`, `placed_at: Timestamp`):

```console
$ ess verify conform synthesize --path rows --out suite.json
$ ess verify conform author --path rows --scenarios rows/placed-order-is-listed.yaml --out authored.json
```

The synthesized `expect_event` step carries `shape`; both synthesized `eventually_view` steps and the
authored one (`contains: {title: a}`) carry none.

## Acceptance

- `ess verify conform synthesize` on the model above writes `ess-conformance/64` (or `/65` with
  `--suite-format 5`), and each `eventually_view` step of `example.order.Orders` carries
  `"shape": {"order_id": {"holds": "primitive", "kind": "string"}, "placed_at": {"holds":
  "primitive", "kind": "timestamp"}, "title": {"holds": "primitive", "kind": "string"}}`.
- `ess verify conform author` on the authored scenario writes the same shape on its view step.
- A model with no view keeps its suite format and bytes (an existing test over a view-less model
  still compares equal).
- The committed suites under `suites/generated/` and `website/data/billing.presentation.json` are
  regenerated, and `ess verify conform run --target interpreted` passes on each example model.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/authored.rs
- inferred: crates/verify/ess-conformance/src/view_row_shape.rs
- inferred: crates/verify/ess-conformance/src/web_replay.rs
- inferred: suites/generated/billing/suite.json
- inferred: suites/generated/gatepass/suite.json
- inferred: suites/generated/oracle-fixture/suite.json
- inferred: website/data/billing.presentation.json
- inferred: crates/verify/ess-conformance/tests/view_row_shape.rs
confidence: medium

`synthesize.rs:10121` (`payload_shape`) is the event flattening to reuse over `ResolvedView.fields`;
`synthesize.rs:10275` (`view_expectations`) and `authored.rs:4678-4680` build the view steps;
13 construction sites in `synthesize.rs` and 2 in `authored.rs` (grep `ExpectView {`,
`EventuallyView {`).

## Fit review

1. **Need.** A synthesized suite must check what the view declares, not only what one scenario
   claims (`run.txt` § 2, § 3). Requester's proposal (theirs): "a new suite format member, selected
   the way earlier additive suite formats were, so suites that do not use it keep their bytes".
2. **Class.** Gap (see the parent story).
3. **Already expressible?** An author can write `satisfies: defined(<field>)` per field; synthesis
   writes nothing (`run.txt` § 2), so the idiom fails policy 1 of
   `docs/design/generic-language-features.md` § Policy.
4. **Fit.** The same selection rule as every additive pair: the suite moves to the major its
   constructs need (`leaf_payloads.rs:12-21`, `member_paths.rs:19-24`). Emitted alike by synthesis
   and authoring, so the two populations of scenarios are held to one rule.
5. **Second adopter.** Any model with a view; every one of the three committed example suites has
   view reads.
6. **Cost.** Every suite with a view read moves to /64 (/65): a coordinated regeneration of the
   committed suites in this story, and generated Go and TypeScript packages from older `ess`
   releases refuse the new suites by major. No source format.
7. **Alternatives.** (a) Emit the shape only behind an opt-in flag: a default suite would still not
   check the declaration, refused under policy 1. (b) Emit only on views whose fields are all
   required: arbitrary; `LeafShape` already handles `Optional`. (c) Chosen: always emit on view
   reads.

## Decisions

Accept, redesigned (the design is `story:a-view-read-checks-every-rows-shape`'s; this is its
synthesis and authoring part). Every view read carries the shape; suites without view reads keep
format and bytes.

## Source

github:beyond10x/ess#680
