---
format: aep.planning-md/3
id: story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and
kind: story
status: active
title: With --model, ess-ui-resolved/1 carries the route table the stock generators bind the document's views and commands to, --http included
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:ess-ui-load-format-json-prints-the-document-the-loader-resolved-every-default-filled-as-the-versioned-format
- depends_on: story:an-ess-ui-1-document-has-one-canonical-byte-form-and-one-digest-the-same-for-every-key-order-yaml-spelling
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:19:21Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:22Z", actor: "human:timo", revision: 5}
---
## Need

The requester asked for "every model reference resolved to its qualified name and type". An own
generator also has to bind the document's views and commands to the HTTP surface the
specification serves. The stock generators take that route table from `ess_ui_check::binding`
through `bind` (`crates/edge/ess-cli/src/ui.rs:218-252`).

- The table is `ess_ui::binding::Binding` (`crates/ui/ess-ui/src/binding.rs:24-39`). It already
  derives `Serialize` (`story:ui-binding-contract`, implemented) and holds:
  - the served components with their view and command routes;
  - every name as the document writes it, mapped to its qualified name;
  - the enum variants a choice's `options` name.
- It reaches a generated React project only as TypeScript, `src/binding.ts`, written as
  `export const binding: Binding = <json>;` (`crates/ui/ess-ui-react/src/assets.rs:286-296`).
- `bind` takes `--http FILE [--refines FILE]`. That replaces the routes with the ones the HTTP
  binding declares (`ui.rs:229-251`). `ess ui load` has no `--http`, so without it an own
  generator would get other routes than the stock one for an app served with an HTTP binding.
- `ess ui load --model` reads the model's enums only (`ui.rs:37-40`, `:106-107`). It does not
  compute the binding.

## Outcome

- `ess ui load` takes `--http FILE [--refines FILE]`, as `ess generate ui` does: the `Load`
  arguments flatten the existing `Http` (`ui.rs:46-`).
- With `--model <spec>` and `--format json`, the `ess-ui-resolved/1` document also holds
  `binding`: the `Binding` that `bind` returns for the same document, model and `--http`, member
  for member. That is the table `ess generate ui --target react` writes into `src/binding.ts`.
- The binding is computed only under `--format json` (P0-D11 A). `ess ui load --model` in text
  form accepts exactly the documents it accepts today.
- A document whose binding cannot be computed is refused under `--format json` with the message
  `ess generate ui --model` prints for it, for example a document that names a view no
  `reached_by: network` component serves.

## Acceptance

Cases appended to `crates/edge/ess-cli/tests/ui_load_json.rs` (from `g03-resolved-json`). Each test
is expected under 5 s and is timed; none may run over 60 s, and the crate's test step stays within
120 s.

The pair: the document `crates/edge/ess-cli/tests/fixtures/ui-http/items/ui.yaml` with the model
`crates/edge/ess-cli/tests/fixtures/ui-http/items/system.yaml`, the pair that
`generate_ui_http.rs` builds (`crates/edge/ess-cli/tests/generate_ui_http.rs:155-180`).

1. `the_model_adds_the_route_table`. Red today: the JSON has no `binding`.
   `ess ui load --path <document> --model <spec> --format json` prints `binding`. It deserialises
   to a `Binding` equal to the object in `src/binding.ts` that
   `ess generate ui --target react --path <document> --model <spec>` writes. The object is the
   JSON text after `export const binding: Binding = ` up to the final `;`.
2. `http_routes_are_the_generators_routes`. The same pair with `--http`, using the binding file
   `generate_ui_http.rs` writes from `http.yaml.in`, prints a `binding` equal to the one
   `ess generate ui --http` writes for the same files.
3. `text_with_a_model_is_unchanged`. `ess ui load --model` in text form prints the summary line,
   and accepts and refuses exactly the documents of the UI corpus it does today.
4. `an_unbindable_document_is_refused_only_in_json`. A document naming an unserved view loads in
   text form and is refused under `--format json` with the generator's message, exit 1.
5. `the_model_bytes_are_pinned`. The output for the pair without `--http` equals the live golden
   `crates/edge/ess-cli/tests/golden/ui-http-items.resolved.json`. It is written by
   `cargo run -q -p ess-cli -- ui load --path crates/edge/ess-cli/tests/fixtures/ui-http/items/ui.yaml --model crates/edge/ess-cli/tests/fixtures/ui-http/items/system.yaml --format json > crates/edge/ess-cli/tests/golden/ui-http-items.resolved.json`.
   The frozen golden `crates/edge/ess-cli/tests/golden/ui-http-items.resolved.first-release.json`
   is a byte copy, which no later story edits. `an_old_reader_still_reads` of
   `g03-resolved-json` is extended to read it, so `binding` has an old-reader check too.
6. `ess_ui_load_prints_the_resolved_document` is amended: with `--model`, the members are the
   formats row's list including `binding`.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-cli --all-targets --locked -- -D warnings`;
- `cargo test -p ess-cli --test ui_load_json --test generate_ui_http --locked`;
- `task ci-lint`;
- `task projection-check` (`cargo xtask cli-reference --check` compares `cli.md`).

Also:
- The `ess ui load` entry of `website/docs/reference/cli.md` lists `--http` and `--refines`.
  `cli.md` is regenerated by `cargo xtask cli-reference`, never edited by hand. `g01-state-cli`
  regenerates it again in round 8.
- `CHANGELOG.md` has an entry under Added.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3. It is the second half of the earlier draft (never filed in the store), split off for size
only. It is part of P0, and it ships in the same release as `g03-resolved-json`.

1. **Need.** An own generator binds to the same routes as the stock one. In the evidence received
   on 2026-11-24, a downstream application's own generator spends 195 of its 1,807 lines on model
   binding. This unit removes the names and routes among them. Types stay with the generator.
2. **Class.** It is a *gap*.
3. **Already expressible?** Only by running the React generator and parsing `binding.ts`.
4. **Fit.** It reuses `Binding` as it already serialises, and the `Http` arguments and `bind` as
   `ess generate ui` uses them. No new shape.
5. **Second adopter.** A CLI client generator that calls the same routes.
6. **Cost.**
   - It couples `Binding`'s struct to a published format: a member rename needs
     `ess-ui-resolved/2`.
   - Two flags on `ess ui load`. No diagnostic.
7. **Considered.**
   - (a) Only the enums that `ess ui load --model` reads today. Too little to bind to.
   - (b) Every view's row field types as well. That duplicates the compiled IR (P0-D11 C).
   - (c) The table without `--http`, saying so in the formats row. Rejected: an app served with
     an HTTP binding would get other routes than the stock generator.
   - (d) Chosen: `Binding` from the same `bind`, with `--http`.

## Decisions

- **P0-D11 (the draft's D5): What does `--model` add?**
  - (A) The existing `Binding`, computed only under `--format json`, with `--http` honoured.
  - (B) The enums only.
  - (C) `Binding` plus every view's row field types.

  **Recommended: (A).** Computing the binding under `--format text` too would let
  `ess ui load --model` refuse documents it accepts today.

## Scope

- `crates/edge/ess-cli/src/ui.rs`: `Load` flattens `Http`; `run` 105-115 calls `bind` (218-252)
  under `--format json`.
- read only: `crates/ui/ess-ui/src/binding.rs`.
- tests appended to `crates/edge/ess-cli/tests/ui_load_json.rs`.
- new: `crates/edge/ess-cli/tests/golden/ui-http-items.resolved.json` and
  `crates/edge/ess-cli/tests/golden/ui-http-items.resolved.first-release.json`.
- `website/docs/reference/cli.md` (generated), `CHANGELOG.md`. Not changed: `formats.md`, whose row
  `g03-resolved-json` writes with `binding` in it.
- Size: S, about 220 changed lines.

## Depends on

- `g03-resolved-json`: the output function, the test file and the formats row.
- `g02-document-digest`, through `g03-resolved-json`.

Round 7, beside `g01-state-document`, with which it shares no file. `g01-state-cli` edits `ui.rs`
and `cli.md` after it, in round 8.
