---
format: aep.planning-md/3
id: story:optional-guards-mean-what-they-say
kind: story
status: implemented
title: An Optional guard is refused or witnessed as its author meant
refs:
- provider: github
  reference: beyond10x/ess#93
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-25T21:35:06Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-25T21:35:38Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-02T09:23:39Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"verification":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

A guard that reads an `Optional` field means what its author meant: `x == null` is refused at
validate with a hint, and both sides of a presence guard (`defined(x)` / `not defined(x)`) get a
synthesized scenario.

## Why

GitHub issue beyond10x/ess#93 (`ess` 0.32.0). `when: note == null` on `note: Optional<String>`
validates and synthesizes an input carrying the four-character string `"null"`. `not defined(x)`
validates, but synthesize refuses with `ESS-SYNTH-003`, because witness rule 1 fills every optional
(`crates/verify/ess-conformance/src/witness.rs:17-19`).

## Decision (operator default, 2026-09-25)

`== null` / `!= null` against an unquoted `null` are **refused** at validate with a hint naming
`defined(x)` / `not defined(x)`. A quoted `"null"` stays a text literal. No absence semantics are
added to `==`.

## Acceptance

- Red first: validate refuses `note == null` and `note != null` with a stable code and a hint naming
  `defined(note)`; a quoted `"null"` still validates as text.
- For each `Optional` path a guard reads through `defined`/`exists`, synthesis tries one candidate
  with that path omitted, bounded like rule 3. `defined(note)` / `not defined(note)` both get a
  scenario; a case asserts both.
- Witness rule 1's doc comment states the new exception.
- "Absent" vs "present but null" on the wire: the implementation states which the omitted
  candidate is, in the witness module doc.

## Out of Scope

Absence semantics for `==`; `when_subject` over Optional fields (#75).

## Reconciliation 2026-10-02

Released implementation; active was stale. Commit eff666c72 is an ancestor of published 0.51.0 (tag 0347ffa222939e3791e574d2dbe42d4b4b02d979). Source and named regressions are unchanged at the inspected release/current heads.

Acceptance mapping: ess-cli/tests/a1_guard_adversary.rs::an_unquoted_null_comparison_is_refused_at_validate_with_a_code_and_the_presence_hint checks ESS-SPEC-017 and presence repair. ess-conformance/tests/synthesis.rs::an_unquoted_null_guard_is_refused_at_validate_and_a_quoted_one_is_the_text holds quoted text compatibility. every_presence_list_and_text_ordering_guard_gets_a_satisfying_and_a_refuting_scenario checks both presence outcomes, and the_absent_side_of_a_presence_guard_omits_the_optional_field_from_the_input checks omission rather than JSON null. witness.rs:19-25 documents the exception, omission bound and wire rule.

Retained actual red evidence: local-evidence:ess-wave-20260925/a1-scratch/red.log records those cases failing before the correction. final2.log and cli2.log record corrected passes with EXIT=0; cli2 has eight passes. The 2026-10-02 complete conformance package run also passes the named synthesis tests. The historical serialized-predicate regeneration qualification remains as documented; this closure does not broaden the source-level decision to compatibility with every old persisted spelling.

The published 0.51.0 release was verified public with four native archives and SHA256SUMS. GitHub issue93 is already closed; no new remote issue action is needed.
