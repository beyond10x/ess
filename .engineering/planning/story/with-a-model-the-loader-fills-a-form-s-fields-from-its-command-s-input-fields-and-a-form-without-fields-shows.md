---
format: aep.planning-md/3
id: story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows
kind: story
status: draft
title: With a model, the loader fills a form's fields from its command's input fields, and a form without fields shows its inputs in both renderers
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format
revision: 1
---
## Need

`form.fields` defaults to `{from: does.input_fields, order: declaration}`
(`schemas/ui/ess-ui.schema.yaml:661`): the input fields of the command the form submits, in the
order the model declares them. Nothing implements this.

- serde gives `[]` (`crates/ui/ess-ui/src/model.rs:1532-1533`).
- React emits no fields (`crates/ui/ess-ui-react/src/emit.rs:1010-1011`;
  `templates/runtime/composites/form.tsx.tmpl:85`, `:165`).
- The terminal draws none (`crates/ui/ess-ui-tui/src/app.rs:5010-5021`).
- A form that leaves out `fields` therefore has no inputs anywhere.
- The loader's only model input is `binding::ModelEnums` (`crates/ui/ess-ui/src/binding.rs:97-122`).
  It answers enum lookups and fixture effects, not a command's inputs. It is implemented by
  `ess_ui_check::Model` (`crates/ui/ess-ui-check/src/model.rs:2052`), by `Binding`
  (`binding.rs:245`), and by the test runner's deferred model (`crates/ui/ess-ui-test/src/lib.rs:171`).
- This default needs the model. Without one, nothing can fill it.

## Outcome

- `ModelEnums` gains `command_inputs(&self, command: &str) -> Option<Vec<String>>`, the command's
  input field names in declaration order. Its default is `None`.
  - `ess_ui_check::Model` answers it from the compiled model.
  - `Binding` answers `None`, unless it already carries the inputs; the implementer says which.
- With a model, the fill step writes `fields` onto every form that leaves it out: one `Field`
  short form per input, in declaration order, which the loader's `Field` expansion then completes.
  Without a model, or for a command the model does not know, `fields` stays absent, and `ess ui check --model`
  reports the unknown command as it does today.
- `ess ui load --model`, `ess ui check --model`, `ess ui test --model` and
  `ess generate ui --model` all load through the same step. A form without `fields` then shows
  its command's inputs in both renderers.
- `ess-ui-resolved/1` printed with `--model` holds the filled `fields`. Printed without a model,
  it does not, and its `model: false` says so (`g02-document-digest`).

## Acceptance

New cases in `crates/ui/ess-ui-check/tests/` (a `model_defaults.rs`), in `crates/ui/ess-ui-tui/tests/`
and in `crates/ui/ess-ui-react/tests/`. Each test is expected under 5 s, or 10 s where it runs the
generated runtime, and is timed. None may run over 60 s.

1. `a_form_without_fields_shows_its_commands_inputs`. Red today.
   - A specification fixture has a command `Approve` with the inputs `order_id`, `note` and
     `priority`, in that order.
   - A document has a form `does: Approve` and no `fields`.
   - Loaded with the model, the tree holds `fields` for `order_id`, `note` and `priority`, in that
     order.
   - `ess ui test --model` finds the three inputs drawn in the terminal, and the generated React
     project renders three inputs.
2. `without_a_model_nothing_is_invented`. The same document loaded without a model has no
   `fields`, and both renderers draw what they draw today.
3. `written_fields_are_kept`. A form that writes `fields: [note]` keeps exactly `[note]`.
4. `ess ui check --model` reports the same findings for every corpus document, apart from forms
   that now show inputs. Those are listed in the report.

Also:

- The `ess-ui/1` reference row of `form.fields` says the default needs a model. `CHANGELOG.md`
  has an entry under Changed.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3, with the owner's order "the loader fills every default".

1. **Need.** An own generator, and both stock renderers, get the inputs the schema promises for a
   form that names only its command.
2. **Class.** This is a *defect*: the documented default is implemented nowhere.
3. **Already expressible?** Writing `fields` out works today. The defect is the absent case.
4. **Fit.**
   - It widens the one model hook the loader already has, rather than adding a second one.
   - `ess-ui` still depends on nothing but `ess-yaml`, because the model comes in through the
     trait.
   - There is no new key.
5. **Second adopter.** A CRUD back office whose forms are all "the command's inputs". It writes no
   field list and stays in step with the model.
6. **Cost.**
   - One trait method.
   - The resolved document depends on the model, so the digest depends on whether a model was
     given (`g02-document-digest` records which).
   - No format version.
7. **Considered.**
   - (a) Fill nothing, and let each renderer read the model (P0-D15 B). That is a second copy per
     renderer.
   - (b) Refuse a form without `fields` when no model is given (P0-D15 C). That refuses documents
     the loader accepts today.
   - (c) Chosen: fill with a model, leave absent without one.

## Decisions

- **P0-D15: How is `form.fields` filled?**
  - (A) From the model's command inputs when a model is given, and absent otherwise.
  - (B) Never by the loader. Each renderer derives it from the model.
  - (C) `ess ui load --format json` requires `--model`.

  **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui/src/binding.rs` (`ModelEnums` 97-122, `Binding` impl 245),
  `crates/ui/ess-ui/src/fill.rs` (the model-filled key), `crates/ui/ess-ui/src/lib.rs` (pass the
  model to the fill)
- cited: `crates/ui/ess-ui-check/src/model.rs` (impl 2052: `command_inputs`),
  `crates/ui/ess-ui-test/src/lib.rs` (171: the deferred model answers `None`)
- new tests as above; a specification fixture under `crates/ui/ess-ui-check/tests/`
- `schemas/ui/ess-ui.schema.yaml` (661 note), `CHANGELOG.md`
- Size: M, about 280 changed lines (code about 90, tests and fixture about 190).

## Depends on

- `g03-loader-fills-defaults`
- `g03-resolved-json` (its `--model` path prints the filled tree)
