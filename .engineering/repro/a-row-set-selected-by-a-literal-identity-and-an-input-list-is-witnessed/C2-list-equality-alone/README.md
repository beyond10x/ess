# C2: the same equality alone in `that:` is counted, and the success outcome is refused for another reason

ess 0.99.9. C1 with the guard replaced by `exists: {in: input.entries, as: e, that: e.name == name}` (the one equality the only predicate of `that:`), the `affects:` entry kept.

```console
ess verify conform synthesize --path . --out out/suite.json
```

Observed (`out/synthesize.txt`): 5 scenarios and 2 refusals. The "selects rows by no equality" refusal is gone. `tier-unit-differs` is refused ("its own row set is False": no arrangement of stored rows takes that branch), and `staged` is refused "a step of the scenario changes rows of the selector's entity in a way synthesis does not follow (a set effect, a binding or a direct setup)": the each-move writes `Entry` rows, which the guard reads. The transition scenario exists.

Run and not committed: `that: {all: [e.name == name]}` (an `all:` of one predicate) gives the same 5 and 2; `all: [kind == tiered, {exists: {…, that: e.name == name}}]` (an `all:` around the quantifier) gives the same 5 and 2.

The suite file `out/suite.json` that the command writes is not kept.
