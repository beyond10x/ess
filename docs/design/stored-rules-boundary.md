# Rules held as data are the system's to evaluate

beyond10x/ess#451, and the pattern half of beyond10x/ess#449. No source or suite format change.

## The question

Some systems store their rules as rows: a condition is an expression, an operator and an
expectation, a rule folds its conditions with ALL or ANY, and a command tests whether a rule holds.
The rules are written at run time, not in the specification. The request was for ESS to give such
a rule semantics: a predicate stored as data and evaluated by the declared predicate semantics.

ESS does not evaluate rules held as data. Doing so would need a value type whose values are
predicates and operands addressed by a name read at run time, while every ESS path is static and
resolved when the specification is compiled (`crates/specify/ess-domain/src/expression.rs`). Every
operator of the stored grammar would need an ESS meaning. Synthesis would have to generate stored
rules and inputs that make them hold or fail, which is a search over the predicate language beyond
the finite bounds of `crates/specify/ess-domain/src/command/finite.rs`. Rust, Go, TypeScript, the
interpreter and Entity Runtime would each need an evaluator for predicates carried as data. That
is an interpreter of another system's rule language, not a domain fact the specification lacks a
way to state.

What the specification does state is the boundary, in three parts below, with a model of each in
`crates/edge/ess-cli/tests/fixtures/stored-rules/`.

## A stored rule's parts are typed rows

A condition is an entity with typed fields: the operator and the fold are enum types
(`Operator: [Equals, GreaterThan, Matches]`, `Fold: [All, Any]`), the expression and the
expectation are text, and the rule a condition belongs to is an identity field. A command that
adds a condition creates the row through `sets:`, so the suite witnesses every field as typed rows
the store keeps. Facts about an operator, such as which take a number, are attributes of the enum
once enum attributes exist (beyond10x/ess#450).

## Whether a stored rule holds is the evaluator's

The command that tests a rule answers with an outcome the input cannot decide, because the answer
depends on the rows and on the evaluator, not on the request:

```yaml
- name: not-met
  external: the evaluator finds the rule's stored conditions do not hold
  error: demo.rules.RuleNotMet
- name: met
  emits: [demo.rules.RuleMet]
  payload: {demo.rules.RuleMet: {rule_id: input.rule_id}}
```

`external:` says the branch is decided outside the specification. The synthesized suite forces it:
the not-met verdict is injected through `configure_external_outcome` before the command is sent,
and the scenario requires `not-met` and its error. The `met` scenario sends the command with no
verdict injected. Nothing in the suite claims to know which stored rules hold.

## A fold over stored results is a row-set guard

Where the system stores each condition's result as a field of its row, such as `holds: Boolean`,
the fold is a fact the specification can state. From `ess/22` a `when_related` row-set guard
selects the rule's rows and tests them
([filtered related reads](filtered-related-reads.md)):

```yaml
- name: met                       # ALL: every stored condition of the rule holds
  when_related:
    entity: demo.rules.Condition
    where: rule_id == input.rule_id
    forall: holds == true
```

ALL is `forall` over the rule's rows; ANY is `exists` over the rows that hold, `exists: true` with
`where: {all: [rule_id == input.rule_id, holds == true]}`. The suite arranges rows that make each
branch hold and fail. The evaluation of each condition stays the system's; only the fold over
results it stored is the specification's.

## Patterns held as data

A regular expression a system stores and evaluates as data, as an operator of its stored
conditions or a reserved name pattern, is the same case: the evaluator is the system's, and the
verdict is an `external:` outcome. ESS has no pattern predicate in a guard either
(beyond10x/ess#449, which follows #95 and #103): a text shape the specification itself checks is
written with an `alphabet:` and `prefix:` newtype, `.count`, `starts_with`, `ends_with`,
`contains` and `any_of` (`website/docs/reference/predicates.md`, "Text shapes without patterns").
A pattern predicate would need a portable dialect with a parser of ESS's own and a translation per
runner, and synthesis would need a matching text and a near miss for every pattern. The request is
reopened only when a guard, not stored data, needs a position-dependent character class the idiom
cannot state, and the candidate then is a per-position alphabet on a newtype, not a regular
expression.
