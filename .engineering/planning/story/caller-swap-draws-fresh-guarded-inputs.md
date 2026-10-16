---
format: aep.planning-md/3
id: story:caller-swap-draws-fresh-guarded-inputs
kind: story
status: active
title: The caller-swapped run draws fresh values for every input a guard reads
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T07:01:18Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-16T07:01:19Z", actor: "human:timo", revision: 3}
---
## Outcome

The caller-swapped half of a synthesized scenario draws fresh values for every input that a guard,
a row-set selector or an identity of the command reads and that does not mention the caller, so a
correct model passes its own suite on `--target interpreted` whatever its keys are scoped to. One
rule covers the shapes fixed one at a time before (#275 scalar identity, #430 struct identity,
#465 a forced outcome's arranged identity) and the new one (https://github.com/beyond10x/ess/issues/580,
a `String` field unique across callers).

## Acceptance

- `a_key_unique_across_callers_survives_the_caller_swap`: the #580 model (one entity with `name`
  unique across callers, `TakeName` refused when any row holds the name, two actors) synthesizes a
  suite whose every scenario passes on `--target interpreted`; the swapped half sends names the
  first half did not write. Fails before the change.
- A planted target that records one fixed caller still fails the swapped half (the swap keeps its
  purpose).
- The suites of models whose inputs never collide keep their bytes, or the bytes table records each
  moved line with its reason.
- Tests are modules of existing test binaries.
