---
format: aep.planning-md/3
id: story:a-command-acts-on-the-row-a-caller-attribute-names
kind: story
status: active
title: A command may act on the row a caller attribute names
summary: 'instance: caller.<attribute> is ESS-COMMAND-001 and in instances: where: ESS-COMMAND-003 on 0.93.0 (adopter report)'
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- informed_by: story:caller-value-source-and-guard
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:48Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:49Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

A downstream adopter's service has commands that act on the caller's own row: the request carries
no identity, and the row is the one an attribute of the authenticated caller names ("update my
settings"). `story:caller-value-source-and-guard` made the caller a value source and a guard
operand; an outcome's instance still has to be an input field. The adopter counts 31 open notes
on it, the largest group of its retrofit.

Refusals on 0.93.0 (`format: ess/37`, actor `demo.t.Owner` with attribute
`item_id: demo.t.ItemId`, command `demo.t.Touch` with input `label` only):

```yaml
outcomes:
  - name: touched
    updates: demo.t.Item
    instance: caller.item_id
```

```
error[ESS-COMMAND-001]: outcome `touched` of `demo.t.Touch` acts on the instance named by `caller.item_id`, which is no input field of it
```

```yaml
outcomes:
  - name: touched
    updates: demo.t.Item
    instances: {where: "id == caller.item_id"}
```

```
error[ESS-COMMAND-003]: `caller.item_id` reads `caller`, which is not a declared observable root
```

## Acceptance

- At a new source format, `instance: caller.<attribute>` validates on an outcome that updates,
  preserves or deletes, where the attribute is typed as the entity's identity and every actor that
  may run the command declares it; `instances: {where: ...}` may read `caller.<attribute>`.
- A caller attribute of another type, and an actor of the command without the attribute, are
  refused naming the attribute and the actor. `creates:` with a caller instance is refused or
  admitted by a stated rule.
- `unknown_instance: true` on such a command answers when the caller's attribute names no row.
- Synthesis arranges the row the caller names and a second row of another caller; the scenario
  expects only the first to change. `ess verify conform run --target interpreted` passes; a target
  that changes another row fails.
- An `ess-http/1` route of the command needs no path or body parameter for the instance; OpenAPI
  shows none.
- Below the new format both spellings keep today's refusals.

## Evidence

Both refusals were rerun for this story on `ess 0.93.0` and are quoted verbatim. The count of
notes is the adopter's. A view reading the caller is
`story:a-view-selects-rows-by-a-caller-attribute`.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).

Scope of the acceptance: `instance: caller.<attribute>`. The form `instances: {where: ...}`
reading `caller.<attribute>` is held: it reads the caller in a row filter, which
`story:a-view-selects-rows-by-a-caller-attribute` asks for views, and that draft is not decided.
The fit review splits the held half off or keeps it held until then.
