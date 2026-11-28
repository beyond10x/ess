# A1: a creating outcome behind a row-set guard selected by no input equality gets no scenario

ess 0.99.8 (`out/version.txt`). Model: `catalog.yaml`, three entities (Kind, Variant, Instance).

`Install` is shaped like a real component install command: before its creating outcome `enabled`, the
refusal `kind-withdrawn` is a row set over `Kind` whose `where:` holds no equality with an input;
the input is reached only through an `exists:` quantifier over `Variant`
(`variant.kind_id == kind_id, variant.variant_id == input.variant_id`). `Configure` moves an
`Instance` that `Install` created.

## Command

```console
ess specify validate --path .
ess verify conform synthesize --path . --out out/suite.json
```

## Observed (`out/synthesize.txt`, exit 0)

```text
refused: refusal[ESS-SYNTH-001]: outcome repro.catalog.Install/no-such-variant has no scenario `repro.catalog.Install/outcome/no-such-variant`
  no witness: `repro.catalog.Install/no-such-variant` is `row set`, which selects rows by no equality between a String or Uuid field and the input or the addressed subject, so the rows it counts are not only the scenario's own
  help: give the field a type that has a finite value, or drop it from the command's input
refused: refusal[ESS-SYNTH-001]: outcome repro.catalog.Install/kind-withdrawn has no scenario `repro.catalog.Install/outcome/kind-withdrawn`
  no witness: `repro.catalog.Install/kind-withdrawn` is `row set`, which selects rows by no equality between a String or Uuid field and the input or the addressed subject, so the rows it counts are not only the scenario's own
  help: give the field a type that has a finite value, or drop it from the command's input
refused: refusal[ESS-SYNTH-001]: outcome repro.catalog.Install/enabled has no scenario `repro.catalog.Install/outcome/enabled`
  no witness: `repro.catalog.Install/enabled` is `row set`, which selects rows by no equality between a String or Uuid field and the input or the addressed subject, so the rows it counts are not only the scenario's own
  help: give the field a type that has a finite value, or drop it from the command's input
note: the specification declares actors and serves no component, so enforcing a grant is the caller's, against the generated grant table, and no `<command>/grant/denied` scenario is owed
12 scenario(s) (0 authored), 3 refusal(s), written to out/suite.json
```

`validate`: `repro v1 — 2 file(s), valid`. Scenario ids: `out/scenario-ids.txt`.

Control: the same model with the `kind-withdrawn` outcome removed writes 14 scenarios and 0
refusals. So the row set costs all three `Install` outcomes, including `no-such-variant`, which is a
plain `when_related: {via: input.variant_id, exists: false}` and is still described as `row set`.
The help line (a finite type, or dropping an input field) does not match the cause.

## Expected

`Install/enabled` (and `no-such-variant`) get a scenario: the quantifier ties the Kind rows to
`input.variant_id`, so synthesis can arrange a Variant of an Available Kind and send it.
