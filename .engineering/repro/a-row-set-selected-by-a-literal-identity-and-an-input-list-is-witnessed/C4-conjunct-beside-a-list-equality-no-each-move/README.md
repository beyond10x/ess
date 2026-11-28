# C4: one more conjunct beside the equality, and the equality is no longer counted

ess 0.99.9. C3 with the guard's `that:` changed to `{all: [e.name == name, e.kind == tiered]}`. The each-move is deleted, so the guard is the only difference from C3.

```console
ess verify conform synthesize --path . --out out/suite.json
```

Observed (`out/synthesize.txt`): 4 scenarios and 3 refusals, `ESS-SYNTH-001` "selects rows by no equality between a String or Uuid field and the input or the addressed subject" on `tier-unit-differs`, `staged` and the transition: the same three as C1. C3 has 6 and 1.

Run and not committed, each on C3 (the each-move deleted), the second conjunct only changed: `e.unit != unit`, `e.kind == kind` (the entity's own enum against the input) and `e.kind == tiered` written before the equality give the same 4 and 3. A `that:` with the equality as its only conjunct (C3) or as an `all:` of one (C2) is counted; with any second conjunct, in either order, it is not.

The suite file `out/suite.json` that the command writes is not kept.
