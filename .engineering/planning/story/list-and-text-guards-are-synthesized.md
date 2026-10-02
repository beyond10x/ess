---
format: aep.planning-md/3
id: story:list-and-text-guards-are-synthesized
kind: story
status: implemented
title: List and text-ordering input guards get synthesized scenarios
refs:
- provider: github
  reference: beyond10x/ess#94
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-25T21:37:13Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-25T21:37:45Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-02T09:23:40Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"verification":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

A command-input guard over a list (`.count`, `exists`, `forall`) or a text ordering (`caller < "m"`)
that passes validate also gets synthesized scenarios for both of its branches.

## Why

GitHub issue beyond10x/ess#94 (`ess` 0.32.0; same class as #74). Validate accepts
`tags.count > 0`, `exists: {in: tags, as: t, that: t == vip}`, `forall: {…}` and `caller < "m"`;
synthesize refuses each with `ESS-SYNTH-002`, because the witness builder makes every list `[]`
(`witness.rs:39`), the input flattener publishes no count or element facts for input lists, and
text has no declared scale.

## Decision (operator default, 2026-09-25)

**Synthesize**, not refuse: a one-element list built from the guard's own literal (`[vip]`
satisfies, `[]` and a list of the path's own text refute), `.count` published for input lists the
way `FactSource::cardinality` does for observed ones. Text ordering is **byte-wise lexicographic**,
identical in the Rust, Go and TypeScript evaluator lanes.

## Acceptance

- Red first: the four guards in the issue's table each get a satisfying and a refuting scenario.
- A case per lane (Rust, Go, TypeScript) asserts the same order for a pair of texts whose byte
  order differs from a locale order (e.g. `"B" < "a"`).
- The witness module doc states the list and text rules.

## Reconciliation 2026-10-02

Released implementation; active was stale. Commit eff666c72 is an ancestor of published 0.51.0 (0347ffa222939e3791e574d2dbe42d4b4b02d979). Inspected implementation and regression paths are unchanged at the release/current heads.

Acceptance mapping: ess-conformance/tests/synthesis.rs::every_presence_list_and_text_ordering_guard_gets_a_satisfying_and_a_refuting_scenario evaluates actual generated inputs for the four reported guard shapes. a_list_guard_is_satisfied_by_one_element_the_guard_itself_writes holds literal-derived list witnesses. ess-cli/tests/a1_guard_adversary.rs::list_and_text_guards_synthesize_through_the_cli holds the CLI seam. The Rust primitive_corpus, Go primitive_corpus and TypeScript predicate.test.ts byte-order cases assert locale-distinguishing text order. witness.rs:42-51 documents byte ordering, one-element lists and count projection.

Actual historical red.log contains failing branch/list witness, Rust ordering and projection cases; final2.log and cli2.log record corrected passes with EXIT=0. Go's ordering case already passed in the red run, so no Go defect or red chronology is fabricated. Retained mut-golocale.log and mut-tslocale.log show ordering discrimination. The current complete conformance package run again passes the named synthesis and primitive corpus tests.

GitHub issue94 is already closed. Release0.51.0 was verified public with four native archives and SHA256SUMS. This resolves the four reported guard shapes and declared byte-order contract, not every possible list predicate.
