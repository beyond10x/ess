---
format: aep.planning-md/3
id: story:a-type-name-written-in-a-widget-or-a-state-resolves-in-the-model-as-a-page-parameter-s-does
kind: story
status: draft
title: A type name written in a widget parameter, a page state or a document type resolves in the model, as a page parameter's does
summary: type_in_model reads only page parameters; a widget parameter typed with a name that exists nowhere passes ess ui check --model
tags:
- adopter-report
- ess-ui
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- informed_by: story:with-model-ess-ui-check-refuses-a-fixture-row-or-a-script-payload-whose-value-contradicts-the-type-the-model
revision: 1
---
## Need

With `--model`, `ess ui check` resolves the type name of a page parameter in the document's `types`, the
schema's constructs, the primitives and the model's types, entities and views (`type_in_model`,
`crates/ui/ess-ui-check/src/names.rs:27-46`). It reads that one place only, `document.pages[*].params`.
Every other type expression a document writes passes with any name. So a widget parameter typed
`{optional: NoSuchId}` is accepted, and so is every name below.

Reported by a downstream specification of reusable building blocks (ess 0.99.2, finding G5 of one
collected report: widget parameter types are not checked against the model). It reproduces on 0.99.9
with a neutral model and documents:
`.engineering/repro/type-names-resolve-in-the-model/` (`run.sh`, `transcript.txt`).

| where the name stands | written | `ess ui check --model` |
|---|---|---|
| a page parameter | `id: NoSuchId`, `{optional: NoSuchId}` | `type_in_model` error, exit 1 |
| a page parameter | `ItemId` (an id of the model) | accepted, as it should be |
| a widget parameter | `{optional: NoSuchId}`, `NoSuchType` | 0 findings |
| a page's `state` | `{type: NoSuchId}`, `{type: {list: NoSuchId}}` | 0 findings |
| a document `types` member | `Wrapped: {record: {inner: NoSuchType}}` | 0 findings |

A second gap, found while reproducing: the body of a widget no page uses is never checked, because a
widget is "expanded at its use site and then checked like a built-in"
(`website/docs/reference/ess-ui.md` § Widget). A widget whose body reads a view the model lacks, with
a column it lacks, gives 0 findings until a page uses it; once used, the column is refused at the use
site (`transcript.txt`, last command). A document that ships a library of widgets for others to use
has none of them checked by its own check.

## Outcome

Every type name a document writes resolves in the document, the schema, the primitives or the model
when `--model` is given, under `type_in_model`, wherever the type expression stands: a page
parameter, a widget parameter, a page's state, a document `types` member, a `list`,
`map`, `optional` or `one_of` inside any of them.

## Acceptance

Per finding. Cases in `crates/ui/ess-ui-check/tests/` (an existing target, no new test binary), with
the model of the repro directory; each timed and expected under 5 s.

- **G5.** `widget-params.ui.yaml` of the repro: 0 errors today; after, exactly two `type_in_model`
  errors, at `widgets/item_card/params/previous` (`NoSuchId`) and `widgets/item_card/params/kind`
  (`NoSuchType`), exit 1. The same widget with `{optional: ItemId}` and a type of the document's
  `types` passes. The finding is reported once at the declaration, not at each use.
- **Same cause, other places.** `state-types.ui.yaml`: an error at each of
  the page state `picked`, the page state `many` and the `types` member `Wrapped`, each naming the unresolved name; with the names
  corrected, 0 findings. `page-params.ui.yaml` keeps its two errors, unchanged.
- **Unused widget (decision WT-D2).** `unused-widget.ui.yaml`: the view `shop.channel.Nothing` and the
  column `nosuch` of the widget no page uses are findings at the declaration; the used widget's
  column is still one finding, not two.
- Without `--model` nothing changes: the check runs only with a model, and every document of the test
  corpus checked without one prints what it printed before.
- The repository's documents checked with `--model` in tests and `examples/partner-portal` report no
  new error, or each new finding is fixed in the fixture in the same change and named in it.
- `website/docs/reference/ess-ui.md` (the `type_in_model` row now says where it applies) and
  `CHANGELOG.md` name it under Changed: "`ess ui check --model` exits 1 for a document with a type
  name that resolves nowhere in a widget parameter, a state or a document type, which it passed
  before".

## Fit review

Written with `.agents/skills/assessing-external-requests/SKILL.md`. The consumer is a downstream
specification of reusable building blocks.

1. **Need, apart from the syntax.** A name that stands for a type of the model is held to the model
   wherever it is written, so a typo or a renamed id is found by the check and not at run time.
   Reproduction: the table above, a neutral `shop.channel` model. The reporter proposed no syntax.
2. **Class.** Gap in a check, a false negative: the reference says a named type resolves in the
   document, the schema or the model (`ess-ui.md` § Type, `type_in_model`) and names the subject
   `pages.*.params.*` only; the widget parameter is outside it, not contradicted by it. Not a
   defect of the document.
3. **Already expressible?** No: the author has no way to ask for the check. Only by using the widget
   from a page, and then the parameter's own type is still unchecked.
4. **Fit.** One check id, one resolution function (`unresolved_types`, `names.rs:58`) applied at
   every place a type expression stands, instead of the page parameters alone. Grid R: the same name
   is read the same way wherever it is written. Siblings: `field_in_model` and `view_in_model`
   already resolve a name in a widget at its use site, with the arguments known; a type name has no
   argument, so it resolves at the declaration. Targets: only `ess ui check` reads it; the renderers
   and the generators do not.
5. **Second adopter.** Any document whose widgets take an id of the model (a row-card widget taking an
   order id), written once and used on several pages.
6. **Cost.** No new key, no format change, no new finding id. A document with an unresolved type name
   in those places exits 1 where it exited 0: named in `CHANGELOG.md`.
7. **Considered.** (a) Change nothing: the author finds the typo when the widget misbehaves. (b)
   Check a widget parameter only (the report's case): rejected, it leaves the state types and the
   `types` members with the same hole. (c) Chosen: every type expression, one function.

## Decisions

All open; each names its recommended option.

- **WT-D1, where a type name is read.** A: every type expression of the document (page and widget
  parameters, page state, `types` members). B: widget parameters only. C: change nothing.
  Recommended: A. The cause is one function reading one place; B repeats the hole three times. Fails
  closed: a name that resolves nowhere is an error.
- **WT-D2, the body of a widget that no page uses.** A: only the declaration's own type names are
  read (this story's findings). B: also every name of the body that reads no `args.<param>` (a literal
  view, a column of a literal view, a command), reported at the declaration once, while a name that
  reads an argument stays checked at each use. C: change nothing. Recommended: B. A widget no page
  of the document uses is a block the document ships, and its body is held to the model today only
  when something happens to use it. B reports at the declaration, so a use does not repeat it.

## Scope

- `crates/ui/ess-ui-check/src/names.rs` (`run`, `unresolved_types`), `crates/ui/ess-ui-check/src/walk.rs`
  (`in_declaration`)
- `crates/ui/ess-ui-check/tests/` (an existing target)
- `website/docs/reference/ess-ui.md` (the `type_in_model` row), `schemas/ui/ess-ui.schema.yaml`
  (`checks.list`), `CHANGELOG.md`
