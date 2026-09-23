---
format: aep.planning-md/1
id: story:an-existing-openapi-document-can-be-imported
kind: story
status: archived
title: An existing OpenAPI document can be imported
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The limitation

Re-probed against ESS 0.23.0 while adopting, and unchanged from what this repository already records:

| want | refusal |
|---|---|
| import a REST surface | `only OpenAPI 3.1 is supported, found 3.0.3` |
| after upgrading it | `supported objects must be closed with additionalProperties: false` |
| with query parameters | `1 coverage gap(s): /paths/…/get/parameters: interface feature not preserved` |

## Why it is filed here rather than left as a known gap

It is the cheapest path to the epic this decomposes. An adopter with an OpenAPI document already has
the resource shape, the envelope, the status codes and the query parameters that
`a-resource-has-an-http-shape-of-its-own` and `a-view-answers-a-request-not-only-a-model` ask to be
declared by hand. Importing one would close both for anybody who has a document — without them
writing a line.

The three refusals together mean almost no real document imports. The adopting project's three are
OpenAPI 3.0.0, 3.0.0 and Swagger 2.0, and the largest has 0 of its 30 object schemas closed. Being
pagination- and filter-heavy, the query-parameter gap alone would strip most of what makes them worth
importing.

## Acceptance

A 3.0 document is accepted or converted; an object without `additionalProperties: false` is admitted
as open rather than refused; and `in: query` parameters are preserved rather than reported as a
coverage gap.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
