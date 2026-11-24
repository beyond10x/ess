---
format: aep.planning-md/3
id: story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and
kind: story
status: draft
title: With --model, ess-ui-resolved/1 carries the route table the stock generators bind the document's views and commands to
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format
- depends_on: story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling
revision: 1
---
## Need

The requester asked for "every model reference resolved to its qualified name and type". An own
generator also has to bind the document's views and commands to the HTTP surface the specification
serves. The stock generators take that route table from `ess_ui_check::binding`
(`crates/edge/ess-cli/src/ui.rs:218-252`).

- The table is `ess_ui::binding::Binding` (`crates/ui/ess-ui/src/binding.rs:24-39`). It already
  derives `Serialize` (`story:ui-binding-contract`, implemented) and holds:
  - the served components with their view and command routes;
  - every name as the document writes it, mapped to its qualified name;
  - the enum variants a choice's `options` name.
- It reaches a generated React project only as TypeScript, `src/binding.ts`
  (`crates/ui/ess-ui-react/src/assets.rs:286-296`).
- `ess ui load --model` reads the model's enums only (`ui.rs:37-40`, `:106-107`). It does not
  compute the binding.

## Outcome

- With `--model <spec>` and `--format json`, the `ess-ui-resolved/1` document also holds `binding`:
  the `Binding` that `ess generate ui --target react --model <spec>` writes into `src/binding.ts`,
  member for member.
- The binding is computed only under `--format json` (P0-D11 A). `ess ui load --model` in text
  form accepts exactly the documents it accepts today.
- A document whose binding cannot be computed is refused under `--format json` with the message
  `ess generate ui --model` prints for it. An example is a document that names a view no
  `reached_by: network` component serves.

## Acceptance

Cases appended to `crates/edge/ess-cli/tests/ui_load_json.rs` (from `g03-resolved-json`). Each test
is expected under 5 s and is timed. None may run over 60 s.

1. `the_model_adds_the_route_table`. Red today: the JSON has no `binding`.
   `ess ui load --path <document> --model <spec> --format json` prints `binding`, for the document
   and model pair `generate_ui_http.rs` builds (`crates/edge/ess-cli/tests/generate_ui_http.rs:160-180`).
   `examples/partner-portal` has no model. It deserialises to a `Binding` equal to the one `ess generate ui --model`
   computes for the same inputs, and equal to the object in the generated `src/binding.ts`.
2. `text_with_a_model_is_unchanged`. `ess ui load --model` in text form prints the summary line,
   and accepts and refuses exactly the corpus documents it does today.
3. `an_unbindable_document_is_refused_only_in_json`. A document naming an unserved view loads in
   text form and is refused under `--format json`, with the generator's message, exit 1.
4. A second golden file pins the bytes of that pair printed with `--model`. The old-reader test of
   `g03-resolved-json` covers `binding`.

Also:

- The `ess-ui-resolved/1` row in `formats.md` names `binding`, says it is present only with
  `--model`, and says a change to `Binding`'s members needs `/2`.
- `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3. This is the second half of the earlier draft (never filed in the store). It is split off for
size only. It is part of P0 and ships in the same release as `g03-resolved-json`.

1. **Need.** An own generator binds to the same routes as the stock one. In the evidence received
   on 2026-11-24, a downstream application's own generator spends 195 of its 1,807 lines on model
   binding. The requester's estimate of about 960 removed lines assumes the bindings are resolved.
   The full table is in `g03-resolved-json`.
2. **Class.** This is a *gap*.
3. **Already expressible?** Only by running the React generator and parsing `binding.ts`.
4. **Fit.** It reuses `Binding` as it already serialises. There is no new shape.
5. **Second adopter.** A CLI client generator that calls the same routes.
6. **Cost.** It couples `Binding`'s struct to a published format: a member rename needs
   `ess-ui-resolved/2`. No diagnostic.
7. **Considered.**
   - (b) Only the enums `ess ui load --model` reads today. Too little to bind to.
   - (c) Every view's row field types as well. That duplicates the compiled IR.
   - (a) Chosen: `Binding`.

## Decisions

- **P0-D11 (the draft's D5): What does `--model` add?**
  - (A) The existing `Binding`, computed only under `--format json`.
  - (B) The enums only.
  - (C) `Binding` plus every view's row field types.

  **Recommended: (A).** Computing the binding under `--format text` too would let
  `ess ui load --model` refuse documents it accepts today.

## Scope

- cited: `crates/edge/ess-cli/src/ui.rs` (`run` 105-115; reuse `bind` 218-252)
- cited: `crates/ui/ess-ui/src/binding.rs` (read only)
- tests appended to `crates/edge/ess-cli/tests/ui_load_json.rs`; one more golden file
- `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: S, about 180 changed lines.

## Depends on

- `g03-resolved-json`
- `g02-document-digest`, which edits the same output function in `ui.rs` and lands first
