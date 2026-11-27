# A2: the A1 row set rewritten with an input equality; the refusal now spreads to every command that needs the created row

ess 0.99.8 (`out/version.txt`). Model: `catalog.yaml`, identical to `../A1-rowset-no-input-equality`
except the `kind-withdrawn` outcome of `Install`, which is now a row set over `Variant` selected
by an input equality, with the Kind read through the quantifier:

```yaml
      - name: kind-withdrawn
        when_related:
          entity: repro.catalog.Variant
          where:
            all:
              - variant_id == input.variant_id
              - exists:
                  in: {entity: repro.catalog.Kind}
                  as: kind
                  that: {all: [kind.kind_id == kind_id, kind.state == Withdrawn]}
          exists: true
```

`Kind.kind_id` is the identity `RegisterKind` mints (`{generated: true}`); `AddVariant` writes
`Variant.kind_id` from its input.

## Command

```console
ess specify validate --path .
ess verify conform synthesize --path . --out out/suite.json
```

## Observed (`out/synthesize.txt`, exit 0)

```text
refused: refusal[ESS-SYNTH-001]: outcome repro.catalog.Install/no-such-variant has no scenario `repro.catalog.Install/outcome/no-such-variant`
  no witness: `repro.catalog.Install/no-such-variant: the rows of `repro.catalog.Kind` related to a candidate are not arranged: no creating command writes `kind_id` from its input` is `row set`, which the related rows of a selector's test are not arranged; only this branch is refused
  help: give the field a type that has a finite value, or drop it from the command's input
refused: refusal[ESS-SYNTH-001]: outcome repro.catalog.Install/kind-withdrawn has no scenario `repro.catalog.Install/outcome/kind-withdrawn`
  no witness: `repro.catalog.Install/kind-withdrawn: the rows of `repro.catalog.Kind` related to a candidate are not arranged: no creating command writes `kind_id` from its input` is `row set`, which the related rows of a selector's test are not arranged; only this branch is refused
  help: give the field a type that has a finite value, or drop it from the command's input
refused: refusal[ESS-SYNTH-001]: outcome repro.catalog.Install/enabled has no scenario `repro.catalog.Install/outcome/enabled`
  no witness: `repro.catalog.Install/enabled: the rows of `repro.catalog.Kind` related to a candidate are not arranged: no creating command writes `kind_id` from its input` is `row set`, which the related rows of a selector's test are not arranged; only this branch is refused
  help: give the field a type that has a finite value, or drop it from the command's input
refused: refusal[ESS-SYNTH-001]: outcome repro.catalog.Configure/configured has no scenario `repro.catalog.Configure/outcome/configured`
  no witness: ``repro.catalog.Install/enabled` `kind-withdrawn` is not decidedly passed over: its row set is Unknown` is `row set`, which sends a row-set command whose required branch the rows the scenario arranges do not decide
  help: give the field a type that has a finite value, or drop it from the command's input
refused: refusal[ESS-SYNTH-001]: entity repro.catalog.Instance has no scenario `repro.catalog.Instance/state/Ready/refuses/repro.catalog.Configure`
  no witness: ``repro.catalog.Install/enabled` `kind-withdrawn` is not decidedly passed over: its row set is Unknown` is `row set`, which sends a row-set command whose required branch the rows the scenario arranges do not decide
  help: give the field a type that has a finite value, or drop it from the command's input
refused: refusal[ESS-SYNTH-001]: transition repro.catalog.Instance/configured has no scenario `repro.catalog.Instance/transition/configured/by/repro.catalog.Configure/configured`
  no witness: ``repro.catalog.Install/enabled` `kind-withdrawn` is not decidedly passed over: its row set is Unknown` is `row set`, which sends a row-set command whose required branch the rows the scenario arranges do not decide
  help: give the field a type that has a finite value, or drop it from the command's input
note: the specification declares actors and serves no component, so enforcing a grant is the caller's, against the generated grant table, and no `<command>/grant/denied` scenario is owed
9 scenario(s) (0 authored), 6 refusal(s), written to out/suite.json
```

## Counts against A1

| | A1 | A2 |
|---|---|---|
| scenarios | 12 | 9 |
| refusals | 3 | 6 |

The three lost (`diff ../A1-rowset-no-input-equality/out/scenario-ids.txt out/scenario-ids.txt`):
`Configure/outcome/configured`, `Instance/state/Ready/refuses/Configure`,
`Instance/transition/configured/by/Configure/configured`. In A1, synthesis still sends `Install`
to arrange an Instance for `Configure`; in A2 it does not, because the arranging `Install` send now
has an undecided row set. This is the mechanism of the drop from 563 to 335 scenarios on the full
specification (there the rewrite was applied to three commands, and every scenario that arranges a
row through one of them goes). Without `Configure`, A1 and A2 both write 8 scenarios and refuse the
same 3 `Install` outcomes.

## Expected

The Kind rows are arranged through `RegisterKind` and `AddVariant` (the Variant's `kind_id` is
written from input and ties it to a minted Kind), so `Install/enabled` gets a scenario, and an
`Install` send that only arranges an Instance for another command is not withdrawn.
