---
format: aep.planning-md/3
id: story:an-external-creators-further-runs-name-arranged-rows
kind: story
status: draft
title: An external creating branch beside a related guard sends only arranged rows
summary: 0.69.1 synthesized RenderPatch/outcome/rendered-limited for a downstream specification with a further run naming a patch no step created; the interpreter failed it. Further runs of such a branch are now its siblings'.
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- informed_by: story:rows-a-guarded-move-reaches-are-made-once
revision: 1
---
## Need

Under 0.69.1 the synthesized suite of the downstream specification of beyond10x/patchback (10 files, 130 authored scenarios) holds 470 scenarios, and the model interpreter fails one: `patchback.audio.RenderPatch/outcome/rendered-limited` (469 passed, 1 failed; patchback wave coordinator, measured 2026-10-25). 0.68.0 refused that route at synthesis, so patchback cannot move its pin to 0.69.1. A synthesized scenario must pass the model interpreter of the specification it was synthesized from.

## Reproduction

- The failing scenario sends `RenderPatch` twice. The first send names the arranged patch and note sequence (`patch-138601`, `note-sequence-360361`); the second, with `block_size_frames` left out, sends literal identities no step created (`00000000-0000-4000-8000-1ffbb7db0250`), so `no-such-patch` (`when_related: {via: input.patch_id, exists: false}`, declared before the external branches) answers instead of `rendered-limited`.
- Instrumented release build of 0.69.1, one line per `run_as` call: `rendered-limited` is run as `Witness::Full` with `external_row=true` and 39 arranged steps, then as `Witness::LiteralFallbacks(0)` with `external_row=false`, 0 arranged steps and no bound instance. It is the only such call in the specification. The plain sibling `rendered` is routed by the related guard and gets no further run at all; `non-finite-output` copies no `else:` fallback and gets none either.
- Cause: `run_as` arranges the related row for an external branch beside `when_related:` guards (beyond10x/ess#464) only for the full witness; every further witness of that branch was arranged as for a command reading no row.
- A four-scenario model (an `exists: false` refusal, an external creating branch and a plain creating branch, both copying an `Optional` through `{input: …, else: 256}`) fails the same scenario on every cached release from 0.50.0 (ess/18) to 0.69.1. No commit in 0.68.0..0.69.1 introduced it; patchback reached it in 0.69.x because the full invocation became reachable there.

## Fix

A further witness of an external branch that reads a related row through its input is refused in `run_as`, as those of a branch the related guard routes already were, so the scenario keeps its full invocation. An external refusal answered before the row (`before_row: true`, ess/25) reads no row and keeps its further runs.

## Acceptance

- `tests/external_creator_further_runs.rs` on fixture `external-creator-beside-related-guard.yaml`: every `RenderPatch` send of `rendered-limited` names an arranged instance, and the four synthesized scenarios pass the model interpreter. Both tests fail without the fix.
- `synthesis_suite_bytes_table`: the 347 pinned models keep their bytes; the table gains the new fixture only.
- The patchback specification synthesizes 470 scenarios and 27 refusals, and all 470 pass the model interpreter.
