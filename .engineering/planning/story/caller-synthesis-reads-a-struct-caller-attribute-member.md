---
format: aep.planning-md/3
id: story:caller-synthesis-reads-a-struct-caller-attribute-member
kind: story
status: draft
title: Caller synthesis reads a struct caller attribute through its member
summary: Conformance caller synthesis keeps only two-segment caller paths, so caller.org.id is not swapped or witnessed (turn 1 caller-attributes adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-w1-caller-adv-1
revision: 1
---
## Need

A guard may read an actor attribute that is a struct through its member, `when: caller.org.id !=
input.org_id`; validation admits it (`is_caller_path` counts any caller path of more than one
segment). Turn 1 fixed the OpenAPI `x-ess-caller` list for this
(story:caller-attributes-walk-every-caller-read-of-a-command). Its adversary
(review-result:zi-w1-caller-adv-1) found the same two-segment filter in conformance caller
synthesis (`crates/verify/ess-conformance/src/synthesize/caller.rs:1122`), so such a read is
neither swapped nor witnessed by the caller-swapped run.

## Acceptance

- A command whose guard reads `caller.<attribute>.<member>` gets the caller-swapped scenario a
  two-segment caller read gets, and it passes against the interpreted target.
- A test pins the struct-member case.

## Scope

- `crates/verify/ess-conformance/src/synthesize/caller.rs` (cited by the review)
