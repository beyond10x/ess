# C3: C2 without the each-move: the success outcome is witnessed

ess 0.99.9. C2 with the `affects:` entry of `staged` deleted.

```console
ess verify conform synthesize --path . --out out/suite.json
```

Observed (`out/synthesize.txt`): 6 scenarios and 1 refusal. `staged` and its transition have scenarios. Only `tier-unit-differs` is refused: `Entry.name` is a `String` that a filter reads and no declared command writes, because the each-move was the only writer of `Entry`.

C2 against C3 isolates the each-move: it is what refuses `staged` once the equality is counted.

The suite file `out/suite.json` that the command writes is not kept.
