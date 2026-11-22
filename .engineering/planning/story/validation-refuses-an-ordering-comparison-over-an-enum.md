---
format: aep.planning-md/3
id: story:validation-refuses-an-ordering-comparison-over-an-enum
kind: story
status: draft
title: Validation refuses an ordering comparison over an enum instead of letting every evaluator order its variant names by bytes
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli/tests/
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/enum_variant_refusals.rs
- confidence: cited
  path: docs/design/language-grids.md
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 7
---
## Outcome

`ess specify validate` refuses `<`, `<=`, `>` and `>=` where either operand is an enum, ordered
or not, as `type_mismatch` naming the enum, at every predicate site (`when:`, `when_subject:`,
`when_related:` `predicate:` and `where:`, view `filter:`). No model validates whose guard the
interpreter, synthesis and authoring then decide by comparing variant names as UTF-8 bytes.
`==`, `!=` and `in` over an enum keep validating. So do ordering comparisons over Integer, Decimal,
Binary64, Timestamp and String.

## Need

Reproduced on the released 0.97.1 binary, and on 0.97.0. Nothing in `CHANGELOG.md` up to 0.97.1
touches it. Generic model `d1/v-gt`, enum `shop.order.Level` with `ordered: true` and
`variants: [low, medium, high]`. The byte order of the names is `high < low < medium`.

```yaml
- name: below
  when_related:
    entity: shop.order.Floor
    where: minimum > input.level
    exists: true
  error: shop.order.Below
- name: placed
  creates: shop.order.Order
  ...
```

The scenario sets a floor with `minimum: medium`, then calls `PlaceOrder(level: low)`, expecting
`below`, and `PlaceOrder(level: high)`, expecting `placed`.

```console
$ ess specify validate --path .
shop v1 — 2 file(s), 1 scenario(s), valid
$ ess verify conform author --path . --scenarios scenarios --out suite.json
1 authored scenario(s) from 1 file(s), 0 refusal(s), suite ess-conformance/34, written to suite.json
$ ess verify conform run --path . --suite suite.json --target interpreted --report-format 2 --report-out report.json --format json
exit 1
  failed outcome shop.order.PlaceOrder/placed | expected outcome = placed observed outcome = below | shop.order.PlaceOrder(level = "high", order_id = "o2")
```

Boundary, each varying one thing (all outputs in `run.txt`):

| variant | result |
|---|---|
| `>`, `>=`, swapped `input.level < minimum`, `input.level <= minimum` | wrong by byte order: `medium > high` holds |
| literal `where: minimum > high` | holds for every level: `below` where `placed` is expected |
| enum without `ordered: true` | validates too, same wrong answer |
| `when_subject: {predicate: level <= input.level}` on the addressed row | wrong: `high <= medium` holds, `not-lower` instead of `lowered` |
| input guard `when: {level: {lt: medium}}` | authoring refuses the correct scenario (`ESS-AUTHOR-041`: "`below` (level < medium) answers it first") |
| input guard `when: {level: {lt: high}}` | synthesis: `ESS-SYNTH-003` "no candidate of the 3 tried satisfies `level < high`" |
| `==` (control) | passes |
| the same guard over an Integer field (control) | passes |
| variant names whose byte order equals declaration order (`a_low`, `b_medium`, `c_high`) | passes, by accident |

## Cause

Verified by reading the code and by the boundary above.

- Validation accepts the comparison. `Checker::compare`
  (`crates/specify/ess-domain/src/expression.rs:3065`) checks only that both sides have the same
  scalar kind (`:3097-3100`), and an enum shares String's text kind. The only ordering refusals
  are Boolean (`:3100`) and Duration (`duration_ordering`, `:3117-3118`, body `:2800-2815`). Nothing
  reads `ValueType.variants` (`:2097`) for an ordering operator.
- Every evaluator then orders the variant names as text by bytes. The shared evaluator
  (`crates/specify/ess-primitives/src/predicate.rs:2300-2320`) compares two texts with `str::cmp`
  wherever the fact source says `orders_text_by_bytes`. The interpreter's fact sources say so for
  every path that is not a Duration: `crates/verify/ess-conformance/src/input.rs:173-175` and
  `:1407-1412`. The doc comment there reads: "every ESS lane orders text by bytes — but a
  `Duration` has no ordering (validate refuses one)".
- The design says the comparison is refused. `docs/design/generic-language-features.md:446-448`:
  "An ordering comparison of an ordered enum in a guard or a filter stays refused; grid O's
  comparison row is not opened here." `:720-721` (answer 5): "`min`/`max`/ordering over an
  unordered enum is a validation refusal naming the flag." `docs/design/language-grids.md:68`
  still has "I don't know" in the enum cell of the ordering row.

The faulty component is validation. The interpreter, synthesis and authoring all read the shared
evaluator and agree with each other. The generated Go and TypeScript evaluators order text by
bytes (`predicate.rs:2303-2304`; inferred, not run). `ess verify conform mutate` replays against
the interpreter (inferred). The Entity Runtime lowering lists an ordering over a non-primitive as
one it cannot express (`crates/generate/ess-entity-runtime/src/lib.rs:4230-4238`; inferred).

## Fix

- Add `enum_ordering` in `crates/specify/ess-domain/src/expression.rs`, beside `duration_ordering`,
  and call it at `:3117` for `op.needs_ordering()`. It refuses when either operand's
  `ValueType.variants` is `Some`, with `ValidationCode::TypeMismatch`. The hint for an unordered
  enum names `ordered: true` and `in: [...]`. The hint for an ordered enum says that comparing by
  declared order is not in the language yet, and to list the admitted variants with `in: [...]`.
- About 25 lines, with no new `ValidationCode`, no evaluator change and no IR change. One check
  covers every evaluator, because none of them can be reached by a model that does not validate.
- Set the enum cell of `docs/design/language-grids.md:68` to "no (refused)". Add a sentence to
  `website/docs/reference/predicates.md` and a `CHANGELOG.md` entry.

## Acceptance

- Write the failing test first, as a case in `crates/specify/ess-domain/tests/enum_variant_refusals.rs`:
  `an_ordering_comparison_over_an_enum_is_refused`. It covers each of `<`, `<=`, `>` and `>=`; an
  enum with and without `ordered: true`; an enum behind a newtype; fact against `input.` and fact
  against a variant literal, on either side. It is refused as `type_mismatch` naming the enum.
- Controls in the same target: `==`, `!=` and `in` over the enum validate, and so do ordering over
  Integer, String and Timestamp.
- `ess specify validate` on `d1/v-gt` (written as a test fixture) exits 1 and names
  `shop.order.Level`. `ess-cli` covers it, because `ess-domain` changed.
- Gate: `ess-domain` and `ess-cli` per the package-gate table, plus `task test-xtask` and
  `cargo test -p ess-compiler --test typed_diagnostics`. The second is needed because a new
  diagnostic moves pins outside its crate.
- The repository-wide byte pins listed in `AGENTS.md` do not move. If a committed fixture or
  example orders an enum, it is named and rewritten in the same change.
- No test over 60 s, and the test step stays within 120 s.

## Fit review

1. Need: a guard that orders enum variants validates, and every evaluator then answers by the
   bytes of the variant names. A rule over a scale (a floor level, a tier) silently takes the
   wrong branch. The reporter proposes no syntax: they expect `floor > input.level` to compare by
   declaration order.
2. Class: defect. Validation contradicts `generic-language-features.md:446-448` and `:720-721`
   (cited in Cause).
3. Expressible today: `in: [...]` per level, or one outcome per floor value. That is the
   reporter's workaround, and it validates and passes.
4. Grid O, row "ordering", column "enum": this decision refuses the whole column for ordering.
   Opening it for ordered enums is the existing draft
   `story:a-guard-compares-variants-of-an-ordered-enum`, which needs a new format.
5. A second adopter: a support desk escalates a ticket whose `priority` (`low`, `normal`, `urgent`)
   is above a queue's threshold. `urgent > normal` is false by bytes.
6. Cost: one refusal. No format, no keyword, no diagnostic code, no IR. The cost is a model that
   validates on 0.97.1 and now does not, which is decided in D-1.
7. Considered: (a) change nothing: rejected, because a wrong answer stays silent in every evaluator;
   (b) order by declaration now: that is the feature story, which touches every evaluator, the
   emitters, synthesis and the Entity Runtime, plus a format; (c) refuse, chosen.

## Decisions

- Accepted as a defect, redesigned, by the planning session on 2026-11-22. The fix is the refusal
  the design already states. Comparing by declared order stays with
  `story:a-guard-compares-variants-of-an-ordered-enum`.
- D-1, open, with a default: refuse at every format, because no format gave this comparison a
  meaning. The alternative is to refuse only from the next format and keep byte order below it.
- This is a draft. No wave and no release is ordered.

## Out of scope

- Comparing by declared order (`story:a-guard-compares-variants-of-an-ordered-enum`, draft). That
  story's acceptance line "below it they are refused naming the format" is met by this story.
- Changing the shared evaluator or the fact sources' `orders_text_by_bytes`.
- A compact `where:` with `and` is read as one comparison against a text literal: `customer_id ==
  input.customer_id and blocked == true` validates as `customer_id == "input.customer_id and …"`.
  It was seen while probing and is a separate defect.

## Scope

- cited: crates/specify/ess-domain/src/expression.rs
- cited: crates/specify/ess-domain/tests/enum_variant_refusals.rs
- cited: docs/design/language-grids.md
- inferred: website/docs/reference/predicates.md
- inferred: CHANGELOG.md
- inferred: crates/edge/ess-cli/tests/
