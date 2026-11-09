---
format: aep.planning-md/3
id: story:derived-measures-and-interval-union
kind: story
status: active
title: A view derives figures from its measures and parameters
tags:
- adopter-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#561
- provider: github
  reference: beyond10x/ess#562
- provider: github
  reference: beyond10x/ess#563
relations:
- serves: vision:O2
- decomposes: epic:generic-aggregates
- depends_on: story:expression-arithmetic-and-time
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:aggregate-group-keys-are-expressions
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/specify/ess-primitives/src/expr.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: docs/design/aggregate-views.md
- confidence: cited
  path: docs/design/generic-language-features.md
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: docs/design/read-api-view-idioms.md
- confidence: inferred
  path: schemas/generated/ess.schema.json
- confidence: inferred
  path: website/docs/reference/spec-versions.md
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:08:02Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-21T22:08:03Z", actor: "human:timo", revision: 4}
---
## Need

An aggregate view states a figure computed from its own measures and its parameters, and the suite
checks that figure exactly: strict uptime per group as `1 - down / (param.to - param.from)`, a
conversion rate as `won / offered`. Today a computed field on an aggregate view is refused, so the
figure leaves the specification and no suite checks it.

This story is the arithmetic-over-measures half; the interval union clipped to a period is
`story:an-interval-union-measure-is-clipped-to-a-period`, which builds on it.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/c-derived` of the triage of
beyond10x/ess#561):

```yaml
  - name: demo.ledger.Uptime
    source: demo.ledger.Window
    params: [{name: from, type: Timestamp}, {name: to, type: Timestamp}]
    group_by: [incident]
    fields:
      - {name: incident, type: String}
      - {name: down_s, type: Integer, aggregate: {sum: duration_s}}
      - {name: uptime, type: Optional<Decimal>, expr: "1 - down_s / (param.to - param.from)"}
```

```text
$ ess specify validate --path c-derived
error[ESS-VIEW-009]: `demo.ledger.Uptime` is an aggregate view, and `uptime` computes from one row; arithmetic over measures is not admitted yet
error[ESS-VIEW-003]: `demo.ledger.Uptime` declares the parameter `from` and no filter reads it, so every value of it returns the same rows
error[ESS-VIEW-005]: `uptime` is neither an aggregate nor a group key, so a row has no single value for it
exit 1
```

## Acceptance

At the next source format (`ess/37`):

- a view field on an aggregate view that is not named in `group_by:` may declare `expr:` reading
  the same row's measures by field name, its group keys and `param.<name>`; with
  `down: Duration` (`{sum: {expr: "ended_at - started_at"}}`) and
  `{name: uptime, type: Optional<Decimal>, expr: "round(1 - down / (param.to - param.from), 4, half_even)"}`
  the view validates (exit 0), and a parameter read only there is not ESS-VIEW-003;
- `Duration / Duration` is a Decimal of elapsed seconds, and a zero divisor answers absent, so the
  field's type must be `Optional<…>` (else `type_mismatch`);
- such a field reading a source row field is refused as `unsupported_construct` naming the field
  ("a derived figure reads measures, keys and parameters"); one reading another derived field is
  refused as `unsupported_construct`;
- `ess verify conform synthesize` witnesses it on a group whose `down` makes the quotient a
  half-even tie at the declared digits and on a zero-length period, which answers absent; `ess
  verify conform run --target interpreted` passes both and a target computing half-up fails the
  first;
- below `ess/37` the field keeps ESS-VIEW-009, its hint naming `ess/37`; a model without one keeps
  its IR bytes;
- generated Rust and Go view queries keep such a view an obligation naming the expression; Entity
  Runtime refuses it by name; `ess verify diff` reports a moved expression as a field change.

## Scope

- cited: crates/specify/ess-domain/src/view.rs (the computed-field refusal around line 1838, `validate_grouping`, parameter-read check)
- inferred: crates/specify/ess-domain/src/expression.rs (a measure and `param.` root in a derived field; `Duration / Duration`)
- inferred: crates/specify/ess-primitives/src/expr.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/specify/ess-compiler/src/resolve.rs
- inferred: crates/verify/ess-conformance/src/aggregate.rs (evaluated after grouping)
- inferred: crates/verify/ess-conformance/src/interpret/views.rs
- inferred: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- inferred: crates/generate/ess-synth/src/view_query.rs
- inferred: crates/verify/ess-diff/src/change.rs
- inferred: crates/specify/ess-domain/src/system.rs (`FORMAT_HISTORY` row for `ess/37`)
- inferred: schemas/generated/ess.schema.json
- cited: docs/design/generic-language-features.md (§ G3, a decided subsection "Derived measures" before code)
- cited: docs/design/language-grids.md (grid O, `/` over Duration)
- cited: docs/design/aggregate-views.md (decision 3 revised)
- cited: docs/design/read-api-view-idioms.md (the ratio idiom retired)
- inferred: website/docs/reference/spec-versions.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A ratio of a group's measures to each other or to the requested period, checked
   exactly by the suite. Reproduction above (run.txt § D). Requester's syntax (theirs,
   beyond10x/ess#563 under the #561 feature): `aggregate: {ratio: {numerator, denominator:
   {param_span}}}`.
2. **Class.** Gap. beyond10x/ess#563 reopens beyond10x/ess#441 (decision 3,
   `docs/design/aggregate-views.md:23`, out of scope at `:1035`); its idiom (`docs/design/read-api-view-idioms.md:28`, "Derived values over aggregates are the consumer's")
   has the consumer divide, which moves the figure out of the specification.
3. **Already expressible?** The operands are, the figure is not; by policy 1
   (`docs/design/generic-language-features.md` § Policy, 1) the consumer-side idiom is no answer.
4. **Fit.** G3 decides it: "a view field `{expr: "…"}` reading the row's measures and view
   parameters … evaluated after grouping, reads no source row, and is checked exactly by the
   suite" (`generic-language-features.md:378-382`). The spelling reuses view-field `expr:` (G1)
   and the `{expr: …}` decision (answer 3, `:714`); a computed field not in `group_by:` is
   derived, one in `group_by:` is a key (`story:aggregate-group-keys-are-expressions`), so one
   rule decides both. `Duration / Duration` follows grid O's closing target for the Duration
   column (`docs/design/language-grids.md:89-90`). Targets: interpreter and synthesis evaluate;
   Rust and Go keep an obligation, named; Entity Runtime refuses by name; diff.
5. **Second adopter.** Conversion rate per campaign (beyond10x/ess#441); share of on-time
   deliveries per carrier; occupancy per day as booked time over the day.
6. **Cost.** The next source format (`ess/37`), counted under G3. No new key: `expr:` exists. One
   grid O cell column completion (`/` over Duration). Suite: computed Decimal values are already
   carried for `avg`; no suite format unless the story finds a new expectation shape, which it
   names. Diagnostics: reuse ESS-VIEW-009 / `type_mismatch`.
7. **Alternatives.** (a) Change nothing: the figure stays outside conformance. (b) The requester's
   `ratio:` key with a `param_span` operand: a key per arithmetic shape; refused. (c) Chosen:
   arithmetic over measures in a derived view field, one expression language.

## Decisions

**Accept, redesigned.** Arithmetic over measures is a derived view field (`expr:`, not named in
`group_by:`) reading the row's measures, keys and parameters, evaluated after grouping, with `/`
answering absent on a zero divisor and `round(…)` stating digits and mode. Decision 3 of
`docs/design/aggregate-views.md` is reopened by the 2026-10-13 decision to build the four generic
features. The interval union moves to its own story. Retitle: "A view derives figures from its
measures and parameters". From `ess/37`, counted under G3.

## Source

github:beyond10x/ess#561 (finding beyond10x/ess#563)
