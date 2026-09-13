---
format: aep.planning-md/1
id: story:a-synthesized-suite-has-a-compact-form
kind: story
status: archived
title: A synthesized suite can be written in a form whose size is proportional to what it asserts
summary: 'Measured: 13 MB for two suites, 24,961 bytes per scenario, 26,527 of 28,534 steps the same negative obligation. Compact JSON is 40% smaller and the writer has no flag.'
relations:
- informed_by: task:ess-gaps-measured-in-a-consumer-specification
revision: 3
---
**Folded into `task:ess-gaps-measured-in-a-consumer-specification` on 2026-09-11**, at the operator's
request that all eight measured gaps arrive as one task for the maintainer to decompose. The body below is
kept because it carries the full argument, the measurements and the candidate resolutions; the task
summarises it. Archived rather than deleted so the record of what was drafted survives.

---

## What is missing

`ess verify conform synthesize --target ir` writes the suite pretty-printed, and there is no other option.

Measured in an internal specification repository, 2026-09-11: two committed suites, **13 MB**,
**24,961 bytes per scenario**. Of 28,534 steps, **26,527 are `expect_no_event`** — the same negative
obligation repeated for every event the scenario does not expect. Compact JSON was measured at 4.3 MB, a 40%
reduction, and the writer has no flag for it.

That size is carried by every consumer that commits the suite as a drift-checked artifact, which is the
pattern the format invites: the suite is regenerated and byte-compared, so it lives in the repository and in
every clone, diff and review of it.

## What would settle it

Either is fine and the design chooses:

1. **A compact writer.** `--format compact` or equivalent; the document's meaning and digest are unchanged,
   only its whitespace.
2. **A denser representation of the negative obligations.** 93% of the steps say "and not this event"; a
   scenario could carry the set once rather than one step per member. This changes the document shape, so it
   is a format question rather than a writer flag, and it is the larger win.

Option 2 is worth measuring before choosing: if the negative set is identical across most scenarios of a
system, it may belong on the suite rather than on each scenario.

## Acceptance

- [ ] A synthesized suite can be written in a form whose size is proportional to what it asserts.
- [ ] The compiled meaning, the digest and the runner's behaviour are unchanged by the choice.
- [ ] A consumer that commits the suite as a drift-checked artifact sees a materially smaller file; the
      the internal specification repository pair is the measured case.

## Provenance

Filed 2026-09-11 from an internal application repository wave 3,
`review-result:adversary-conformance-suite-pass-1`, closing note.
