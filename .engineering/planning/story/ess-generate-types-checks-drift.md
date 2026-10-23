---
format: aep.planning-md/3
id: story:ess-generate-types-checks-drift
kind: story
status: draft
title: Ess generate types checks drift
summary: 'Adopter report S6: ess generate types has no --check (ess#654).'
refs:
- provider: github
  reference: beyond10x/ess#654
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

`ess generate types` has no `--check`, unlike the other generators. Adopters generate into a temporary directory and `diff -r -x .ess-output`, because `.ess-output/state.json` carries a fresh `anchor_id` per directory. Adopter report S6, ess 0.68.0.

## Acceptance

- `check_passes_on_a_fresh_generation`: `generate types ... --out <dir> --check` right after generating into `<dir>` exits 0 and writes nothing.
- `check_names_a_drifted_file`: after editing one generated file, `--check` exits non-zero and names that file.
- `check_names_a_missing_and_an_extra_file`: a deleted generated file and an unexpected file are both named.
- `check_ignores_the_output_state_anchor`: the state file's anchor is not reported as drift.
