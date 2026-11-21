---
format: aep.planning-md/3
id: story:refusal-overlap-witnessed-for-distinct-guard
kind: story
status: implemented
title: Two input refusals are witnessed together when one guard is distinct over a list
tags:
- adopter-report
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/refusal_pair_overlap.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:41Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:42Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-13T18:27:26Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

When two input-guarded refusals can hold together and one guard is a `distinct` predicate over a
List input, synthesis sends one request at their overlap and expects the declared-first refusal,
as it already does when the guard is a `count` comparison.

## Problem

Both guards read only the input, so this is the path beyond10x/ess#455 covers
(`refusal_pair_overlaps`, `crates/verify/ess-conformance/src/synthesize.rs:18243`, searched through
`overlap_witness` :18162 and `first_candidate` :9405), not the stored-row path of
`story:overlap-witnesses-per-phase`. The candidate ladder finds an input in the overlap for a
`count` guard and none for a `distinct` guard.

Neutral model, domain `demo.cfg`, format `ess/32`:

- Type `demo.cfg.Item`: struct with `name: String`.
- Entity `demo.cfg.Config`: identity `account_id: demo.cfg.AccountId`, field `site: Optional<String>`.
- Command `demo.cfg.Create`, inputs `account_id: demo.cfg.AccountId`, `site: Optional<String>`,
  `items: Optional<List<demo.cfg.Item>>`; outcomes in order:
  - `missing`: `when: {any: [{site: {exists: false}}, site == ""]}`, error `Missing`;
  - `duplicate`: `when: {all: [defined(items), {not: {distinct: {in: items, as: i, by: i.name}}}]}`,
    error `Duplicate`;
  - `created`: creates `Config`, sets `site: input.site`.

On ess 0.92.0, `ess verify conform synthesize --path model.yaml --target ir` writes 3 scenarios:
`missing` sends `site: ""` with `items: []`, `duplicate` sends `site: "site"` with a repeated
name, and none sends both, so the precedence of `missing` over `duplicate` is never pinned.
`ess verify conform mutate --target interpreted --class precedence-swap` reports
`unwitnessed precedence-swap/demo.cfg.Create/missing/duplicate … ESS-MUTATE-004: the suite does not change`, exit 3.

Control: with the `duplicate` guard written `{all: [defined(items), items.count > 1]}`, the
`missing` scenario also sends `site: ""` with two items of the same name requiring `missing`, and
`mutate` reports the precedence-swap mutant killed, exit 0.

## Acceptance

- A synthesis test over the model: the `demo.cfg.Create/outcome/missing` scenario has a send with
  `site: ""` and two items of the same `name`, requiring `missing` and error `Missing`.
- `ess verify conform mutate --path <model> --target interpreted --class precedence-swap` reports
  `precedence-swap/demo.cfg.Create/missing/duplicate` killed, exit 0.
- The `count` control keeps its overlap send and its killed mutant.
- A `distinct` overlap synthesis cannot witness is named in a note (`Note::UnwitnessedOverlap`).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/witness.rs
- cited: crates/verify/ess-conformance/tests/refusal_pair_overlap.rs

confidence: medium
