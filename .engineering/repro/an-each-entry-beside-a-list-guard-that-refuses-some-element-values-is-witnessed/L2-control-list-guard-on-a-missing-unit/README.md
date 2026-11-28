# L2: control for L1, a list guard on a missing unit

ess 0.99.9. L1 with the list guard replaced by

```yaml
      - name: tiered-without-unit
        when:
          exists: {in: entries, as: e, that: {all: [e.kind == tiered, not defined(e.unit)]}}
        error: shelf.stock.EntriesRepeated
```

Observed (`out/synthesize.txt`): 7 scenarios and 0 refusals; `staged` has its scenario. Only the guard that excludes every element with a unit unless it is tiered (L1) leaves no four elements.

The suite file `out/suite.json` that the command writes is not kept.
