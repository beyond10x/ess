---
format: aep.planning-md/3
id: story:a-comparison-over-an-optional-field-a-row-holds-absent-is-decided-or-named-by-validate-and-not-left-unknown-for-the-interpreter-to-refuse
kind: story
status: draft
title: A comparison over an Optional field a row holds absent is decided or named by validate, and not left Unknown for the interpreter to refuse
summary: A comparison over an absent Optional is Unknown in a row-set selector, a related predicate, a subject predicate and an affects filter while defined() reads False; validate accepts it and the interpreter answers Unsupported
tags:
- downstream-report
- gap
- issues-2026-11-26
relations:
- serves: vision:O2
- informed_by: story:an-absent-optional-under-false-and-unknown-is-arranged
revision: 2
---
## Need

A comparison over an `Optional` field that a row holds absent (written `{cleared: true}`) is Unknown in a row-set selector, a related predicate, a subject predicate and an `affects:` filter, while `defined()` on the same field reads False. `ess specify validate` accepts the guard, and the interpreter answers `Unsupported` for every input that meets such a row.

Reduction (downstream gap 25, read from source, not run): an entity with an `Optional` link field `parent`; a command whose row-set selector reads `where: {all: [parent == input.parent, name == input.name, state == Open]}`; one stored row holds `parent` absent. `validate` exits 0. Any input of that command that reaches the row-set read makes the row's membership Unknown, and the interpreter answers `Unsupported` for it.

Wanted (the reporter's): the comparison reads False against an observed absence, or `validate` names the guard.

## What main does today (origin/main 14248d409e)

- A comparison with an operand that resolves to no value is Unknown: `crates/specify/ess-primitives/src/predicate.rs:2231-2245` (`evaluate_compare`, `left.resolve(facts)` returning `None` gives `Truth::Unknown`).
- That is the documented rule, with its idiom: "`evaluate_compare` answers `Unknown` when an operand does not resolve ... an unknown fact selects no branch and never the default ... An author who wants `None` to select something writes `not defined(field)`. No new refusal" (`docs/design/cross-record-and-stored-field-guards.md:185-188`). Also `docs/design/expression-family-source22.md:525` (absent, unobserved or `null` text: comparisons are Unknown) and `docs/design/review-expression-typechecking.md:98` ("Optional scalar absent keeps Defined False and comparison Unknown").
- Known absence is a separate notion where the history checker holds it: `docs/design/generated-history-values.md:79` (`Cleared`: known absence) and `:41` (`observed_presence`, so `defined` is Unknown only where presence is unobserved). `docs/design/explorer-optional-unknown.md:19-21` keeps known absence known through payload comparison and view comparison. Neither makes a comparison with a known-absent operand False.
- A row the selector leaves Unknown stays a possible member, "never dropped", and a read it leaves undecided decides nothing: `crates/verify/ess-conformance/src/interpret/execute/row_set.rs:5-8`; `Selected::Unknown` becomes `Undetermined::Undecidable` at `:252`, and an ordered head with an unknown member is undecidable at `:286-293`. `docs/design/filtered-related-reads.md:52-55` ("Unknown rows are retained as possible members, never dropped"). A view filter with an Unknown answer is `unsupported` at `crates/verify/ess-conformance/src/interpret/views.rs:214,223`.
- `validate` has no check that a comparison over an `Optional` field in those four positions meets a presence test: the finite prover "declines `Optional` paths" (`cross-record-and-stored-field-guards.md:187-188`).
- `--strict-requires` does not touch guards: it refuses a newer release than the one a specification pins (`crates/edge/ess-cli/src/requires.rs:1-22,120`). It was named in the report only as the flag the check ran under.
- Store search (`Unknown`, `absent`, `cleared`, `defined(`, `strict-requires`): `story:an-absent-optional-under-false-and-unknown-is-arranged` and `story:a-plain-comparison-beside-an-input-presence-test-is-witnessed` (synthesis arranges an absent Optional for the False and Unknown cases), `story:an-absent-payload-read-is-claimed-absent-in-either-spelling-not-as-null`, `story:a-branch-may-clear-the-field-it-owns` and `story:a-view-member-an-outcome-clears-is-claimed-by-the-suite-and-published-by-the-interpreted-target-in-the-member`. None changes what a comparison with an absent operand answers or asks `validate` to name it.

## Decisions

Open, lettered. Recommended: B.

- A: a comparison with an observed absence reads False. Cost: a change of meaning, so a new source format (the next open one) and no change below it; `!=` and `not (f == x)` over an absent field become True where they are Unknown today, which contradicts the page above; every target reads it (Rust, Go, TypeScript, the interpreter, synthesis, `ess verify diff`); an unobserved absence stays Unknown, so the interpreter must tell the two apart (the `observed_presence` seam of `generated-history-values.md`). Fails open: a row holding an absence is silently not selected.
- B: `validate` names such a guard. A comparison over an `Optional` field in a row-set selector, a related predicate, a subject predicate or an `affects:` filter, with no presence test of the same path in the same conjunction, is refused (a warning below the format that admits the refusal), and the refusal's hint names the idiom `defined(f) and f == x` (the conjunction is False where `f` is absent and the comparison Unknown, by the Kleene rule the evaluator already applies: to be shown by a test). Cost: refuses documents that validate today, so it is format-gated; it over-refuses where every writer sets the field, unless the finite prover is used to waive it. Fails closed.
- C: both. A decides the semantics, B names the guard below the format that carries A.

## Acceptance

- The reduction above: on main `validate` exits 0 and the interpreted suite answers `Unsupported`; the red test pins both.
- Under A: the same model runs, the row with the absent field is not selected, and `validate` exits 0; `!=` over an absent field and an unobserved absence keep their documented answers in the cases the format does not change.
- Under B: `validate` exits non-zero on the reduction naming the field, the position and the idiom; the guard with `defined(parent) and` in front validates and the interpreted suite decides it; a field no writer leaves absent validates unchanged.
- Under every option: below the new format the answers and the IR bytes of existing documents do not move; `docs/design/cross-record-and-stored-field-guards.md` and `website/docs/reference/predicates.md` state the rule.

## Fit review

1. Need: a row holding a known absence must neither stall the interpreter nor be a hidden trap in a selector; proposed by the reporter, as the first wanted form, a comparison reading False.
2. Class: gap in what `validate` says (a guard the interpreter can never decide is accepted); the Unknown answer itself is documented behaviour, not a defect.
3. Already expressible: yes for the guard (`defined(f) and f == x`, the page's own idiom); not for the author learning it, because nothing names it.
4. Fit: grid R (what can be read where) puts the same Optional read in four positions, so one rule serves all four, not one (`docs/design/language-grids.md`).
5. A second adopter: a line item with an `Optional` category, selected by a command matching `category == input.category`.
6. Cost: B adds no key and no code if an existing code is reused, and is format-gated; A adds a meaning and every target pays.
7. Considered: change nothing (rejected: an accepted guard that cannot run); A; B; C.

## Scope

- `crates/specify/ess-primitives/src/predicate.rs`, `crates/specify/ess-domain/src/command/row_set.rs`, `related_guard.rs`, `subject_fact.rs` (checks of the four positions).
- `docs/design/cross-record-and-stored-field-guards.md`, `website/docs/reference/predicates.md`.

## Status

Pull request 842 (https://github.com/beyond10x/ess/pull/842) is open and not delivered (2026-11-27): red test `f016bb9345` and fix `90b61d2a6a`; a refusal from `ess/38`, a warning below. Not in any release.
