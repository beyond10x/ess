---
format: aep.planning-md/3
id: story:every-reader-of-a-view-fields-wire-name-has-a-test-that-names-it-and-an-executor-reads-the-name-from-the-compiled-ir
kind: story
status: draft
title: Every reader of a view field's wire name has a test that names it, and an executor reads the name from the compiled IR
summary: 'Rust, Go and web answer a view field''s wire: by one function; the semantic row, the web target, a collection answer and the IR member have no test naming it; VW-D1 open'
tags:
- documentation
- external-request
- gap-reports-2026-11-24
- test-gap
relations:
- serves: vision:O2
- informed_by: story:a-view-parameter-carries-its-wire-name-on-every-http-reader
revision: 1
---
## Summary

A view field's `wire:` names the field in every row a read answers. Every reader that ESS ships
does answer it (the generated Rust and Go servers, and the web target by reading), but three
places have no test that names it, and an executor built on the ESS crates has no documented place
to read it: an executor on crates 0.98.0 answered the model's field names for a view that declared
`wire:`. The guide (`website/docs/guides/specify/views.md`, "A view parameter is named on the wire
by its `wire`") says "A field's `wire:` sets its name in each row the read answers, so the rows
carry `jobId`"; nothing says which layer applies it to a semantic row, or where its value is in the
compiled IR.

## Reduction

```yaml
views:
  - name: demo.jobs.JobsAll
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.jobs.JobId, wire: jobId}
      - {name: owner_label, type: String, wire: ownerLabel}
```

An `ess-http/1` binding `GET /jobs` on the view, one stored row. Expected by the guide:
`{"items":[{"jobId":"j1","ownerLabel":"x"}]}`.

## Findings (read at the head of 0.99.2; `file:line` under `crates/`)

- One function picks the name in every generated reader: `ess_gen::schema::wire_field_name`
  (`generate/ess-gen/src/schema.rs:84`, calling `types::wire_name` at `generate/ess-gen/src/types.rs:838`):
  `naming.wire`, else the field's name.
- Generated Rust server: writes each row member under it (`generate/ess-synth/src/rust/wire.rs:520-548`);
  the derived route (`rust/http.rs:1714`) and the bound route (`rust/http/bound.rs:2050`) share the
  encoder. Test: `a_bound_view_reads_its_parameter_and_writes_its_field_under_their_wire_names`
  (`generate/ess-synth/tests/http_binding/go.rs:1726`; both servers started and asked; one row on a
  bound `get` route: `{"item_id":"w1","itemTitle":"Lamp"}`). No test asserts a view field's wire
  name in a collection answer (`tests/http_binding/wrapped_answers.rs`) or on the derived route.
- Generated Go server: the same (`go/http.rs:952-995`, `go/http/bound.rs:2680`, `go/http.rs:2188`);
  the same test, the same limits.
- Web target: the page reads `row[field.wire]` (`web/page.rs:559`), `wire` being
  `wire_field_name(field)` (`web/catalog.rs:91-92`), and the bridge writes rows with the Rust
  encoder (`web/bridge.rs:522`). No test declares `wire:` on a view field.
- Interpreter (`ess verify conform run --target interpreted`, and `ess verify conform serve`): its
  row is semantic, `ViewRow = BTreeMap<String, Node>` keyed by the field's name
  (`verify/ess-conformance/src/target.rs:1126`; `interpret/views.rs:127`, `:446`, `:464`). No wire
  step exists in it. No test or page states that the keying is by model name and that an adapter
  applies `wire:`.
- No interpreted HTTP surface serves an `ess-http/1` binding: `ess-http` says "Nothing here serves a
  request" (`specify/ess-http/src/lib.rs:11`) and `ess specify http` only validates and compiles.
- Compiled IR: `ResolvedView.fields: Vec<ResolvedField>` (`specify/ess-compiler/src/ir.rs:2360`),
  `ResolvedField.naming: Naming` (`ir.rs:311`), `Naming.wire: Option<String>`
  (`specify/ess-domain/src/name.rs:255`), copied from the declared field at
  `specify/ess-compiler/src/resolve.rs:1993-1997`. In the IR document an executor reads:
  `views["<qualified view name>"].fields[<i>].naming.wire` (`naming` is `{}` for a field that
  declares none). No test pins that member:
  `edge/ess-cli/tests/compiled_view_delivery.rs:27-37` asserts `views[name].naming.wire` and the
  `name` and `type_ref` of each field, not `fields[i].naming.wire`.
- An executor that depends on `ess-compiler` and `ess-domain` has no accessor for a view field's wire
  name: `ResolvedField` has no `wire()`; `Variant::wire()` (`specify/ess-domain/src/types.rs:870`)
  and `Naming::wire_or` (which takes a qualified name, `ess-domain/src/name.rs:291`) are the nearest;
  `wire_field_name` lives in `ess-gen`.

## Red tests to write first

Each is expected green on the reading above; a red one is the defect it names.

1. `ess-conformance`: a model with a view field `wire: jobId`; the interpreted target's row holds
   the key `job_id`, not `jobId` (pins the semantic row).
2. `ess-synth` `http_binding`: a bound collection route (`body: {items: array}`) and the derived
   route of a view whose field declares `wire:`, on both servers: the row members carry the wire
   name.
3. `ess-synth` `web`: a view field with `wire:`; the generated catalogue's field `wire` and the
   bridge's row encoder carry it.
4. `ess-cli` `compiled_view_delivery`: `views[name].fields[i].naming.wire` is the declared wire name,
   and `naming` is `{}` where none is declared.

## Acceptance

- The four tests exist, each names the wire name it asserts, and are green; none runs an expensive
  step per test (the test-budget rule of `AGENTS.md`): one shared build for the Rust and Go servers
  where tests 2 and the existing test can share it.
- The guide's views page states, under the wire-name section, which layer applies a view field's
  `wire:` (the generated servers and the web target; a semantic row of the conformance protocol
  stays keyed by the model's name) and where an executor reads it (`views[...].fields[i].naming.wire`).

## Fit review

1. **Class.** Tests and documentation for a feature every shipped reader already implements; not a
   defect of a reader. The reporter's executor is outside this repository.
2. **Decision VW-D1, open (not taken here).** How an executor reads a view field's wire name:
   - (a) From the IR member and the guide sentence above, as the reporter's own route needs it. No
     new API. Recommended.
   - (b) (a) plus a public `ResolvedField::wire(&self) -> &str` in `ess-compiler`, beside the
     `Variant::wire()` precedent, so no executor re-implements the fallback; `wire_field_name` in
     `ess-gen` would call it.
   - (c) The semantic row of `ess-conformance-target/1` carries wire names. Changes a published
     protocol format and every adapter that reads it; not recommended.
3. **Answer owed to the reporter.** The IR location above; the story id of this draft for the three
   missing tests.

## Decisions

- VW-D1 open: options (a), (b), (c) above; recommended (a), or (a) with (b) if the owner wants one
  accessor. Not architecture of a published format unless (c) is chosen.
