---
format: aep.planning-md/3
id: story:with-a-model-the-loader-fills-a-form-s-fields-from-its-command-s-input-fields-and-a-form-without-fields-shows
kind: story
status: active
title: With a model, the loader fills a form's fields from its command's input fields, and a form without fields shows its inputs in both renderers
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:19:01Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:03Z", actor: "human:timo", revision: 4}
---
## Need

`form.fields` defaults to `{from: does.input_fields, order: declaration}`
(`schemas/ui/ess-ui.schema.yaml:661`): the input fields of the command the form submits, in the
order the model declares them. Nothing implements this.

- serde gives `[]` (`crates/ui/ess-ui/src/model.rs:1532-1533`).
- React emits no fields (`crates/ui/ess-ui-react/src/emit.rs:1010-1011`;
  `templates/runtime/composites/form.tsx.tmpl:85`, `:165`).
- The terminal draws none (`crates/ui/ess-ui-tui/src/app.rs:5010-5021`).
- So a form that leaves out `fields` has no inputs anywhere.
- The loader's only model input is `binding::ModelEnums` (`crates/ui/ess-ui/src/binding.rs:97-122`).
  It answers enum lookups and fixture effects, not a command's inputs. Three types implement it:
  - `ess_ui_check::Model` (`crates/ui/ess-ui-check/src/model.rs:2052`);
  - `Binding` (`binding.rs:245`);
  - the test runner's deferred model (`crates/ui/ess-ui-test/src/lib.rs:171`).
- This default needs the model. Without one, nothing can fill it.

## Outcome

- `ModelEnums` gains `command_inputs(&self, command: &str) -> Option<Vec<String>>`: the command's
  input field names in declaration order. The default implementation returns `None`.
  - `ess_ui_check::Model` answers from the compiled model.
  - `Binding` answers `None`. The test runner's deferred model forwards the call to the model it
    wraps.
- The model pass (`crates/ui/ess-ui/src/fill/model.rs`, the first pass, `g03-loader-fills-defaults`)
  writes `fields` onto every form that leaves it out, when a model is given. It writes one
  expanded `Field` per input, `{field: <input>, name: <input>}`, in declaration order. The later
  passes then fill the `Field` defaults (`as`, `sortable`, and `label` once
  `g03-defaults-that-move-behaviour` has landed), because they run after the model pass. The pass
  writes the row of its `HELD` list (`form.fields`) where a model is given. The list stays, so the
  constant pass never writes the schema's `{from: …}` form into a form loaded without a model.
- Without a model, or for a command the model does not know, `fields` stays absent. `ess ui check
  --model` reports the unknown command as it does today.
- `ess ui load --model`, `ess ui check --model`, `ess ui test --model` and
  `ess generate ui --model` all load through `load_resolved` with the model, so a form without
  `fields` shows its command's inputs in both renderers.
- **Before and after:**
  - The terminal, with `--model`: before, no inputs; after, one input per command input.
  - React, from `ess generate ui --model`: before, no inputs; after, the same.
  - Without a model, neither renderer moves.

## Acceptance

New test files:
- `crates/ui/ess-ui-check/tests/model_defaults.rs`, with the specification fixture
  `crates/ui/ess-ui-check/tests/fixtures/model-defaults/system.yaml` and the document
  `crates/ui/ess-ui-check/tests/fixtures/model-defaults/ui.yaml`;
- `crates/ui/ess-ui-react/tests/model_defaults.rs`;
- `crates/ui/ess-ui-test/tests/model_defaults.rs`.

Each test is expected under 5 s, or 10 s where it runs the generated runtime, and is timed; none may
run over 60 s, and each crate's test step stays within 120 s. Each file joins its crate's single
test binary of `story:the-test-step-meets-the-test-budget` if that has landed; otherwise it is a
new file, which that story moves.

The fixture: a command `Approve` with the inputs `order_id`, `note` and `priority`, in that order,
and a document whose form has `does: Approve` and no `fields`.

1. `a_form_without_fields_is_filled_from_the_model` (`ess-ui-check`). Red today.
   `load_resolved` with the fixture's model holds `fields` as three entries
   `{field: order_id, name: order_id, as: text, sortable: false}`, then `note`, then `priority`.
   `g03-defaults-that-move-behaviour` (round 4) adds `label` to these entries and updates this
   expectation.
2. `react_emits_the_three_inputs` (`ess-ui-react`). The emitted form source for the section lists
   the fields `order_id`, `note` and `priority`, in that order.
3. `the_terminal_draws_the_three_inputs` (`ess-ui-test`). An `ess-ui-test/1` test opens the page
   and takes one step, `expect: {at: pages.<page>.sections.0, text: [order_id, note, priority]}`.
   Run with the model, it passes. Red today. Once the `Field.label` fill lands (round 4), the
   terminal draws `order id`, and that unit updates this step.
4. `without_a_model_nothing_is_invented`. The same document loaded without a model has no
   `fields`, and test 3 without the model fails exactly as it does today.
5. `written_fields_are_kept`. A form that writes `fields: [note]` keeps exactly one field, `note`.
6. Existing tests: the test sets of `ess-ui-check`, `ess-ui-tui`, `ess-ui-react` and `ess-ui-test`
   pass with no expectation changed.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui -p ess-ui-check -p ess-ui-test --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui -p ess-ui-check -p ess-ui-test -p ess-ui-react -p ess-ui-docs --locked`.
  `ess-ui-docs` compares the committed reference (`crates/ui/ess-ui-docs/tests/reference.rs`).
- `task test-xtask`.

Also:
- The note of `form.fields` (`schema:661`) says the default needs a model.
  `website/docs/reference/ess-ui.md` is regenerated by `cargo run -q -p ess-cli -- ui docs`.
- `CHANGELOG.md`, under Changed: "With `--model`, a form without `fields` shows its command's
  inputs in the terminal and in a generated React project, as `ess-ui/1` documents."

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3, with the owner's order "the loader fills every default".

1. **Need.** An own generator, and both stock renderers, get the inputs the schema promises for a
   form that names only its command.
2. **Class.** It is a *defect*: the documented default is implemented nowhere.
3. **Already expressible?** Writing `fields` out works today. The defect is the absent case.
4. **Fit.**
   - It widens the one model hook the loader already has, rather than adding a second one.
   - `ess-ui` still depends on nothing but `ess-yaml`, because the model comes in through the
     trait.
   - No new key.
5. **Second adopter.** A back office whose forms are all "the command's inputs". It writes no
   field list and stays in step with the model.
6. **Cost.**
   - One trait method.
   - The resolved document depends on the model, so the digest depends on whether a model was
     given. `ess-ui-resolved/1`'s `model` member records which (`g03-resolved-json`).
   - No format version.
7. **Considered.**
   - (a) Fill nothing, and let each renderer read the model (P0-D15 B). That is a second copy per
     renderer.
   - (b) Refuse a form without `fields` when no model is given (P0-D15 C). That refuses documents
     the loader accepts today.
   - (c) Chosen: fill with a model, leave absent without one.

## Decisions

- **P0-D15: How is `form.fields` filled?**
  - (A) From the model's command inputs when a model is given; absent otherwise.
  - (B) Never by the loader. Each renderer derives it from the model.
  - (C) `ess ui load --format json` requires `--model`.

  **Recommended: (A).**

## Scope

- `crates/ui/ess-ui/src/binding.rs`: `ModelEnums` 97-122, and the `Binding` impl at 245.
- `crates/ui/ess-ui/src/fill/model.rs`: the model pass, which writes its `HELD` row; the list stays.
- `crates/ui/ess-ui-check/src/model.rs`: `command_inputs` in the impl at 2052.
- `crates/ui/ess-ui-test/src/lib.rs`: the deferred model at 171 forwards the call.
- `schemas/ui/ess-ui.schema.yaml`: the note at 661.
- `website/docs/reference/ess-ui.md`: generated.
- new: the three test files and the two fixture files named above.
- `CHANGELOG.md`.
- Size: M, about 300 changed lines: code about 90, tests and fixtures about 210.

## Depends on

- `g03-loader-fills-defaults`: `fill/model.rs` and `Loaded`.

Round 2, beside `g03-actor-from-session`, with which it shares no file. It edits the schema and the
generated reference, as do `g03-placement-in-loader` (round 3) and
`g03-defaults-that-move-behaviour` (round 4). `g03-resolved-json` prints its result but does not
gate it, so the old edge from this unit to `g03-resolved-json` is dropped.
