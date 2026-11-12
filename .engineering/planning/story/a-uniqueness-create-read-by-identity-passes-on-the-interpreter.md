---
format: aep.planning-md/3
id: story:a-uniqueness-create-read-by-identity-passes-on-the-interpreter
kind: story
status: implemented
title: A create under a uniqueness check, read by identity, passes on the interpreter
summary: With a when_related row-set uniqueness check, a view by identity and no examples, the synthesized created scenario expects unsuffixed values the arranged inputs suffix (0.88.0 adversary pass, pre-existing)
tags:
- backlog
- defect
- from-finding
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/identity.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/suffixed_instance_inputs.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:42:19Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:42:20Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-12T06:24:42Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Need

Found by the 0.88.0 adversary pass on the suffixed-instance unit, pre-existing and outside its
acceptance, and still present on 0.92.0. A create is guarded by a `when_related` row-set uniqueness
check, has a view read by identity, and has no input examples. It synthesizes a `created` scenario
that fails on the interpreter. The view is expected to hold `name = "name", widget_id = "widget_id"`
and holds `name = "name-1", widget_id = "widget_id-1"`.

Minimal reproduction: the `UNIQUE` model of
`crates/verify/ess-conformance/tests/suffixed_instance_inputs.rs` with its two `example:` keys
removed (`without_examples`). `CreateWidget(widget_id, name, config)` has these outcomes:
`exists` (`existing_instance: true`), then
`name-taken` (`when_related: {entity: Widget, where: name == input.name, exists: true}`), then
`created`. The view is `WidgetById` (`params: [widget_id]`, `filter: widget_id == param.widget_id`).

```text
$ ess verify conform synthesize --path model.yaml --out suite.json
3 scenario(s) (0 authored), 0 refusal(s), written to suite.json
$ ess verify conform run --target interpreted --path model.yaml --suite suite.json --report-format 2
  failed demo.widget.CreateWidget/outcome/created
  ESS-CF-VIEW expected: demo.widget.WidgetById holds a row where name = "name", widget_id = "widget_id"
              observed: {name = "name-1", widget_id = "widget_id-1"}
```

The cause is not a disagreement over the suffix, as first written. The scenario creates a further
widget (`widget-2`, `name-1`) before the subject (`widget`, `name`), then a third after it. It reads
`WidgetById` with `params: {widget_id: {kind: observed, event: WidgetCreated, field: widget_id}}`.
The runners resolve `observed` to the event's first occurrence
(`crates/verify/ess-conformance/src/runner.rs:3426-3432`), which is the further widget's. The
scenario already holds `capture_instance widget` for the subject, but the view parameter is not
sent as that instance (`crates/verify/ess-conformance/src/synthesize.rs:14454`, `bound`, with the
identity from the `observed` fallback near `:14337`).

## Acceptance

- `a_uniqueness_create_read_by_identity_passes_on_the_interpreter`, in
  `crates/verify/ess-conformance/tests/suffixed_instance_inputs.rs` on `without_examples(UNIQUE)`:
  the synthesized suite has no refusal and passes the interpreted target.
- In that scenario, the `WidgetById` read sends `widget_id` as `{kind: instance, instance: widget}`.
  No `{kind: observed}` reference to `WidgetCreated` follows a step that published `WidgetCreated`
  for another instance.
- The suites `suffixed_instance_inputs` pins for models that already pass keep their bytes.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/synthesize/identity.rs
- cited: crates/verify/ess-conformance/tests/suffixed_instance_inputs.rs
- inferred: CHANGELOG.md

confidence: medium
