---
format: aep.planning-md/3
id: story:validate-reports-the-authored-scenario-that-covers-each-unscenarioed-synthesis-refusal-beside-it-and-counts-them-without-changing-what-unscenarioed-means
kind: story
status: draft
title: Validate reports the authored scenario that covers each unscenarioed synthesis refusal beside it and counts them, without changing what unscenarioed means
summary: validate counts a refusal an authored scenario covers as unscenarioed while synthesize counts it as covered by authored; validate adds covered_by and counts.covered_by_authored (D155 = A, downstream report 2026-11-26)
tags:
- external-request
- issues-2026-11-26
relations:
- serves: vision:O2
- informed_by: story:feature-request-434
- informed_by: story:an-authored-scenario-retires-the-refusal-it-covers
revision: 1
---
## Need

A downstream application reported on 0.99.2 (received 2026-11-26) that `ess specify validate --format json` counts as unscenarioed a refusal that an authored scenario already covers, while `ess verify conform synthesize` ends with "1 refusal occurrence(s), 1 covered by authored". The two figures read as a contradiction to a person who gates a pipeline on the validate figure.

Verdict: intended as written, not a defect. `completeness.unscenarioed` lists every synthesis refusal of the model synthesized without seeds (`crates/edge/ess-cli/src/main.rs`, `Completeness::of`), and the validate guide states that `unscenarioed` equals the refusal figure of synthesis (`website/docs/guides/specify/layout-and-validation.md`, line 178). `docs/design/review-conformance-coverage.md` § "An authored scenario covers a generated refusal" keeps a refusal a refusal for every reader that does not read the `covers:` mark. The synthesize figure counts the covered ones inside the refusal figure, so the two numbers agree on what a refusal is and differ on whether a covered one is shown apart.

The gap is the report, not the meaning: validate reads no authored scenario, so it cannot say which entries are covered. Subtracting the covered entries would change the documented meaning of `unscenarioed` and break "a refusal stays a refusal" for the reader that ignores the mark.

## Outcome

`ess specify validate --format json|yaml` reads the `covers:` marks of the authored scenarios, from the same `--scenarios` source synthesize reads when it is given, and reports both: each covered entry of `completeness.unscenarioed` carries `covered_by: [<scenario id>, ...]` (written only when non-empty), and `completeness.counts.covered_by_authored` (written only when `ess-inputs.yaml` lists scenarios) counts the covered entries. The documented meaning of `unscenarioed` is unchanged: it still equals the refusal figure of synthesis.

## Acceptance

- Red first: a model with one command, one synthesis refusal and one authored scenario whose `covers:` names it gives `completeness.counts.covered_by_authored: 1` and `covered_by` on that entry; before the change the key is absent.
- Without authored scenarios the printed report is byte-identical to the one before.
- `unscenarioed` still equals synthesize's refusal figure on the same model.
- The validate guide's sentence on `unscenarioed` says the entry may carry `covered_by`, and that the count keeps its meaning.
- The keys are printed additions to the report, not a format of a document; an older reader of the report that rejects unknown keys is the one named in the changelog line.

## Fit review

1. **Need.** A pipeline reads one command and learns which synthesis refusals an authored scenario covers. Reproduction above: one command, one refusal, one authored scenario with `covers:`.
2. **Class.** *Convenience* that a report cannot express today: the fact is in the inputs, and validate does not read it.
3. **Already expressible?** Yes by running `synthesize` and reading its "covered by authored" line, but that figure is text, not the validate JSON a gate reads.
4. **Fit.** The key is printed output, no format document. It reuses the `covers:` mark and the `--scenarios` source that synthesize reads, and it adds to the entries `unscenarioed` already has (`caused_by`, `unanswered`).
5. **Second adopter.** A team gates CI on "`unscenarioed` does not grow" and wants the covered ones apart.
6. **Cost.** Two additive printed keys; validate reads the authored scenarios; no refusal code.
7. **Considered.** (a) Change nothing. (b) Subtract the covered entries: it breaks the documented meaning and the design page. (c) Report both (chosen).

## Decisions

D155 = A (the operator session on the owner's delegation, 2026-11-26): `validate` reports `covered_by` per entry and `counts.covered_by_authored`; the documented meaning of `unscenarioed` is unchanged. The options were A, report both; B, subtract the covered entries and keep the keys (breaks the documented meaning); C, change nothing.

## Delivery

Branch `issues/open-2026-11-26-c` at `6dcc7a8873`, merged into the open-issues wave; not released when written.

## Scope

Cited: `crates/edge/ess-cli/src/main.rs` (`Completeness::of`), `crates/edge/ess-cli/tests/validate_completeness.rs`, `website/docs/guides/specify/layout-and-validation.md`, `docs/design/review-conformance-coverage.md`, `CHANGELOG.md`.
