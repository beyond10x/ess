---
format: aep.planning-md/3
id: story:a-view-may-answer-values-no-row-holds
kind: story
status: draft
title: A view may answer values no row holds
summary: 'Adopter reports N20, N23: views need a source and stored fields (ess#674); needs an ADR.'
tags:
- ess-0.71.0
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#674
relations:
- serves: vision:O2
- informed_by: architecture-decision-record:a-command-guard-reads-a-component-setting
- decomposes: epic:adopter-gaps-wire-and-answers
- decomposes: epic:zero-open-issues-2026-11
revision: 5
---
## Need

A view answers only what stored rows hold: it must name a `source`, and each field is a source field,
an `expr:` over the row, or a field of a related row. A value the process holds, such as a limit or a
capability a component is started with, has no row, so a limits endpoint cannot be specified, and a
field holding such a value cannot be added beside stored fields. Components already declare these
values as `settings:` with an optional fixed `value:` (`crates/specify/ess-domain/src/component.rs:160`,
`:178-213`), but nothing reads a setting outside the deployment projection.

Minimal reproduction on 0.92.0 (`triage/674/repro/run.txt`; a `Book` entity, a component declaring
`settings: [{name: max-tags, type: Integer, value: "20"}]`):

```text
$ ess specify validate --path sourceless       # view Limits, no source, field max_tags: Integer
  - system.yaml: missing field `source`
exit 1
$ ess specify validate --path unstored         # field url on a view over Book, no expr
  - error[ESS-VIEW-001]: `example.library.Book` has no field `url`, so `example.library.BookLinks` promises an observation nothing produces
exit 1
$ ess specify validate --path setting-read     # expr: "setting.max-tags"
  - error[ESS-VIEW-001]: `setting.max` is no field of `example.library.Book`, ...
exit 1
```

The third run also shows a spelling gap in the accepted design: setting names are kebab-case
(`max_tags` is refused as `ESS-COMPONENT-002`, run 5), and an expression reads `setting.max-tags` as
`setting.max` minus `tags`.

## Acceptance

- `docs/design/component-setting-reads.md` (new, status accepted) states the `setting.<name>` root,
  its spelling, the first cut, the format and every target's treatment; `docs/design/language-grids.md`
  grid R gains the column `setting` with the view field cell open from the new format and the
  remaining cells named with the record that opens them.
- Under the next source format (`ess/37` unless the open format of the same release takes it), with
  the component above serving the view:
  - `a_view_without_source_validates`: a view with no `source` whose fields are
    `{name: max_tags, type: Integer, expr: setting.max_tags}` and `{name: api_level, type: Integer,
    expr: "1"}` validates and compiles; the IR records the view with no source and one row
    (`cargo test -p ess-domain --locked --test view_settings`, new).
  - `a_setting_is_read_with_hyphens_as_underscores`: `setting.max_tags` reads the setting
    `max-tags` (each `-` written `_`, the rule of `environment_variable`,
    `crates/specify/ess-domain/src/component.rs:584-593`, injective on setting names);
    `setting.max-tags` is refused as `undeclared_reference` with a hint naming `setting.max_tags`.
  - `a_sourced_view_reads_a_setting`: the same field on a view over `Book` validates.
  - Refused by name: a setting without a fixed `value:` (hint naming the first cut);
    `secret: true` (`conflicting_declaration`); a setting no component serving the view declares, or
    declared at different types by two of them; a type not assignable to the field; a sourceless view
    with a field that is neither a literal nor a setting, with `filter:`, `params:`, `shape:` or an
    aggregate.
- At `format: ess/36` the sourceless view keeps `missing field source` and `setting.` keeps its
  refusal, each naming the new format; a model without either keeps its IR bytes at every format.
- Every reader of the view IR compiles against the optional source: a reader reasoning about the
  source's rows skips a sourceless view; the interpreter, synthesis, `ess-gen`, `ess-http`,
  `ess-synth` and Entity Runtime lowering refuse a sourceless view or a `setting.` field by name
  until their own stories land. `ess verify diff` reports a view gaining or losing its source as
  breaking.
- `FORMAT_HISTORY` gains the row; `cargo xtask schema` regenerates the schema.

## Scope

- inferred: docs/design/component-setting-reads.md (new)
- cited: docs/design/language-grids.md (grid R)
- cited: crates/specify/ess-domain/src/view.rs (`ViewSpec.source`, line 1124; `computed`, 1150)
- cited: crates/specify/ess-domain/src/view/from.rs
- cited: crates/specify/ess-domain/src/expression.rs (roots resolved at 917-994)
- cited: crates/specify/ess-domain/src/component.rs (settings, 160-213; `environment_variable`, 584)
- cited: crates/specify/ess-domain/src/system.rs (`FORMAT_HISTORY`)
- cited: crates/specify/ess-compiler/src/resolve.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- cited: crates/verify/ess-diff/src/diff.rs
- cited: crates/verify/ess-conformance/src/interpret/views.rs (refusal by name only)
- cited: crates/specify/ess-http/src/lib.rs (refusal by name only)
- cited: crates/generate/ess-synth/src/rust/behaviour.rs (refusal by name only)
- inferred: crates/specify/ess-domain/tests/view_settings.rs (new)
- inferred: schemas/generated/ess.schema.json (regenerated)
confidence: medium

## Fit review

1. **Need.** A view answers a value no stored row holds: a start-time setting of the component, or
   a literal, with or without a source entity. Reproduction above. Requester's proposals: none
   beyond the two failing shapes (issue body).
2. **Class: gap.** No position reads a setting (ADR `a-command-guard-reads-a-component-setting`,
   Context: the deployment projection is the only reader, `ess-deployment/src/runtime.rs:910`), and
   a view without a source is a parse error (`ViewSpec.source`, `view.rs:1124`).
3. **Already expressible?** Partly: a constant `expr:` on a sourced view validates and the suite
   checks it (run 4, interpreted target passes). It is no answer: it repeats the value per stored
   row, answers nothing when no row exists, and states a literal that can disagree with the
   setting the deployment receives, so the figure the reader sees is not the one checked
   (policy 1, `docs/design/generic-language-features.md:588-594`).
4. **Fit.** Grid R gains a column, `setting`, as `caller` and `now` are columns
   (`docs/design/language-grids.md`); the settings ADR opens its guard cells, Amendment 1 the view
   field cell. This story opens the view field cell only, so it counts as `part` (policy 3), and
   the rest of the column is named on the grid page. The spelling reuses the existing hyphen rule
   of `environment_variable`, adding no syntax. It composes with `expr:` (G1), which already
   computes view fields. Every reader handles the optional source or refuses by name (last
   acceptance item); per-target building is split into three stories.
5. **Second adopter.** A file service publishing `max_upload_bytes` and `allowed_types` for its
   clients; a feature-capabilities endpoint listing which optional features a deployment enables.
6. **Cost.** One source format (shared with the exchange layer when both land in one release,
   Amendment 1, Format), the optional view source in the IR (every reader recompiled), about six
   refusals, a diff kind, a schema regeneration. No migration.
7. **Alternatives** (`decision-blocker:a-view-without-a-source`): A, settings in views, staged
   (chosen); B, literals only, no settings: leaves a deployment's limit unstated; C, decline with
   the constant idiom: fails policy 1 (3 above). An implementation-supplied value the suite cannot
   check stays out (policy 1).

## Decisions

**accept, redesigned.** Option A, staged: Amendment 1 of the accepted ADR
`a-command-guard-reads-a-component-setting` (operator, 2026-10-28,
`approval-record:a-view-reads-a-setting-first-cut`). First cut: only settings with a fixed `value:`;
no suite `configuration:`. Refined here: the root spells a setting's hyphens as underscores (the
`environment_variable` rule), which the ADR's own examples (`setting.open_mode`) already assume;
and the story is split by target (this one: design page, validation, IR, refusals by name).

## Source

github:beyond10x/ess#674
