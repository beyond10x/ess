---
format: aep.planning-md/3
id: story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer
kind: story
status: active
title: A list field names the text that joins its items, in every renderer
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:39:18Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T00:39:20Z", actor: "human:timo", revision: 4}
---
## Need

A column or record field `as: list` joins its items with `, ` in both renderers, and nothing in
the document can say another join. A path stored as a list of segments, `[docs, guide, intro.md]`,
reads `docs, guide, intro.md`, which is not the path.

- React: `formatValue` maps a list item by item and joins with `", "`
  (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:822-823`). `FieldValue` has no `list`
  case (`fields.tsx.tmpl:200-201`).
- Terminal: `display` joins a sequence with `", "` (`crates/ui/ess-ui-tui/src/expr.rs:255`).
  `cell_value` has no `list` case (`view.rs:1899`).
- The reference lists `list` among the values of `as` and does not say how it joins
  (`website/docs/reference/ess-ui.md:1505`).

The two renderers agree, and the reference is silent. This is not a defect. The coordinator's
classing of G4 as a defect is not confirmed. It is a gap: how a list is spelled as text is a
presentation rule every renderer must keep, and today it is an unwritten one.

Minimal reproduction (fresh, not run here):

```yaml
columns:
  - {field: segments, as: list}    # fixture row: {segments: [docs, guide, intro.md]}
```

Both renderers show `docs, guide, intro.md`.

The requester's proposal, restated: the field names the separator, and a cell can stay on one
line, written `{field: segments, as: list, separator: " / "}`.

## Outcome

`as: list` joins its items with `, ` unless the field declares `separator: <text>`. The reference
states it, and both renderers draw it. Each item is drawn by the rule for its own value
(`sl03-number-as-written`, `sl05-object-one-rule`). "A cell stays on one line" is styling and is
decided in SL-D9, not here.

## Acceptance

- Red first: `ess ui load` accepts `{field: segments, as: list, separator: " / "}`. Today it is refused
  as an unknown field of `Field` (`crates/ui/ess-ui/src/model.rs:2500-2546`,
  `deny_unknown_fields`). The case goes in `crates/ui/ess-ui/tests/value_formats.rs`.
- Over `{segments: [docs, guide, intro.md]}` with `separator: " / "` the cell reads
  `docs / guide / intro.md` in React (`crates/ui/ess-ui-react/tests/value_formats.rs`, all new React
  cases in one test function so that the project compiles once; each test runs in 60 s or less, the
  step in 120 s or less) and in the terminal
  (`crates/ui/ess-ui-tui/tests/value_formats.rs`). Without `separator` both read
  `docs, guide, intro.md`, which is unchanged.
- `separator` on a field whose `as` is not `list`, and on a form field, is refused at the field's
  path, as `unit` off a duration is (`crates/ui/ess-ui/src/path.rs:499-518`).
- `separator` holding a line break is refused, so the joined text stays one line.
- The `ess-ui/1` reference lists `separator` and says `as: list` joins with `, ` by default.
- `CHANGELOG.md` names the key under Added.

## Fit review

Tree `bcfc51555`, read-only. Second report, G4.

1. **Need.** A list whose items form one phrase (a command line, a path's segments, a set of
   flags) reads as that phrase. Reproduction above.
2. **Class.** *Gap*. The renderers agree and the reference is silent, so it is not a defect.
3. **Already expressible?** No. `labels` keys a scalar. `as: tags` draws chips. A model view field
   with `expr:` cannot join a list: its operators are arithmetic and time
   (`website/docs/reference/spec-versions.md`, row `ess/27`). `story:a-ui-expression-joins-texts`
   (draft) asks for text joining in expressions, which is a different place (an `expr` position, not
   a field's value).
4. **Fit.** No grid applies (`language-grids.md:15`, `:63`). One optional key beside the existing
   per-`as` keys `unit` (duration) and `scale` (percent), held by the same "only on such a field"
   check. The requester's spelling is kept.
5. **Second adopter.** A build console shows a command stored as words, `[make, test]`, joined
   by a space as `make test`.
6. **Cost.** One optional key in `ess-ui/1`. Without it a document draws the same text. No new
   version: "a new format version is required when meaning … changes" (`AGENTS.md` § Determinism
   and formats), and the precedent is `unit` gaining `m` and `h` within `ess-ui/1`. An older `ess`
   refuses the key, as it refuses every key added later.
7. **Considered.** (a) Change nothing: the joined text is wrong for a phrase. (b) A join function
   in UI expressions: that is `story:a-ui-expression-joins-texts`, and a column's value is not an
   expression. (c) Chosen: `separator` on `as: list`.

## Decisions

**Accept as proposed** for `separator`. The "one line" half goes to SL-D9 (styling, default
decline).

## Scope

- cited: `crates/ui/ess-ui/src/model.rs` (`Field` 2500-2546), `crates/ui/ess-ui/src/path.rs` (beside `check_unit`)
- cited: `schemas/ui/ess-ui.schema.yaml` (`Field`)
- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (822-823), `fields.tsx.tmpl`
- cited: `crates/ui/ess-ui-tui/src/view.rs` (`cell_value`, which passes the field to the drawing
  function), `crates/ui/ess-ui-tui/src/expr.rs` (the drawing function `sl05-object-one-rule` adds;
  `display` unchanged)
- inferred: `crates/ui/ess-ui-react/src/emit.rs` (passes the key to `FieldSpec`)
- tests: `crates/ui/ess-ui/tests/value_formats.rs`, `crates/ui/ess-ui-react/tests/value_formats.rs`,
  `crates/ui/ess-ui-tui/tests/value_formats.rs`
- inferred: `website/docs/reference/ess-ui.md` (regenerated), `CHANGELOG.md`
- Size: S, about 220 changed lines.

## Depends on

Builds in the third group, after `sl05-object-one-rule`. That story edits the same join in
`core.tsx.tmpl` and adds the drawing function in `expr.rs` that this story's separator goes through.
It goes beside the first report's second chart-sort unit, with which it shares only the schema and
the regenerated reference. It lands before `sl06-time-zone` and `sl10-duration-clock`, which add
keys to the same `Field`.

## Source

A downstream application, 2026-11-24, second report, G4.
