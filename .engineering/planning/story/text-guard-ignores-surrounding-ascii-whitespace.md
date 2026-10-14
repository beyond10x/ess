---
format: aep.planning-md/3
id: story:text-guard-ignores-surrounding-ascii-whitespace
kind: story
status: draft
title: A text guard ignores surrounding ASCII whitespace
relations:
- depends_on: story:create-or-update-selects-update-branches-by-stored-fields
revision: 1
---
## Outcome

A text guard can compare a `String` fact with its surrounding ASCII whitespace removed, alone or
combined with the case-insensitive operators, so a model states that ` DONE ` is accepted as
`DONE` when the implementation trims before it checks.

Closes https://github.com/beyond10x/ess/issues/532.

## Fit review

1. **Need.** An implementation trims a code and upper-cases it before checking it against a fixed
   set. The model can say the case half (`{reason: {in_ignore_case: [NEW, HOLD, DONE]}}`, ess/15)
   but not the trim half, so the refusal guard `not: {reason: {in_ignore_case: […]}}` claims
   ` DONE ` is refused and the implementation accepts it. Requester's syntax (theirs):
   `{trimmed: {in_ignore_case: […]}}` "or an equivalent".
2. **Class: gap.** `docs/design/value-expressions.md` § E7 and
   `crates/specify/ess-primitives/src/predicate.rs:499-530` (`FoldOp`) define ASCII case folding
   only; `docs/design/string-predicate-operators.md` defines `starts_with`, `ends_with`,
   `contains` byte for byte. No operator ignores whitespace (searched `trim`/`whitespace` in both
   pages: no match).
3. **Already expressible?** Only by enumerating padded literals, which is unbounded (any number of
   spaces, tabs). `starts_with`/`ends_with` cannot express "only whitespace around". No idiom.
4. **Fit.** A normalizer that wraps a text operator, `{trimmed: <text operator>}`, composes with
   every existing text operator (`equals`, `in`, `equals_ignore_case`, `in_ignore_case`,
   `starts_with`, `ends_with`, `contains`) instead of multiplying operator names
   (`in_ignore_case_trimmed`, …). It follows E7's rule that every evaluator answers it from its
   standard library alike: ASCII whitespace only (space, `\t`, `\n`, `\v`, `\f`, `\r`), Rust
   `trim_matches` over that set, Go `strings.Trim(s, " \t\n\v\f\r")` (not `TrimSpace`, which trims
   Unicode spaces), TypeScript a fixed character class. Composes with guards (`when:`,
   `when_subject`, view `filter:` where text operators are admitted); bindings and views take it
   where they take the wrapped operator, or refuse it by name. Entity Runtime refuses it as it
   refuses folds (`CaseFoldUnsupported` precedent, E7); `infra-spec/1` refuses it; a selection plan
   refuses it as it refuses `FoldMatch`.
5. **Second adopter.** A promo code form accepts ` SPRING24 ` pasted with a trailing newline.
6. **Cost.** One new map key in the predicate grammar: source format `ess/25`; a suite carrying it
   in a view `satisfies` expectation needs the next `ess-conformance` majors (as E7 took /20 and
   /21); a `ValidationCode` reuse (`type_mismatch` on a non-text fact, `unsupported_construct`
   wrapping a non-text operator), so `macro-guards.json` and `review-typed-diagnostics.md` pins
   move only if a new code is added (none planned).
7. **Considered.** (a) change nothing: the guard stays wrong for padded input; (b) new operators
   per combination: four names now, more with every text operator; (c) the wrapper (chosen,
   close to the requester's proposal, which is taken with the whitespace set fixed to ASCII).

## Decisions

Accept, redesigned in one respect: the requester's `{trimmed: …}` wrapper is taken, with the
trimmed set fixed to ASCII whitespace so every evaluator agrees; Unicode spaces compare as
themselves.

## Acceptance

Each line names the failing test written first.

- Design section `docs/design/value-expressions.md` § E9 "Surrounding whitespace" (ess/25) states
  the key, the set, where it applies and each target's answer.
- `crates/specify/ess-primitives/src/predicate.rs` unit tests
  `trimmed_wraps_every_text_operator` and `trimmed_over_a_non_text_fact_is_a_type_mismatch`.
- `crates/specify/ess-domain/tests/trimmed_text_guard.rs`
  `trimmed_guard_validates_at_ess_25_and_is_refused_below_it`.
- `crates/verify/ess-conformance/tests/trimmed_text_guard.rs`
  `synthesis_witnesses_trimmed_case_folded_membership`: the accepting witness is a literal padded
  with ASCII spaces and a tab in another ASCII case; the refuting witnesses are the literal with
  an inner space and the literal padded with U+00A0 NO-BREAK SPACE, so a target that does not trim,
  or trims Unicode spaces, fails one scenario.
- same file, `trimmed_guard_runs_alike_on_rust_go_and_typescript`.
- `crates/generate/ess-entity-runtime` test (file inferred) `trimmed_guard_is_refused_by_name`.

## Scope

- cited: `crates/specify/ess-primitives/src/predicate.rs:495-530, 3107-3190` (fold operators and
  their reader), `crates/specify/ess-domain/src/primitive_admission.rs:473` (format admission).
- inferred: `crates/verify/ess-conformance/src/witness.rs`, `src/text_match_format.rs`,
  `src/synthesize.rs`, `crates/generate/ess-synth/src/{rust,go}/invariant.rs`,
  `crates/generate/ess-entity-runtime/src/subset.rs`, `crates/infra/infra-spec/src/raw.rs`,
  `crates/specify/ess-domain/src/expression.rs`, the TypeScript runner template.

## Plan

- depends_on: the unit that opens `ess/25`.
- du estimate: 10G.
- adversary: yes (three evaluators must agree byte for byte).
- shared surfaces: `predicate.rs` reader with no other wave m story; `primitive_admission.rs`
  with every ess/25 key.
