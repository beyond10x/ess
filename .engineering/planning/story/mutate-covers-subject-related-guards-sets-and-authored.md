---
format: aep.planning-md/3
id: story:mutate-covers-subject-related-guards-sets-and-authored
kind: story
status: draft
title: mutate covers when_subject/when_related guards and every sets entry, and runs authored scenarios
tags:
- defect
refs:
- provider: github
  reference: beyond10x/ess#515
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
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
