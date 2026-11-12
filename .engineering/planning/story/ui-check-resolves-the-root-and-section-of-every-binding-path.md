---
format: aep.planning-md/3
id: story:ui-check-resolves-the-root-and-section-of-every-binding-path
kind: story
status: active
title: ui check resolves the root and the section of every binding path
summary: ess ui check accepts an unknown binding root and a section path naming no section or no listed member, in a navigate parameter and a read's params, on 0.94.0 (adopter report)
tags:
- backlog
- defect
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- depends_on: story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:16Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:17Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Defect

`ess ui check` accepts a binding path whose root is no root of the expression grammar, and a
`section.<name>` path that names no section of the page or a member the reference does not give,
in a navigate parameter and in a read's `params`. The reference
(`website/docs/reference/ess-ui.md`, Expressions) says "a renderer evaluates them, a validator
resolves every path". A renderer then evaluates the path to nothing: the read goes out without its
parameter and a link has no target.

Reported by an adopter on 0.90.0 (2026-11-12); reproduced on the 0.94.0 release binary.

On `examples/partner-portal/ui.yaml`, line 339, the read parameter
`deal_id: section.deals.selection.id` replaced, then `ess ui check --path ui.yaml`:

| `deal_id:` | result |
|---|---|
| `section.deals.selection.id` (as committed) | 0 errors, 1 warning (the placeholder) |
| `bogusroot.x` | 0 errors, 1 warning |
| `section.nosuch.selection.id` | 0 errors, 1 warning |
| `section.deals.junk.id` | 0 errors, 1 warning |
| `section.deals.row.id` | 0 errors, 1 warning |

The same four paths in a navigate parameter (`navigate: {to: q, params: {id: <path>}}` on a
record's action, checked with `--model`) add no finding to the document's baseline either.

Related: `story:a-visible-expression-resolves-its-row-and-actor-paths-wherever-an-action-stands`
covers `row.` and `actor.` paths inside an action's `visible`. This story is the root and the
`section.` member, at every expression site.

## Acceptance

1. A path whose first segment is not a root of the reference's Expressions table is an error at
   the node path of the expression, naming the segment and the roots, in every expression site
   the document model types as an expression: a read's `params`, a navigate's `params`, `bind`,
   `visible`, `when`, `args` and a field's value.
2. `section.<name>` whose `<name>` is no section of the page is an error naming the page's
   sections; `section.<name>.<member>` whose member the reference does not list is an error
   naming the members.
3. The check runs without `--model`.
4. `examples/partner-portal/ui.yaml` passes with its one placeholder warning, and each row of the
   table above other than the first is a test case that fails before the change.
5. The list of expression sites the check walks is derived from the document model's `Expr`
   fields, so a new expression field is covered without a new rule.

## Evidence

Probe script, documents and output: session scratch `ess/adopter9/` (`probe.sh`, `probe.txt`,
`pp-*`; not in the repository).

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).

Review note: acceptance point 2 (the members of `section.<name>`) depends on the choice
`story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither` makes
for `section.<name>.row`. That story is settled first; this one enforces the list it leaves in
the reference.
