---
format: aep.planning-md/3
id: story:a-sent-value-may-be-mapped-or-left-out
kind: story
status: active
title: A sent param or input may be left out while a condition does not hold
summary: no value map for what is sent (constant-name spellings), and a false param is always sent
tags:
- backlog
- external-request
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:both-renderers-read-one-binding-expression-grammar
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- confidence: cited
  path: crates/ui/ess-ui-tui/src/http.rs
- confidence: cited
  path: crates/ui/ess-ui/src/model.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:41:45Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:41:46Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A value the UI sends cannot be left out while a condition does not hold. A read param bound to a
boolean state is sent as `false` when the state is false: the React runtime leaves out only
`undefined`, `null` and `""` (`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`), the
terminal only `null` (`crates/ui/ess-ui-tui/src/http.rs`), and an API that reads a present
`only_open=false` as a filter answers differently from one given no param. No grammar says "send
this param only while that toggle is on": on ess 0.92.0 a param written as a mapping is refused,

```yaml
reads: {view: example.order.Orders, params: {q: state.search, only_open: {value: true, when: state.open_only}}}
# error document_loads …/sections/list: `collection`: an expression is a scalar, not Mapping {…}
```

A second report asked to send a picked value through a value table (a row carries `email`, the
command takes `EMAIL`); see Decisions.

## Acceptance

- A read param, an overlay or navigation param and a `bind` entry may be written
  `{value: <expr>, when: <expr>}`: the entry is sent only while `when` is truthy, and left out (not
  sent as `null` or `false`) otherwise. The bare expression form is unchanged.
- Both renderers send the same request for the same state: with `state.open_only` false the read
  carries `q` alone, with it true `q` and `only_open=true`
  (`crates/ui/ess-ui-react/tests/conditional_params.rs`, `crates/ui/ess-ui-tui/tests/conditional_params.rs`).
- `ess ui check` checks both expressions as it checks a bare one, refuses any other key in the
  mapping by path, and with `--model` refuses `when:` on a param the view requires (`read_params`)
  and on a required, non-`Optional` command input.
- A read whose param is left out is a different cache key from one that sends it.

## Scope

- cited: `crates/ui/ess-ui/src/model.rs`
- cited: `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`
- cited: `crates/ui/ess-ui-tui/src/http.rs`
- inferred: `crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/src/emit.rs`
- inferred: `crates/ui/ess-ui-check/src/rules.rs`
- inferred: `crates/ui/ess-ui-check/src/model.rs`
- inferred: `schemas/ui/ess-ui.schema.yaml`
- inferred: `website/docs/reference/ess-ui.md`
- confidence: medium

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 item C19, and the two gaps of the 2026-11-07
report this story was first drafted from.

1. **Need.** A param or input is sent only while a condition holds, so an optional filter is off
   by being absent. Reproduction above (`repro/run.txt`, variant `c19-param-dropped`). The
   requester's words, theirs: "a read param cannot be dropped while a toggle is off"; the earlier
   report's: "left out when false" and "map a sent value through a table".
2. **Class.** Left out: *gap* (no spelling, run above). Value table: *local policy* (one backend
   spells a value differently from the view that offers it).
3. **Already expressible?** Left out: no; there is no conditional expression and a toggle state is
   `true` or `false`, never `null`. Value table: yes, see Decisions.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). One shape
   for every place the UI sends a value: read params, overlay and navigation params, binds — the
   siblings where the same question arises. `when` is the word the document already uses for a
   condition (`Guard.when`, `navigation.restrictions[].when`, `schemas/ui/ess-ui.schema.yaml`).
   It composes with the list form of `story:both-renderers-read-one-binding-expression-grammar`:
   `value` may be a list.
5. **Second adopter.** A log viewer whose "errors only" switch adds `level=error` to the query and
   removes it when off, against an API that reads `level=` as "no level".
6. **Cost.** Additive in `ess-ui/1`: a second form of a param or bind value; no migration; the
   expression loader (`model.rs:37-50`) accepts the mapping only in those positions.
7. **Considered.** (a) Change nothing. (b) `omit_false: true` on a param: one value only, and a
   second key per sibling. (c) Chosen: `{value, when}`, which also leaves out a param while a
   selection is empty. For the table: (a) a `value_maps` construct as `label_maps`: a second
   spelling of a value the specification should spell once; (b) chosen: the idiom.

## Decisions

**Accept, redesigned.** "Left out when false" becomes the conditional entry `{value, when}` on
every sent value, not a flag on read params. The value table is declined as local policy, with the
idiom: the view publishes the value in the spelling the command takes (a field of its row), and the
action binds that field, `bind: {kind: row.kind_code}`; the specification then says both spellings
and the suite checks them, so no figure leaves the specification
(`docs/design/generic-language-features.md` § Policy, policy 1). No new format version;
`docs/design/ui-read-filter.md` gains the conditional entry before code. Depends on
`story:both-renderers-read-one-binding-expression-grammar`, which changes the same expression loader.

## Source

`github:beyond10x/ess#723`
