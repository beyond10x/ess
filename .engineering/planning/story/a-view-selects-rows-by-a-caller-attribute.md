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
revision: 1
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
