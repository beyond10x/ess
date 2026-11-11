---
title: Cover a refusal synthesis could not witness
sidebar_position: 2
description: "Mark a synthesis refusal as covered by an authored scenario that reaches the refused outcome, with covers: in ess-scenario/7."
---

# Cover a refusal synthesis could not witness

A synthesis refusal such as `ESS-SYNTH-018` leaves the declared coverage incomplete: the suite says it
could not witness an outcome, and `ess verify conform synthesize --suite-format 5` exits 1 until
something answers it. An authored scenario that reaches the refused outcome answers it when it says so.
From `type: ess-scenario/7` a document lists the scenario ids it covers:

```yaml
type: ess-scenario/7
covers:
  - example.review.TakeNote/outcome/too-many-tags
domain: example.review
scenario: a-note-with-too-many-tags
summary: A note sent with 2001 tags, one past the 2000 a note carries, is refused before the note of that id is found.
timeline: [...]
```

Each entry is the exact id the refusal names. Nothing is matched by command or outcome: an authored act
that ends in the refused outcome claims nothing until `covers:` names it. The refusal stays in the
suite, and the coverage suite (`ess-conformance/65`) marks it `covered_by` the authored scenario. The
synthesis then exits 0 and its summary ends `, 1 covered by authored`.

The mark is a claim the run checks. `ess verify conform run` reports declared coverage complete only
when the covering scenario is selected and every selected scenario passes. A covering scenario that
fails fails the run, and a child suite from `ess verify conform select` that leaves it out is reported
inconclusive. An id synthesis did not refuse is `covers.unknown-scenario`, naming the id, and nothing is
written. Below `ess-scenario/7`, `covers:` is refused as an unsupported key.
