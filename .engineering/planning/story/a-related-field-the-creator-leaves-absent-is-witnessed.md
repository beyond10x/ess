---
format: aep.planning-md/3
id: story:a-related-field-the-creator-leaves-absent-is-witnessed
kind: story
status: implemented
title: A when_related predicate over a field the creating command leaves absent is not witnessed
refs:
- provider: github
  reference: beyond10x/ess#239
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T05:37:20Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T05:37:20Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T09:58:25Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

A `when_related:` predicate that holds while a related row's optional field is still absent (`{site: {exists: false}}`, the field set only by a later command) is witnessed on the row as its creating command leaves it (beyond10x/ess#239).

## Acceptance

- the #239 shape synthesizes the branch on a related row created without the field, and its sibling on a row where a later command set it;
- a target reading the field as present fails;
- committed suites regenerate byte-identical or the change is listed in CHANGELOG.
