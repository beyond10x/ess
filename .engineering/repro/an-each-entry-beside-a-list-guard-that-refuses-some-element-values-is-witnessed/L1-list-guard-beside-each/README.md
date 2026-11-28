# L1: a list guard that refuses some element values leaves an each: outcome with no four elements

ess 0.99.9 (`out/version.txt`). Model: `stock.yaml`, the model of C1 in the directory `a-row-set-selected-by-a-literal-identity-and-an-input-list-is-witnessed/` with the row-set guard deleted and one list guard added before `no-such-release`:

```yaml
      - name: unit-not-for-tiered
        when:
          exists: {in: entries, as: e, that: {all: [e.kind != tiered, defined(e.unit)]}}
        error: shelf.stock.EntriesRepeated
```

```console
ess specify validate --path .
ess verify conform synthesize --path . --out out/suite.json
```

Observed (`out/synthesize.txt`): 6 scenarios and 1 refusal, `ESS-SYNTH-001` on `staged`: "`staged.affects[0]` is `entries`, which offers no four elements that keep three identities apart, change every member the entry reads on a held row, and reach the branch". The guard's own outcome has a scenario, with a note that its list is sent without spreading.

Controls: L2 swaps the guard for one that refuses a tiered entry with no unit (7 scenarios, 0 refusals). The model with no list guard at all writes 6 and 0. `validate`: `shelf v1 — 2 file(s), valid`.

The suite file `out/suite.json` that the command writes is not kept.
