---
format: aep.planning-md/3
id: story:a-view-selects-rows-by-a-caller-attribute
kind: story
status: draft
title: A view may select rows by an attribute of the caller
summary: caller.<attribute> in a view filter is ESS-VIEW-003 on 0.93.0 (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
revision: 3
---
## Need

A downstream adopter's reads answer "the rows of the caller's own account" or "the caller's own
row": the selection comes from who asks, not from a parameter. A view filter cannot read the
caller, so the caller's attribute must be modelled as a view parameter, which any caller could set
to another value. The adopter counts about 45 open notes on it.

Reproduced on 0.93.0 (`ess/37`; `view-paging.yaml` with the actor given
`attributes: [{name: desk, type: demo.jobs.JobType}]`, which validates, and the view's filter
changed to `type == caller.desk`):

```text
  - error[ESS-VIEW-003]: `caller.desk` reads `caller`, which is not a declared observable root; a view cannot select on something its source never observes
  `ess-domain` refuses this as `unobservable_fact`
```

Related: `story:caller-attributes-are-readable-in-a-when-related-predicate` and
`story:a-caller-attribute-is-compared-with-each-element-of-a-stored-list` cover command guards, not
views.

## Acceptance

- At a new source format a view `filter:` may compare a projected or source field with
  `caller.<attribute>` using `==` or `!=`, alone or under `all:`/`any:`. The view then declares
  which actors may read it, and every one of them declares the attribute.
- Synthesis reads the view as two callers whose attribute differs and expects each to see only its
  own rows; a target that answers the other caller's rows fails. A read with no caller is refused
  by name.
- `ess-http/1` routes such a view without a parameter for the attribute; OpenAPI lists none.
- A model without a caller read in a view keeps its IR bytes and its suite bytes.

## Related (2026-11-25)

A second adopter reports the same need from the HTTP binding (twentieth gap report, shape 1a): a view
parameter such as `account_id` that the real service takes from the bearer token becomes a query
parameter the service ignores and any caller can set. The binding binds a model parameter only from the
path, the query or the body, and no mapping from a caller attribute exists or is proposed
(`decision-blocker:the-caller-reaches-a-view-or-a-route-only-through-declared-attributes`, HC-D1,
security-marked, with the owner, fail-closed A: the caller stays a declared attribute, read in the
view filter, and the binding adds no source). This story is the model half of HC-D1 A. For a
command the same need is expressible since 0.38.0: the account is not an input, and the stored field is
set `{caller: account_id}` (`docs/design/caller-values.md:29`, `:44`).

Two additions follow from the report and wait for the decision:

- The one-row answer of such a view (`body: single`) needs `selects_identity` to count an identity
  compared with `caller.<attribute>` as a selection by identity, as it counts `param.<name>`
  (`crates/specify/ess-http/src/lib.rs:8807-8824`; the same widening for a literal is
  `story:a-single-view-may-select-its-row-by-a-literal-identity-and-a-literal-route-serves-it`, HB-D2).
- The "who am I" read (shape 1b) is `story:a-view-may-answer-the-row-that-describes-the-reading-actor`.

`story:a-view-may-scope-rows-to-the-reading-actor` (draft) is an older statement of this need (a protocol specification's
gap); the two are one feature, and the owner's pick under HC-D1 decides which survives.
