---
format: aep.planning-md/3
id: story:a-guard-says-what-an-absent-value-compares-as
kind: story
status: active
title: 'A guard says what an absent Optional compares as: coalesce on the left of a comparison, decided in every lane'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#620
relations:
- decomposes: epic:generic-expression-language
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T00:15:54Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":3}}}
- {from: "proposed", to: "active", at: "2026-10-21T00:15:55Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":3}}}
---
## Outcome

A guard decides a comparison over an absent `Optional`: the specification says what an absent value
compares as with `coalesce(<fact>, <default>)` as the comparison's operand, written on the left of a
compact comparison from the next source format (`coalesce(is_error, true) == false`), and every lane
decides such a record — the interpreter, synthesis (which sends the absence), and the generated Rust
and Go behaviours — the way a running implementation does.

Binding design: `docs/design/generic-language-features.md` § G1, operations table, presence row
(`coalesce(x, y)`) and the rule "`Unknown` is never silently false". Parent:
`epic:generic-expression-language`.

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/620 (a downstream specification on ess
0.66.0, 2026-10-20; reproduction in its public repository's session and spending domains).

1. **Need.** A comparison reading an absent `Optional` is `Unknown`, so a guard over records whose
   optional fields may be absent cannot be decided by the model, and synthesis refuses its
   connective's children (`ESS-SYNTH-022`). `defined(x)`/`missing(x)` beside the comparison did not
   clear the refusal: the child `x == false` cannot alone decide a conjunction where `x` is absent.
   The structured `{compare: {left: {expr: "coalesce(x, d)"}, …}}` validated from `ess/27` but every
   evaluator read the call as `Unknown` on an absent `x` (`Derived::value` mapped an unbound read to
   unknown), so it did not help either. Requester's proposals (theirs): a presence-aware comparison,
   or `coalesce` usable in a guard.
2. **Class.** Gap in the guard positions (compact spelling), and a defect in the evaluation of a
   construct already admitted (`coalesce` documented as "a where present, otherwise b").
3. **Already expressible?** Only by moving the record out of the specification into a hand-written
   test, which the policy forbids (the checked figure leaves the specification).
4. **Fit.** The design's presence row already has `coalesce`; this opens it in the guard positions
   of grid R and keeps "`Unknown` is never silently false": the author states the default. A
   presence-aware comparison operator would be a second spelling of the same thing with an implicit
   default, rejected.
5. **Second adopter.** A retry count absent until the first retry (`coalesce(attempts, 0) < 3`); an
   optional stop reason (`coalesce(stop_reason, "none") == "budget"`).
6. **Cost.** A source format for the compact spelling (`ess/29`); witness ordering in synthesis;
   a default in the generated behaviours' read.
7. **Alternatives.** (a) Change nothing. (b) A presence-aware comparison (absent compares false):
   implicit, and wrong for `!=`. (c) `coalesce` as a guard operand (chosen).

## Decisions

Accept, redesigned into G1's presence row: `coalesce(<fact>, <default>)` on the left of a compact
comparison from `ess/29`; the evaluation of `coalesce` over an absent fact fixed in every format.

## Acceptance

- `a_call_on_the_left_of_a_guard_validates_from_ess_29` and
  `below_ess_29_the_spelling_is_refused_naming_ess_29_and_the_structured_form_validates`
  (`crates/specify/ess-domain/tests/coalesce_guard.rs`).
- `coalesce_guard_absent_inputs_are_witnessed_and_decided`: synthesis sends each coalesced input
  absent, the interpreted target passes, a target reading an absent value as a present one fails,
  and the Go and TypeScript runners admit the suite
  (`crates/verify/ess-conformance/tests/coalesce_guards.rs`).
- `coalesced_guards_in_the_generated_rust_behaviour_and_the_dropped_default_fails` and its Go twin
  (`crates/generate/ess-synth/tests/coalesce_guard_behaviour.rs`).
