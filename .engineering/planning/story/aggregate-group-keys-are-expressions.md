---
format: aep.planning-md/3
id: story:aggregate-group-keys-are-expressions
kind: story
status: active
title: A group key is an expression, with one group per List element
tags:
- adopter-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#561
- provider: github
  reference: beyond10x/ess#565
relations:
- serves: vision:O2
- decomposes: epic:generic-aggregates
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-expression-measure-that-skips-absent-rows-is-witnessed
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/views.rs
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:08:02Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:08:02Z", actor: "human:timo", revision: 3}
---
## Need

An aggregate view groups by a value computed from the source row, and a row whose `List` field
names several elements counts once in the group of each element: downtime per affected service,
where a window names the services it affects as a list. Today a group key is a source field of
value equality, a `List` key is refused, and no spelling computes a key.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/c-list-key`, `repro/c-each-key` of the
triage of beyond10x/ess#561; `Window` carries `services: List<String>` and `duration_s: Integer`):

```yaml
  - name: demo.ledger.DowntimeByService
    source: demo.ledger.Window
    group_by: [services]
    fields:
      - {name: services, type: List<String>}
      - {name: down_s, type: Integer, aggregate: {sum: duration_s}}
```

```text
$ ess specify validate --path c-list-key
error[ESS-VIEW-002]: `demo.ledger.DowntimeByService` groups by `services` of type List<String>, which has no value equality a group could be keyed on
exit 1
$ ess specify validate --path c-each-key        # group_by: [{expr: "each(services)"}]
system.yaml: invalid type: map, expected a string
exit 1
```

## Acceptance

At the next source format (`ess/37`), with `Window` rows A (`services: [api, web]`,
`duration_s: 60`) and B (`services: [web]`, `duration_s: 30`):

- this view validates (`ess specify validate`, exit 0):

  ```yaml
  group_by: [service]
  fields:
    - {name: service, type: String, expr: "each(services)"}
    - {name: down_s, type: Integer, aggregate: {sum: duration_s}}
  ```

- `ess verify conform synthesize` writes `DowntimeByService/aggregate` arranging a row with two
  elements, and `ess verify conform run --target interpreted` passes it; the interpreter answers
  the rows `{service: api, down_s: 60}` and `{service: web, down_s: 90}`;
- a key computed without `each`, `{name: band, type: Integer, expr: "level.weight"}` in
  `group_by: [band]`, validates and groups by the computed value;
- `group_by: [services]` over the bare list keeps ESS-VIEW-002; a Timestamp key stays refused;
  `each(x)` outside a group key field, or over a field that is no `List`, is refused as
  `type_mismatch`; `each` twice in one view is refused as `unsupported_construct`;
- below `ess/37` a computed group key is refused as `unsupported_format_version` naming `ess/37`,
  and a model without one keeps its IR bytes;
- generated Rust and Go view queries keep such a view an obligation naming the key expression;
  Entity Runtime refuses it by name; `ess verify diff` reports a changed key expression as
  `GroupingChanged`.

## Scope

- cited: crates/specify/ess-domain/src/view.rs (`validate_grouping`, the List-key refusal around line 2210, the computed-field refusal around line 1838)
- inferred: crates/specify/ess-domain/src/expression.rs (`each`, admitted only as a group key)
- inferred: crates/specify/ess-compiler/src/ir.rs (`ResolvedAggregation`, a key's expression)
- inferred: crates/specify/ess-compiler/src/resolve.rs
- inferred: crates/verify/ess-conformance/src/aggregate.rs (`evaluate`, per-element fan-out)
- inferred: crates/verify/ess-conformance/src/interpret/views.rs
- inferred: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- inferred: crates/generate/ess-synth/src/view_query.rs (obligation naming the key)
- inferred: crates/verify/ess-diff/src/change.rs (`GroupingChanged`)
- inferred: crates/specify/ess-domain/src/system.rs (`FORMAT_HISTORY` row for `ess/37`)
- inferred: schemas/generated/ess.schema.json
- cited: docs/design/generic-language-features.md (§ G3, a subsection "Expression group keys" decided before code)
- inferred: docs/design/language-grids.md (grid R, a group key position)
- inferred: website/docs/reference/spec-versions.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A figure per element of a list a row carries (downtime per affected service), and a
   figure per computed value of the row. Reproduction above (run.txt § C). Requester's syntax
   (theirs, beyond10x/ess#565 under the #561 feature): `group_by: [service_ids]`.
2. **Class.** Gap: no spelling groups per element or by a computed value (run.txt § C; ESS-VIEW-002
   at `crates/specify/ess-domain/src/view.rs:2210`).
3. **Already expressible?** Only with a second entity, one row per (window, service), which the
   domain does not have and every writer must keep in step; the figure the reader sees would then
   rest on a modelling artefact. No idiom.
4. **Fit.** G3 decides "a group key is an expression, including `each(list_field)`"
   (`docs/design/generic-language-features.md:375-377`) and `{expr: …}` as the spelling (answer 3,
   `:714`). The design adds no key: a view field's `expr:` exists (G1; fixture
   `crates/verify/ess-conformance/tests/fixtures/expression-arithmetic-and-time.yaml:63`) and
   `group_by:` already names view fields (`view.rs:1975`); a computed field named in `group_by:`
   is a key, and reads the source row. Grid R: opens the "group key" position for every source
   root the measure input reads (fields, enum attributes), not one cell. Inferring per-element
   grouping from a list type is refused: it would change what `group_by: [list]` means. Targets:
   interpreter and synthesis evaluate; Rust and Go keep an obligation, named, as `ess/36` did for
   expression measures (`docs/design/generic-language-features.md:437-440`); Entity Runtime
   refuses by name; diff has `GroupingChanged`.
5. **Second adopter.** Articles counted per tag; tickets per affected component; orders by
   weight band (`level.weight`).
6. **Cost.** The next source format (`ess/37`), counted under G3 (policy 2, answer 2). No new key;
   one function (`each`) admitted only in a group key field. No suite format: the scenario's
   vocabulary is unchanged. No new diagnostic code.
7. **Alternatives.** (a) Change nothing: the adopter models a join entity. (b) Infer per-element
   grouping from a `List` key: hides a different count; refused. (c) A new `group_by: [{expr}]`
   entry shape: a second way to name a key; refused. (d) Chosen: a computed view field named in
   `group_by:`, with `each` for per-element keys. The requester's shape is changed: the list is
   not the key, `each(list)` is.

## Decisions

**Accept, redesigned.** A group key is a view field named in `group_by:`; it may compute its value
with `expr:` over the source row, and `each(<List field>)` yields one group per element (a row
counts once in each group its list names). The bare list key stays refused. From the next source
format (`ess/37`), counted under G3; `docs/design/generic-language-features.md` § G3 gains a
decided subsection for it before code.

## Source

github:beyond10x/ess#561 (finding beyond10x/ess#565)
