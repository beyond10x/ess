---
format: aep.planning-md/3
id: specification:synthesis-refusals-are-read-from-a-coverage-suite
kind: specification
status: draft
title: Synthesis refusals are read as data from a coverage suite
summary: 'Decline of a machine-readable refusal list on synthesize --format json, with the idiom: coverage.refused of a --suite-format 5 suite, run on 0.96.0'
tags:
- adopter-report
relations:
- informed_by: epic:adopter-gaps-verdicts
- serves: vision:O2
revision: 1
---
## Statement

The machine-readable list of synthesis refusals is `coverage.refused` of a coverage suite. `ess
verify conform synthesize --path <specification> --suite-format 5 --out <suite.json>` writes it;
`--format json` on an ordinary suite prints the suite (`events`, `provenance`, `scenarios`) and no
refusal, and is not the route.

Answers a request of 2026-11-21 from a downstream prototype building a ratchet gate (the refusal
count may not rise; a domain a change touches must have none), which asked for "a
machine-readable refusal list from `ess synthesize --format json`" carrying, per refusal, a stable
code, the domain, the declaration and the source location. Decision: **decline, with this idiom**.
Three of the four members exist. The fourth, the source location of the declaration, does not; the
requester was told so and asked to say what it needs it for.

## Sources

- `crates/verify/ess-conformance/src/coverage.rs:288`: `pub struct Refusal`, "One actual refusal
  occurrence. Identical records retain multiplicity", with `origin`, `scenario`, `subject`,
  `source`, `code`, `message`. `source` is the rejected authored scenario's source identity, null
  for a generated candidate. It is not the location of the declaration.
- `website/docs/guides/verify/runners.md:236-245`: "Opt into declared coverage", the
  `--suite-format 5` command.

## The idiom

```console
$ ess verify conform synthesize --path examples/billing --suite-format 5 --out suite.json
32 selected scenario(s), 0 authored source(s), 0 refusal occurrence(s)
$ jq -c '.coverage.counts' suite.json
{"authored":0,"generated":32,"outside":0,"refused":0}
```

Run on 0.96.0, 2026-11-21. On a downstream specification whose text output prints 45 `refused:`
lines, the same command wrote `coverage.counts.refused: 45` and 45 records in `coverage.refused`
(codes `ESS-SYNTH-001`, `-003`, `-004`, `-008`, `-012`, `-013`), each with the keys `code`,
`effect`, `message`, `needs`, `origin`, `retained`, `scenario`, `scope`, `source`, `subject`.
`coverage.knowledge` was `complete_inventory`.

What a gate reads:

- the count: `.coverage.counts.refused`;
- the stable code: `.coverage.refused[].code`;
- the declaration: `.coverage.refused[].subject`, `{kind, name}` with `kind` one of `entity`,
  `type`, `outcome`, `transition` in that run; `name` is the qualified name, or
  `{command, outcome}` for an outcome and `{entity, transition}` for a transition;
- the domain: the qualified name without its last segment (`<system>.<domain>.<Name>`);
- the candidate scenario: `.coverage.refused[].scenario`.

## What stays open

- Under `--format json` with an ordinary suite, the 45 `refused:` lines of the text output are
  printed nowhere (observed 2026-11-21 on 0.96.0: 0 lines on either stream, against 45 in text
  mode). Whether that is intended is not settled here.
- No record carries the file and line of the declaration it is about.
