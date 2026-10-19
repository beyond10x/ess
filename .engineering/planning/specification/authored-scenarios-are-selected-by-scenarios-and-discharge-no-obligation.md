---
format: aep.planning-md/3
id: specification:authored-scenarios-are-selected-by-scenarios-and-discharge-no-obligation
kind: specification
status: draft
title: Authored scenarios are read from --scenarios and discharge no synthesized obligation
summary: Decline of downstream findings 3 and 4 with the idiom, run on the integration build
tags:
- adopter-report
relations:
- informed_by: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Statement

Two rules about authored scenarios (`ess-scenario/*`), both already documented:

1. **Selection.** `synthesize` and `run` read authored scenarios only from `--scenarios`. Passing
   the model directory as `--scenarios` selects exactly the files its `ess-inputs.yaml`
   `scenarios:` list names; passing a plain subdirectory reads that directory's own files and
   bypasses the manifest; passing nothing reads none, "even when the model's ess-inputs.yaml lists
   some" (`website/docs/guides/verify/author-scenarios.md:15-20`). `validate` checks the listed
   scenarios so that one `synthesize --scenarios` would refuse is refused there too
   (`crates/edge/ess-cli/src/input_discovery.rs:96-101`, beyond10x/ess#112).
2. **Obligations.** An authored scenario runs and counts in the report beside the synthesized ones,
   but it does not discharge a synthesized obligation: a synthesis refusal stays in the refusal list
   until synthesis can derive the witness. An authored scenario is "what the specification cannot
   oblige, written down by a person" (`examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml:1-8`);
   letting it stand in for a derived obligation would hide the generator gap that the refusal
   reports. The same answer was given for https://github.com/beyond10x/ess/issues/597.

Answers findings 3 and 4 of a downstream specification report (2026-10-19). Decision: **decline,
with this idiom**. Finding 3's motivating refusal is a synthesis defect, filed as
`story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed`; when it lands the refusal goes away
without an authored stand-in.

## The idiom

```console
$ ess verify conform synthesize --path <model-dir> --scenarios <model-dir> --out <suite>
$ ess verify conform run --target <t> --path <model-dir> --scenarios <model-dir> …
```

Keep the authored scenario for the behaviour it pins; track each remaining synthesis refusal against
the issue or story that owns it, and gate on the refusal list, not on the authored count.

## Run

Neutral model `.engineering/repro/inputs-listed-scenarios/` (`ess-inputs.yaml` lists
`scenarios/cancel-discards-pending.yaml`), integration build (0.62.0 plus the 2026-10-13 port),
2026-10-19:

```console
$ ess specify validate --path .
cascade v1 — 1 file(s), 1 scenario(s), valid
$ ess verify conform synthesize --path . --out <file>
6 scenario(s) (0 authored), 1 refusal(s)
$ ess verify conform synthesize --path . --scenarios . --out <file>
7 scenario(s) (1 authored), 1 refusal(s)
$ ess verify conform run --target interpreted --path . --scenarios . --report-format 2 --report-out <file>
  passed cascade.feed/authored/cancel-discards-pending
  7 scenarios: 7 passed, 0 failed, 0 error, 0 unsupported
```

The refusal in both synthesize runs is `ESS-SYNTH-001 cascade.feed.Cancel/outcome/cancelled`, the
defect above; the authored scenario that covers the same cascade passes and leaves it in place.
