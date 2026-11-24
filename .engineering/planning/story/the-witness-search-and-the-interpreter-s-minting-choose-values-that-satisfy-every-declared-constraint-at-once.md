---
format: aep.planning-md/3
id: story:the-witness-search-and-the-interpreter-s-minting-choose-values-that-satisfy-every-declared-constraint-at-once
kind: story
status: draft
title: 'The witness search and the interpreter''s minting choose values that satisfy every declared constraint at once: a type''s invariants, its alphabet, and the negation of every input refusal a branch sits beneath.'
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#804
- provider: github
  reference: beyond10x/ess#809
relations:
- serves: vision:O2
revision: 2
---
# Generic feature: the witness search satisfies every declared constraint

Umbrella, not built itself. Its units are split by cause, because the causes sit in different
functions:

| unit | issue | function (crates/verify/ess-conformance/src/) | size |
|---|---|---|---|
| `d809-a-guard-read-text-input-gets-a-candidate-its-invariant-admits` | #809 | `witness.rs`, `invariant_ladders` (5282-5328) | S-M |
| `d804-a-missing-row-branch-is-sent-an-input-outside-every-input-refusal` | #804 | `synthesize/related_guard.rs`, `unclaimed` (5392-5448) | S |
| `d816-a-generated-value-of-a-length-constrained-newtype-is-minted`: an amendment of the active `story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes`, not a unit | #816 item 4 | the mint site that story picks (`interpret/values.rs::mint` 60-93 → `witness.rs`, `fields` 1203-1229, or `response.rs` `text_choice`) | S added |

Already covered by active stories, not repeated here:

- #734, `story:a-guard-comparing-a-stored-list-with-an-input-list-is-arranged`: grounds hinted or
  spread creation candidates against the creating branch's own input guards, in `make_creations`
  (`synthesize/subject_fact.rs` about 4992-5080); tests in `tests/subject_field_in_input_list.rs`.
- #726, `story:per-state-refusals-are-witnessed-when-the-moving-guard-reads-an-input-list`: passes
  the moving branch's own stored guard to the per-state input search, in `refusal_input_from` /
  `grounded_over_input` (`subject_fact.rs` about 3214-3412); tests in
  `tests/wrong_state_own_guard_over_input_list.rs`.
- Neither touches `witness.rs`, `Directed::goals`, the invariant ladders or `unclaimed`. The triage's
  "non-empty stored lists" constraint is theirs.

## Need

Synthesis and the interpreted target draw values for inputs and minted fields from a search that
honours some declared constraints and not others. A value the model's own types admit and the
branch's guards need exists, yet the branch is refused (ESS-SYNTH-003) or the scenario is
unsupported, and the result depends on an input's name (#804) or on whether an `example:` is
written (#809). One adopter lost 54 of 113 scenarios (#809); another 40 of 123 (#816 item 4).

## Members

- #809: a guard-read `Optional<Code>` input, `Code` a `String` newtype with an alphabet and
  `value.count == 1`: the one candidate breaks the invariant. Reproduced on 0.98.0.
- #804: an `exists: false` branch beneath four input refusals needs an input two leaves off the
  base: no goal asks for every refusal refuted at once. Reproduced on 0.98.0, and renaming the input
  `writer` to `from` makes it witnessed on 0.98.0.
- #816 item 4: `{generated: true}` of a `String` newtype with `value.count == 52` cannot be minted
  by the interpreted target. Reproduced on 0.98.0 (`repro/816-4/`: 2 scenarios, 2 unsupported,
  "its type has no shape the interpreter can mint").
- #734 and #726: active stories above.

## Order

d809 is in the hotfix wave the owner ordered on 2026-11-24 (03:10 CET); d804 and the d816
amendment go into the one pull request of every other open issue. d816 is
folded into `story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes`, so
`witness::fields` has one builder: that story's implementor, who picks the mint site once and
covers the positive text rules and the length rules there. d809 (`invariant_ladders`, 5282) lands
first; the story with the amendment reuses its ladder (`fields`, 1203, same file, far apart). d804 touches `related_guard.rs` near the active
story `synthesis-arranges-the-rows-a-when-related-guard-names-through-a-member-of-a-struct-input-and-the-interpreted`,
which rewrites the `insert` at `unclaimed:5418`: whichever lands second rebases.

## Fit review

1. Need: a value that satisfies the declared types and the branch's guards must be found when one
   exists, independent of names. Proposed syntax: none (the requesters' workarounds were
   `example:` and an `UNMAPPED:` marker).
2. Class: defect for each member. `website/docs/reference/diagnostics.md:197`: ESS-SYNTH-003 is
   "Every candidate this synthesizer knows how to try was refuted"; #809's refusal names a candidate
   its own type refuses, and #804's result changes with a name.
3. Expressible: `example:` on the input (#809), an `example:` equal to the caller's value (#804);
   both move a checked figure out of the search and are per-input workarounds.
4. Fit: no authored surface; synthesis and interpreter only.
5. Second adopter: any fixed-length code (country codes, check digits) used in a guard; any command
   with three or more input refusals over different leaves above a related-row refusal.
6. Cost: suite bytes of models that are refused today (narrow fixes leave every other suite as it
   is); see each unit.
7. Considered: one unit for all (rejected: three functions, three test files, independent
   reviews); documenting `example:` as the idiom (rejected: #804 has no per-input idiom that does
   not depend on names, and the suite then checks only the example).

## Decisions

- Decision: accept as proposed (three defects), redesigned into one feature: two units (d809,
  d804) and one amendment of an active story (d816).
- DF-D2 (in `DECISIONS.md`) decides the reach of d804's fix.
