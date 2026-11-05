---
format: aep.planning-md/3
id: story:a-uniqueness-create-read-by-identity-passes-on-the-interpreter
kind: story
status: draft
title: A create under a uniqueness check, read by identity, passes on the interpreter
summary: With a when_related row-set uniqueness check, a view by identity and no examples, the synthesized created scenario expects unsuffixed values the arranged inputs suffix (0.88.0 adversary pass, pre-existing)
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

Found by the 0.88.0 adversary pass on the suffixed-instance unit, pre-existing and outside its
acceptance: a create guarded by a `when_related` row-set uniqueness check, with a view read by
identity and no input examples, synthesizes a `created` scenario that fails on the interpreter. The
view is expected to hold `name = "name", widget_id = "widget_id"` and holds
`name = "name-1", widget_id = "widget_id-1"`: the scenario's expectation and the arranged inputs
disagree on the suffix.

## Acceptance

- `a_uniqueness_create_read_by_identity_passes_on_the_interpreter`: that model's synthesized suite
  passes on the interpreted target with no refusal.
