---
format: aep.planning-md/3
id: story:synthesis-witnesses-stay-unique-per-arranged-instance
kind: story
status: draft
title: A witness drawn for a length guard stays unique per arranged instance
refs:
- provider: github
  reference: beyond10x/ess#480
relations:
- serves: vision:O2
revision: 1
---
## Outcome

A guard written `id.utf8_bytes < 1` no longer makes synthesis draw the same literal `é` for every
arranged instance (https://github.com/beyond10x/ess/issues/480). Measured on 0.55.0: the issue's
`catalog` reproducer arranges three items in `AddNote answers added` and all three carry `é`;
written `id == ""`, the ids are distinct.

## Acceptance

- In the issue's reproducer the three arranged items carry three distinct identities under both
  guard forms, and each satisfies the guard's negation.
