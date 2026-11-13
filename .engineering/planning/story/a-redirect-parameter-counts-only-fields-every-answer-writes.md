---
format: aep.planning-md/3
id: story:a-redirect-parameter-counts-only-fields-every-answer-writes
kind: story
status: active
title: A redirect parameter counts only fields every answer writes
summary: Written::of takes a newtype over Optional and a wrong_state error's field under the unknown-instance answer as always written, so a redirect may read a parameter an answer lacks (found by reading in a review pass, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
relations:
- serves: vision:O2
- informed_by: story:two-errors-of-one-wire-name-and-different-fields-bind-on-one-status
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-13T11:02:41Z", actor: "agent:operator-delegate", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-13T11:02:41Z", actor: "agent:operator-delegate", revision: 3}
---
## Finding

`ESS-HTTP` holds a redirect's parameters to values every answer of the outcome writes. `Written::of`
(`crates/specify/ess-http/src/lib.rs`, `fn of` of `struct Written`, lines 6221-6236 at
`2b4e28a786`) counts a parameter read from an error field as always written unless the field's type
is `Optional` as written:

```rust
Ok((_, type_ref)) => !matches!(type_ref, Some(ResolvedTypeRef::Optional { .. })),
```

Two kinds of field pass that test and can be missing from an answer:

- a field typed by a newtype over `Optional<...>`, which the model admits;
- a field of a `wrong_state` error where the outcome answers an unknown instance: by
  `docs/design/unknown-instance-seams.md` that answer writes none of the error's fields.

A redirect whose parameter reads such a field is then admitted, and the answer that lacks the
field has no value for the parameter.

Found by the first review pass of
`story:two-errors-of-one-wire-name-and-different-fields-bind-on-one-status`, by reading the code at
that unit's base; no test and no run. That unit's rule for two errors of one wire name had the same
premise and is corrected there; the redirect rule was so before it and is this story.

## Acceptance (sketch, to be fixed by the fit review)

- Two tests, red before: a redirect parameter read from a field typed by a newtype over `Optional`,
  and one read from a `wrong_state` error's field of a command that moves, updates or deletes an
  instance named in its input, are refused as a parameter not every answer writes, with the message
  the rule uses today.
- The question "does every answer of this outcome write this field" is asked by one function, the
  one the sibling unit uses for its field rule.
- A binding 0.94.0 admitted and whose every answer does write the field stays admitted.

## Fit review

Owed. This draft records a finding by reading only.
