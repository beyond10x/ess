---
format: aep.planning-md/3
id: story:mutate-covers-subject-related-guards-sets-and-authored
kind: story
status: active
title: mutate covers when_subject/when_related guards and every sets entry, and runs authored scenarios
tags:
- defect
refs:
- provider: github
  reference: beyond10x/ess#515
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-11T11:03:11Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-11T11:03:11Z", actor: "human:timo", revision: 3}
---
## Outcome

`ess verify conform mutate` mutates `when_subject` and `when_related` predicates, generates a
`sets-drop` site for every `sets:` entry or states why not, and runs authored scenarios beside the
synthesized ones.

## Evidence

https://github.com/beyond10x/ess/issues/515 (ess 0.56.0): on a model with 10 `when_subject`/`when_related` guards and
3 `sets:` sites, 53 mutants, none on a subject or related predicate and none `sets-drop`; 15
unwitnessed because the baseline refuses their generated scenario while 10 authored scenarios
that would witness them are not run (`mutate` takes no `--scenarios`).

## Acceptance

- Guard mutants (negate, connective, comparison boundary) are generated for `when_subject` and
  `when_related` predicates and scored as `when:` mutants are.
- Each `sets:` entry yields a `sets-drop` site, or the audit lists the entry with the reason it
  has none.
- `mutate --scenarios <dir>` runs authored scenarios for the baseline and every mutant; a mutant
  only an authored scenario pins is killed by it.
- The manifest format moves only if a persisted field changes meaning, decided explicitly.

## Split

Three units: predicate operators; sets-drop sites; `--scenarios`. They share `mutate.rs`, so they
run one after another.

## Scope (delivered in wave 2026-10-11j)

| unit | commits | files |
|---|---|---|
| guard mutants on `when_subject` / `when_related` | 8957fc5feb, b3b0a09ede | `crates/verify/ess-conformance/src/mutate.rs`, `crates/edge/ess-cli/tests/mutate_stored_guards.rs`, `crates/verify/ess-conformance/tests/mutation_stored_guards.rs`, fixture `mutation-stored-guards.yaml`, `synthesis-suite-bytes-table.tsv` |
| a `sets-drop` site or a stated reason for every `sets:` entry | c8ef841803, c5d27cce85 | `mutate.rs`, `crates/edge/ess-cli/src/main.rs`, `tests/mutation_sets_entries.rs`, `ess-cli/tests/mutate_sets_entries.rs`, fixture `mutation-sets-entries.yaml`, the guide `website/docs/guides/verify/mutation-audit.md` |
| `mutate --scenarios` on the interpreted target | 1cafd5a1f7, 903d0e51d1, 801fb4df42, 6df02180d0 | `mutate.rs`, `now_offset.rs`, `interpret/bindings.rs`, `ess-cli/src/main.rs`, `tests/mutation_authored.rs`, `ess-cli/tests/mutate_authored.rs`, `tests/mutation_identical_answer.rs`, the two adversary files, `docs/design/mutation-audit-and-model-runner.md`, the four repository-wide tables |

Reasons for an entry with no site: `creates_row`, `source_not_input`, `optional_field`, `no_row`,
printed as notes on stderr (not persisted; no format moves). `--scenarios` runs only with
`--target interpreted`; authored scenarios run on the interpreter of the specification and of each
mutant, on the scenario clock `run` uses. Two adversary passes
(review-result:adversary-wave-20261011j-j3-pass-1, -pass-2), both fixed. Left for later:
story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock (the synthesized half has
no clock, since the interpreted target joined the audit); a `when_subject` guard on the updating
branch can hide a `sets-drop` survivor (synthesis does not move the input apart under that guard;
reproduced, cause unconfirmed).
