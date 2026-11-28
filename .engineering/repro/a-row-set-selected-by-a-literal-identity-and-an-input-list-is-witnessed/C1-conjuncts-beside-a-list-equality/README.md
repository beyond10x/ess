# C1: a list equality beside other conjuncts selects no rows, so every outcome of the guard's command after it is refused

ess 0.99.9 (`out/version.txt`). Model: `stock.yaml`: a `Release` (Draft to Staged) and an `Entry` row set that one command writes with `affects:` and `each:` over its list input `entries`.

`StageRelease` has five outcomes. `tier-unit-differs` is a `when_related` row-set guard over `Entry`. Its `where:` is `all:` of two tests of the stored row and an `exists:` over `input.entries` whose `that:` is `all: [e.kind == tiered, e.name == name, defined(e.unit), e.unit != unit]`. The equality `e.name == name` reads the input list against a `String` field of the stored row.

```console
ess specify validate --path .
ess verify conform synthesize --path . --out out/suite.json
```

Observed (`out/synthesize.txt`): 4 scenarios and 3 refusals, all `ESS-SYNTH-001` "selects rows by no equality between a String or Uuid field and the input or the addressed subject": the guard's outcome `tier-unit-differs`, the success outcome `staged`, and the transition `Release/transition/stage/by/StageRelease/staged`. `validate`: `shelf v1 — 2 file(s), valid`.

Controls, run and not committed (the same model with one change): the guard deleted writes 6 scenarios and 0 refusals; the `affects:` entry deleted, the guard kept, writes 4 and 3 with the same three refusals. So the guard is the cause, not the list input or the each-move.

C2 to C4 change only the guard's `that:` to find which part of the shape is not counted as an equality.

The suite file `out/suite.json` that the command writes is not kept.
