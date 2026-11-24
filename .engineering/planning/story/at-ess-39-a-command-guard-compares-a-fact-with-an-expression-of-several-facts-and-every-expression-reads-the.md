---
format: aep.planning-md/3
id: story:at-ess-39-a-command-guard-compares-a-fact-with-an-expression-of-several-facts-and-every-expression-reads-the
kind: story
status: active
title: At ess/38 a command guard compares a fact with an expression of several facts, and every expression reads the length of a list as xs.count, validated, compiled and decided by the interpreted target
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:51:58Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-24T03:51:59Z", actor: "human:timo", revision: 6}
---
## Need

A command refuses a request whose entries do not cover every cell of two lists the request (or
the record it addresses) holds: "one entry for every pair of a line and a site" is checked as
`input.cells.count == lines.count * sites.count`, given that the entries are distinct and each
names a line and a site of the record (both already expressible). ESS can state neither half of the
comparison's right side:

- **In a guard**, an operand is one fact, a literal, one fact moved by one constant
  (`Operand::Offset`, legal on the right only) or one fact through functions
  (`Derived::Call`, which "reads exactly one fact", `crates/specify/ess-primitives/src/predicate.rs:640-690`).
  A product of two facts is read as text and refused as `type_mismatch` with the hint "… is read
  as text, because a comparison does no arithmetic but one constant offset of a fact"
  (`crates/specify/ess-domain/src/expression.rs:1782-1796`, raised at `:2403-2418`; expected from
  the code, not run). Grid O records the row "fact `+ - * /` fact" as `no` in every column
  (`docs/design/language-grids.md`, grid O, row 5).
- **In a value**, `{expr: "input.cells.count * 2"}` is refused: `input_path::resolve` walks struct
  members only and answers `Opaque` at a `List` (`crates/specify/ess-domain/src/command/input_path.rs:100-126`),
  and the expression reader turns that into "`input.cells.count` reads `cells.count`, which
  `<Command>` does not declare as input" (`crates/specify/ess-domain/src/command/expression_value.rs:66-90`),
  a message that is wrong about `cells`, which is declared. The design page lists `count(xs)` in
  the expression language's collection family (`docs/design/generic-language-features.md:98`); the
  shipped language has no such function (`crates/specify/ess-primitives/src/expr.rs:160-224`) and
  no length read.

Reported by a downstream application on 0.98.0 (sixth list, item G12, rank P2): a refusal
"incomplete coverage" stays an `external:` cause, so no scenario checks the figure the reader
sees, the number of cells covered.

Minimal reproduction (fresh, generic names, not run here). Entity `demo.plan.Plan {plan_id,
lines: List<demo.plan.LineId>, sites: List<demo.plan.SiteId>}`; command `demo.plan.Admit
{plan_id, cells: List<demo.plan.CellKey>}`:

```yaml
outcomes:
  - name: cells-repeat
    when: {not: {distinct: {in: cells, as: c}}}
    error: demo.plan.CellsRepeat
  - name: incomplete
    when_related:
      via: input.plan_id
      predicate: "input.cells.count != lines.count * sites.count"
    error: demo.plan.Incomplete
  - name: admitted
    moves: demo.plan.Plan.admit
    instance: plan_id
```

Today the `incomplete` predicate is refused (the arithmetic hint above). The `{expr: "input.cells.count
* 2"}` form in an event payload is refused with the misleading `undeclared_reference`.

## Outcome

At `ess/39`:

- **A guard operand may be an expression of several facts.** On either side of a comparison in a
  command guard (`when:`, `when_subject:`, the predicate of `when_related:`), an operand may be any
  well-typed expression of the shared language reading one or more facts of the roots that
  position reads, written unquoted as today's function calls are (`challenge !=
  base64url(sha256(…))`), canonically `{expr: "<expression>"}`. `Derived` gains a variant for an
  expression reading two or more facts, sharing `Call`'s canonical form; an expression reading one
  fact stays `Call`, so its IR bytes and every caller of `Derived::parent()` (synthesis, the
  witness, Entity Runtime lowering) are unchanged, and each exhaustive `match` on `Derived` gets an
  arm. The comparison is typed by the expression language's result
  type: `+ - *` over Integers an Integer, over Decimals a Decimal, `/` an `Optional<Decimal>`
  (so it needs `coalesce`, as in values), `Timestamp - Timestamp` a Duration (SX-D5 A). An
  operand outside the bound of signed 128 bits is `Unknown`, never false.
- **Every expression reads a length.** `<path>.count` of a `List<…>` or `Map<…>` (through any
  newtype; an `Optional` list reads as in a guard) is an Integer read in every expression position:
  `{expr: …}` in `sets:` and `payload:`, a computed view field, a measure input, and the guard
  operand above. This is the spelling guards have read since `ess/11` (`language-grids.md`, grid O,
  row "length"); `count(xs)` is not added, and the design page's table is amended to say `.count`
  (SX-D6 A). A declared member named `count` keeps meaning that member.
- **Refused by name, with why.** The same operand in a row-set or selector `where:`/`forall:`, an
  `instances:`/`affects:` filter, an invariant, a view `filter:` and a measure `where:` is refused
  as `unsupported_construct`, "an expression of several facts is decided in a command guard in this
  cut; generated queries do not evaluate it", as the digest functions are refused there today.
  Below `ess/39`, each new reading is refused as `unsupported_format_version` naming `ess/39`, and
  text that was a literal stays the literal it was.
- **The misleading message is fixed at every format.** A path that walks past a list or a map names
  the segment it cannot read: "`input.cells.count`: `cells` is `List<demo.plan.CellKey>`, which
  has no member `count`; a list's length is read as `input.cells.count` from ess/39".
- **The interpreted target decides it.** The interpreter evaluates the operand with the shared
  evaluator (`Expr::evaluate`), as it evaluates `Derived::Call` today.
- The grids page records grid O row "fact `+ - * /` fact", Integer and Decimal columns, and row
  "`Timestamp - Timestamp`", as `yes` in command guards from `ess/39`, and the length row's List and
  Map columns in expressions. The CHANGELOG entry carries its `Language:` count as **part** of G1
  (the value positions opened in `ess/27`).

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- First: a case in the existing target `crates/specify/ess-domain/tests/expression.rs` (no new
  test binary) with the reproduction at `ess/39`. Today it is refused (`type_mismatch`, the
  arithmetic hint). After: it validates, and the compiled predicate of `incomplete` holds an
  `{expr: …}` operand; a pin in `crates/specify/ess-compiler/tests/expression_validation.rs` holds
  the IR bytes.
- The same document at `format: ess/38` is refused as `unsupported_format_version` naming `ess/39`,
  at the predicate.
- `when: "total != input.price * input.qty"` (Integer inputs) validates; with `input.price` a
  Decimal and `input.qty` an Integer it validates as Decimal; `input.a / input.b` against an Integer
  is refused as `type_mismatch` naming `coalesce`; a String operand is refused naming the types.
- The same predicate in a row-set `where:`, an invariant and a view `filter:` is refused as
  `unsupported_construct` with the sentence above, at each site.
- `{expr: "input.cells.count * 2"}` in an event payload validates at `ess/39` into an Integer
  field and is refused below it; at `ess/38` the refusal names `cells`'s type and `ess/39`, not
  "does not declare as input".
- A computed view field `{name: n, type: Integer, expr: "lines.count"}` validates at `ess/39`.
- A struct input with a member named `count` keeps reading that member in a guard and in `{expr}`;
  a test pins it.
- The interpreted target answers `incomplete` for a plan with 2 lines, 3 sites and 5 cells, and
  `admitted` for 6 distinct cells, in a case of `crates/verify/ess-conformance/tests/expression_arithmetic_and_time.rs`
  that drives the interpreter with hand-written steps (synthesis is `sx12-arithmetic-witness`).
- A model that reads no several-fact operand and no `.count` in an expression keeps its IR bytes
  and digest: `an_old_model_keeps_its_bytes` over the repository's examples.
- `docs/design/language-grids.md` and `docs/design/generic-language-features.md` (collection
  family, `.count`) are updated in the change; `website/docs/reference/predicates.md` documents the
  operand; `spec-versions.md` and `formats.md` describe `ess/39`'s addition.
- The byte pins listed in `AGENTS.md` § Gate are run and unchanged; `task test-xtask` passes (a
  new message text).
- Each new test is timed and expected under 5 s; none may run over 60 s, each crate's test step
  stays within 120 s, and each build-pool job within 10 minutes.

## Fit review

Verdict: **accept, redesigned**. Tree `bcfc51555`, read-only.

1. **Need.** A coverage figure the reader sees, "every cell of two lists is covered", is checked by
   the specification rather than named as an external cause. Requester's syntax (theirs):
   `input.<list_a>.count * input.<list_b>.count` inside `{expr: …}`. The place they need it is a
   refusal guard.
2. **Class.** Gap: grid O row "fact `+ - * /` fact" is closed in guards; the length row is closed in
   expressions. Also a message defect (the `undeclared_reference` text).
3. **Already expressible?** No. The idiom the application uses, an `external:` cause, moves the
   checked figure out of the specification, which policy 1 rules out as an answer
   (`generic-language-features.md:588-594`).
4. **Fit.** Grid O row "fact `+ - * /` fact" (Integer and Decimal) and row "`Timestamp - Timestamp`",
   in the three command guard positions of grid R; and grid O row "length", List and Map columns,
   in every expression position. The design page's target already opens both ("arithmetic applies
   to Integer and Decimal everywhere, not on the right of a comparison only",
   `language-grids.md`, grid O, Target): this is no new decision, it is G1's remaining guard cells.
   It reuses the existing operand (`Derived::Call`) and canonical form, the guard positions where
   function calls are admitted, the length spelling guards already read, and the evaluator values
   use. The positions left refused are refused whole, by name, with the reason (generated queries
   and selector synthesis), as the digest functions' were.
5. **Second adopter.** "A seat map holds rows × columns seats" (`seats.count == rows * columns`);
   "a quote's total equals price × quantity" (`total != input.price * input.qty`); an order's
   allocation equal to the sum of two inputs.
6. **Cost.** A source format: `ess/39` (not `ess/38`, which the wave of 2026-11-24 opens for the
   grid R row "an address names a member of a struct input"; this row is not that row, policy 2).
   No new keyword, no new `ValidationCode`. Synthesis cost is `sx12-arithmetic-witness`; generated
   targets `sx12-arithmetic-targets`. The count of the language policy: part of G1.
7. **Considered.** (a) Change nothing: the figure stays external. (b) A count-product construct for
   this request: one cell, policy 2 refuses it. (c) The coverage statement itself, "for every line
   and every site an entry exists" (SX-D4 B): the domain fact, with no reliance on distinctness;
   it needs a rows quantifier under two list quantifiers, which the first report's map named for
   the quantifier design of report item 9 (the first report's P2B map). Chosen: the arithmetic operand
   and the length read (SX-D4 A), which is the requester's idiom and is sound only beside the
   distinctness and membership guards the reproduction shows; the reference says so.

Covers sixth list, item G12. Touches report item 14 of the first report (dominance over keyed
lists, `g14-keyed-list-compare`) only by sharing the guard positions; it does not cover it.

## Decisions

- **SX-D4 (decided: A, 2026-11-24). What item G12 opens.** (A) Arithmetic over several facts in command guards and a list's
  length in every expression (the requester's count-product idiom). (B) The coverage quantifier
  "for every pair of two lists a row exists", designed with the quantifier work of report item 9.
  (C) Both, A now and B later. (D) Decline: the application keeps the `external:` cause.
  **Recommended: (A).**
- **SX-D5 (decided: A, 2026-11-24). Which guard positions.** (A) `when:`, `when_subject:` and the `when_related:` predicate;
  every selector, filter, invariant and view position refused by name. (B) Every predicate position
  at once, with selector synthesis and generated queries in the same decision. (C) `when:` only.
  **Recommended: (A).**
- **SX-D6 (decided: A, 2026-11-24). How an expression spells a length.** (A) `xs.count`, as guards spell it since `ess/11`.
  (B) `count(xs)`, as the design page's table writes it, `.count` staying guard-only. (C) Both.
  **Recommended: (A).**

## Scope

- cited: `crates/specify/ess-primitives/src/predicate.rs` (`Derived` `:656-670`, `Derived::call`
  `:685-688`: the several-fact variant, its `value`, canonical form and `fact_paths`)
- cited: `crates/specify/ess-primitives/src/expr.rs` (reads of `.count` typed Integer; evaluation
  through `SourceReads`)
- cited: `crates/specify/ess-domain/src/expression.rs` (`resolve_lexical_reading` `:1500-1563`,
  `left_calls` `:1569-1610`, `scaled_parameter` `:1782-1796`, the comparison check `:2403-2418`)
- cited: `crates/specify/ess-domain/src/command/expression_value.rs` (`Roots::read` `:66-90`),
  `crates/specify/ess-domain/src/command/input_path.rs` (`resolve` `:100-126`)
- inferred: `crates/specify/ess-domain/src/view.rs` (a computed field's and a measure input's
  `.count` read), `crates/specify/ess-domain/src/system.rs` (the `ess/39` rows)
- cited: `crates/verify/ess-conformance/src/interpret/` (evaluation of the operand)
- cited: `docs/design/language-grids.md`, `docs/design/generic-language-features.md`,
  `website/docs/reference/predicates.md`, `spec-versions.md`, `formats.md`, `CHANGELOG.md`
- inferred: cases in `crates/specify/ess-domain/tests/expression.rs` and `expression_values.rs`,
  `crates/specify/ess-compiler/tests/expression_validation.rs`,
  `crates/verify/ess-conformance/tests/expression_arithmetic_and_time.rs` (no new test binary)
- Size: L, about 580 changed lines.

## Depends on

- The unit that opens `ess/39` as the newest format with no construct of its own
  (`g15-ess39-opens` of the first report's P2B group, or this unit's own first commit if it lands
  first; one opener only), which itself follows the wave of 2026-11-24's `ess/38`.
- `sx12-arithmetic-witness` and `sx12-arithmetic-targets` depend on this.
