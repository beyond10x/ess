---
format: aep.planning-md/3
id: story:a-scenario-with-a-repeated-key-is-refused
kind: story
status: draft
title: A scenario document that repeats a mapping key is refused, as a model file is
summary: ess-scenario/1 silently takes the last of two equal keys; model YAML refuses the same repetition.
tags:
- downstream
- ess-issues-3
revision: 1
---
## Problem

An authored scenario (`ess-scenario/1`) that repeats a mapping key is read without complaint, and
the last value wins. A model file that repeats a key is refused ("duplicate entry with key
`version`"), so the two readers of one repository's YAML disagree. Reported 2026-10-29 by an
adopter on ESS 0.75.0: a step's `input:` named `tenant` twice.

Checked on the 0.77.0 head (84671d6, `ess 0.77.0`) against the adopter's model at its commit
c947b74:

- With the key repeated with the same value, `ess verify conform synthesize` exits 0: "183
  scenario(s) (34 authored), 0 refusal(s)". `ess specify validate` exits 0 and reports the
  scenarios valid.
- With the second value changed to `tenant-other`, synthesis still exits 0, and the compiled step
  sends `"tenant":{"kind":"literal","value":"tenant-other"}`. The first value is dropped without
  a word.
- The same repetition in `system.yaml` is refused by validate.

## Acceptance

- A repeated mapping key at any depth of an `ess-scenario/1` document is refused when the
  scenario is read (validate and synthesize alike), naming the file, the key and both positions.
- Equal keys in separate mappings remain valid.
