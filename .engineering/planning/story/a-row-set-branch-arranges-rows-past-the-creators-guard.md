---
format: aep.planning-md/3
id: story:a-row-set-branch-arranges-rows-past-the-creators-guard
kind: story
status: implemented
title: A row-set branch arranges rows past the creator's own guard
summary: A row-set refusal's accepting branch is refused with ESS-SYNTH-001 when the only creator of the selected rows carries a when_related via guard; reported on 0.67.0 and 0.68.0
refs:
- provider: github
  reference: beyond10x/ess#682
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T12:41:20Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T12:41:21Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:29Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Report (adopter, ESS 0.67.0 and 0.68.0)

- Command `AddItem` creates an `Item` whose `bin_id` it sets from `input.bin_id`. It first refuses with `when_related: {via: input.bin_id, exists: false}` (no such bin).
- Command `CheckBin` refuses with `when_related: {entity: Item, where: "bin_id == input.bin_id", exists: false}` and otherwise answers an accepting branch.
- `ess verify conform synthesize` refuses the accepting branch of `CheckBin`:

```
refused: refusal[ESS-SYNTH-001]: outcome CheckBin/holds-items has no scenario `CheckBin/outcome/holds-items`
  no witness: `CheckBin/holds-items: `empty` is not decidedly passed over: its row set is True` is `row set`, which has no arrangement of rows the declared commands produce on which the rows its selector selects take this branch
```

## What the code does

- Probe on 0.68.0 (verified): with the `via` refusal removed from `AddItem`, synthesis writes every scenario and 0 refusals. The guard on the creator is what loses the arrangement.
- The message comes from `crates/verify/ess-conformance/src/synthesize/row_set.rs` (`consistent`): no arranged `Item` row matches the filter, so the refusal's row set stays true.
- Inferred, not traced: the row-set arrangement does not arrange the related row (`Bin`) that the creator's own `via` guard needs before it creates the `Item`.
- The hint ("write each field the filter reads from an input") is wrong for this case: the field is already written from an input.

## Workaround

Leave the creator unguarded and refuse a missing bin elsewhere, for example with a separate command or an explicit `exists: false` branch the creator does not carry.

## Fix (0.69.0)

1. Arranging rows for a row-set branch passes the creator's own `when_related` guards, arranging the related row first, as the creator's own scenario already does.
2. A conformance test holds a minimal specification of this shape: `CheckBin/outcome/holds-items` is synthesized and executes.
3. Where no creator can be passed, the refusal names the creator's guard instead of the input hint.
