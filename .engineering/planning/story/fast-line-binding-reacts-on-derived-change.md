---
format: aep.planning-md/3
id: story:fast-line-binding-reacts-on-derived-change
kind: story
status: draft
title: A binding cannot react only when a derived value changed
tags:
- fast-line
revision: 1
---
## Outcome

A binding can react in another domain only when a command changed a derived value, for example run a step only when a role change alters the user entitlement.

## Origin

Fast-line intake, 2026-10-04. On 2026-09-23 an agent run against a downstream identity service specification reported that a binding cannot express this conditional cross-domain reaction. Partly overlaps beyond10x/ess#268 (event-payload conditions on bindings) and #194 (conditional invocation); nobody answered the report.

## Open question

- Whether #268 event-payload `where` predicates cover it once the payload carries the before/after entitlement, or a derived-change condition is needed.
