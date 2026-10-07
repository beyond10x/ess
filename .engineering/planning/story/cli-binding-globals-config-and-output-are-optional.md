---
format: aep.planning-md/3
id: story:cli-binding-globals-config-and-output-are-optional
kind: story
status: draft
title: A CLI binding may omit the config and output globals (ess-cli/2)
refs:
- provider: github
  reference: beyond10x/ess#481
relations:
- serves: vision:O2
revision: 1
---
## Outcome

A CLI that has only a state-directory flag can be described truthfully
(https://github.com/beyond10x/ess/issues/481): `ess-cli/2` and `ess-cli-plan/2` make `config` and
`output` optional; `state` stays required.

## Fit review (summary)

- Need: a CLI with only `--state-dir` must declare `--config` and `--output` it does not have;
  `null`, `~` and `""` are each read as a flag name (measured on 0.55.0, issue).
- Class: gap. `Globals` requires three Strings (`crates/specify/ess-cli-contract/src/wire.rs:91`);
  the generated runtime turns each into a clap flag (`ess-cli-project/src/runtime.rs:261-273`) and
  `output` selects JSON (`runtime.rs:424`).
- Considered: keep all three required and refuse null with a clearer message; optional `config`
  and `output` in a new format pair; optional `config` only. The second is chosen.

## Decisions

- **accept, redesigned**: a new format pair rather than relaxing `ess-cli/1` (a key becoming
  optional changes the envelope; `AGENTS.md`, Determinism and formats).

## Acceptance

- Spec first: the format change is declared where the repository declares formats before the
  generator and runtime change.
- `ess specify cli` admits an `ess-cli/2` binding without `config` and/or `output`, and still reads
  `ess-cli/1`; `null`, `~` and `""` are refused with a message naming the global.
- The compiled plan is `ess-cli-plan/2`; a generated CLI omits an absent flag; without `output`
  it prints JSON only. An `ess-cli/1` binding compiles to the same plan bytes as today.
- Old-reader compatibility tests; the reference pages and release notes name both formats and what
  a CLI without `output` does.
