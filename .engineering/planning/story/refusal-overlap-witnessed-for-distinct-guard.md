---
format: aep.planning-md/3
id: story:refusal-overlap-witnessed-for-distinct-guard
kind: story
status: draft
title: Two input refusals are witnessed together when one guard is distinct over a list
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

When two input-guarded refusals can hold together and one guard is a `distinct` predicate over a
List input, synthesis sends one request at their overlap and expects the declared-first refusal,
as it already does when the guard is a `count` comparison.

## Evidence

An adopter on ess 0.56.0. Both guards read only the input, so this is the path 32afe8ef40 covers
(`refusal_pair_overlaps`, `crates/verify/ess-conformance/src/synthesize.rs:13736-13758`), not the
held-state and stored-row paths of `story:overlap-witnesses-per-phase`. Neutral model to write
for the test, domain `demo.cfg`:

- Type `demo.cfg.Item`: struct with `name: String`; a conversion `Optional<String>` to `String`.
- Entity `demo.cfg.Config`: identity `account_id`, field `site: String`.
- Command `demo.cfg.Create`, inputs `account_id: demo.cfg.AccountId`, `site: Optional<String>`,
  `items: Optional<List<demo.cfg.Item>>`; outcomes in order:
  - `missing`: `when: {any: [{site: {exists: false}}, site == ""]}`, error `Missing`;
  - `duplicate`: `when: {all: [defined(items), {not: {distinct: {in: items, as: i, by: i.name}}}]}`,
    error `Duplicate`;
  - `created`: creates `Config`, sets `site: input.site`.

`ess verify conform synthesize --path <model> --target ir` wrote 3 scenarios: `missing` sends
`site: ""` with no items, `duplicate` sends `site: "site"` with duplicate names, and none sends
both, so the precedence of `missing` over `duplicate` is never pinned.

Control: with the `duplicate` guard written `items.count > 1`, the `missing` scenario also sends
two items, so the overlap witness exists. The overlap witness is built for a `count` guard and not
for a `distinct` guard.

## Acceptance

- A synthesis test over the model: one scenario sends `site: ""` with two items of the same name
  and expects `missing` and its error; the precedence-swap mutant of the two refusals is killed.
- The `count` control keeps its overlap scenario.
- A `distinct` overlap synthesis cannot witness is named in a note.
