---
format: aep.planning-md/3
id: story:an-authored-scenario-retires-the-refusal-it-covers
kind: story
status: draft
title: An authored scenario retires the refusal it covers
summary: 'Authored scenarios never retire the refusal they cover, and synthesize --path ignores ess-inputs.yaml scenarios: (beyond10x/ess#735)'
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#735
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/coverage.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/spec.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: inferred
  path: docs/design/review-conformance-coverage.md
- confidence: inferred
  path: website/docs/guides/verify/author-scenarios.md
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 5
---
## Need

A synthesis refusal's help line sends the author to "cover the state with an authored scenario
(ess-scenario/1)". Once that scenario is in the suite and passes, the refusal is still counted, so
declared coverage can never be complete and the gate stays red. Minimal reproduction (ess 0.92.0;
the specification of beyond10x/ess#733 with `scenarios/confirm-twice.yaml`, an `ess-scenario/1`
document whose last act is `ConfirmDecision` -> `already-exists`, listed in `ess-inputs.yaml`):

```text
$ ess verify conform synthesize --path spec --scenarios spec --out with.json
refused: refusal[ESS-SYNTH-004]: outcome example.review.ConfirmDecision/already-exists has no scenario `example.review.ConfirmDecision/outcome/already-exists`
  ...
  help: ..., or cover the state with an authored scenario (ess-scenario/1)
11 scenario(s) (1 authored), 1 refusal(s), written to with.json
$ ess verify conform run --target interpreted --path spec --suite with.json --report-format 2
  passed example.review/authored/confirm-decision-already-exists
  11 scenarios: 11 passed, 0 failed, 0 error, 0 unsupported
$ ess verify conform synthesize --path spec --scenarios spec --suite-format 5 --out with-coverage.json
refused[ESS-SYNTH-004]: ...
11 selected scenario(s), 1 authored source(s), 1 refusal occurrence(s)
[exit 1]
coverage.counts: {"authored": 1, "generated": 10, "outside": 0, "refused": 1}
coverage.refused[*].scenario: ["example.review.ConfirmDecision/outcome/already-exists"]
```

Complete coverage requires "no refusal occurrence with scope in_scope"
(`docs/design/review-conformance-coverage.md:147`), and nothing links an authored scenario to the
refusal it was written for. The issue's second request, that `synthesize --path` read the manifest's
`scenarios:` list, is not part of this story (declined with the idiom; see Decisions).

## Acceptance

Under the recommended answer to the blocker (an exact, author-declared link):

- An authored scenario declaring that it covers
  `example.review.ConfirmDecision/outcome/already-exists`, synthesized with `--suite-format 5`,
  gives a suite whose `coverage.refused` still holds the occurrence, marked covered by
  `example.review/authored/confirm-decision-already-exists`; the command prints
  `..., 1 refusal occurrence(s), 1 covered by authored` and exits 0.
- `ess verify conform run --target interpreted --report-format 2` on that suite reports coverage
  complete when the covering scenario passes, and inconclusive when it fails or is filtered out.
- A link naming a scenario id the synthesis did not refuse is refused before output, naming the id
  (as `known-failures.unknown-scenario` is for known failures).
- A suite without such a link keeps its format and bytes; the text summary of ordinary
  (`--suite-format 4`) synthesis is unchanged.

## Scope

- inferred: docs/design/review-conformance-coverage.md
- inferred: crates/verify/ess-conformance/src/coverage.rs
- inferred: crates/verify/ess-conformance/src/coverage_build.rs
- inferred: crates/verify/ess-conformance/src/authored.rs
- inferred: crates/specify/ess-domain/src/spec.rs
- inferred: crates/edge/ess-cli/src/coverage.rs
- inferred: website/docs/reference/formats.md
- inferred: website/docs/guides/verify/author-scenarios.md
confidence: low

Split once the blocker is answered: design page; authored document key and its reader; coverage
record and the Rust reader; the Go and TypeScript coverage readers.

## Fit review

1. **Need.** An author who follows the refusal's own help cannot reach complete declared coverage;
   the authored scenario passes but the refusal is never discharged (repro above, triage
   `repro/run.txt`). The requester asked that the refusal be "reported as covered (or moved to a
   covered-by-authored count)".
2. **Class: gap.** No construct links an authored scenario to a generated obligation;
   `docs/design/synthesis-seeds.md:17-19` records that "an authored scenario is appended after
   synthesis and does not discharge a generated obligation", and the coverage contract counts every
   in-scope refusal against completeness (`review-conformance-coverage.md:146-147`).
3. **Already expressible?** No. A seed discharges only a subject-row obligation or a counter limit
   (`synthesize --help`, `--synthesis-seed`), not an existence refusal. `--known-failing` covers
   failed scenarios, not refusals (`website/docs/reference/formats.md:390`).
4. **Fit.** Not a grid R or O cell (`docs/design/language-grids.md`); it is the coverage contract.
   Siblings: `known-failures` links by exact id, "no wildcard, prefix, command-wide or outcome-wide
   match", and is "never a marker on an authored outcome" (`formats.md:390`). An inferred link
   (any authored act expecting the refused outcome) would claim checks the generated scenario makes
   and the authored one may not (row unchanged, no event, every creating path:
   `docs/design/outcome-shapes.md:388-394`). Targets: Rust, Go and TypeScript coverage readers, the
   report, `ess verify diff` of suites; the interpreter is unaffected.
5. **Second adopter.** A specification whose view ordering refusal (`OrderUnwitnessed`) is checked
   by an authored scenario such as `examples/billing-scenarios/outstanding-invoices-rank-latest-first.yaml`
   wants the same gate to go green.
6. **Cost.** An authored link is a new key (an `ess-scenario` version) and a new field in the
   declared-coverage refusal record (a coverage suite pair); the text summary and report change.
   An inferred link costs no authored key but still the coverage field, and weakens the claim.
7. **Alternatives.** (a) Change nothing and fix the help line so it no longer promises discharge;
   (b) inferred link by command and outcome; (c) explicit link by exact scenario id, the occurrence
   kept and marked covered (recommended: matches known-failures, keeps multiplicity,
   `review-conformance-coverage.md:145`). The requester's "move to a covered count" is (b) or (c).

## Decisions

Accept, redesigned. The triage left one question open: may a passing authored scenario discharge a
generated synthesis refusal for declared-coverage completeness (`review-conformance-coverage.md:147`),
against `synthesis-seeds.md:17-19`; and if so, is the link an author-declared exact scenario id (new
`ess-scenario` key and coverage pair) or inferred from the authored act's command and outcome?

Decided 2026-11-09 by the coordinator of the zero-open-issues run, taking the triage's recommended
answer under the operator's standing rule "decide, don't ask; take the reversible option" (recorded in
the run's non-interactive approval-record): yes, by an exact author-declared scenario id; the refusal
occurrence is kept in the suite and marked covered, never deleted; declared coverage counts it complete
only when the covering scenario passes in the same run. Nothing is inferred from the authored act's
command and outcome. The design page `docs/design/review-conformance-coverage.md` gains the rule
first, beside the position in `synthesis-seeds.md:17-19`, which this narrows rather than reverses: a
synthesis refusal still stays a refusal; the covered mark is the author's claim, checked by the run.

The issue's second request, `synthesize --path` reading the manifest's `scenarios:` list, is declined
with the idiom: `docs/design/review-authored-discovery.md:253` decides that "omitted `--scenarios`
continues to mean no authored scenario selection, even when a model manifest has a scenarios list",
and `--scenarios <dir>` with an immediate `ess-inputs.yaml` selects its exact list
(`ess verify conform synthesize --path spec --scenarios spec` gives `11 scenario(s) (1 authored)`,
run.txt).

## Source

github:beyond10x/ess#735
