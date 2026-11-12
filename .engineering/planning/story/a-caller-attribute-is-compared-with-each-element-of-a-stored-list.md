---
format: aep.planning-md/3
id: story:a-caller-attribute-is-compared-with-each-element-of-a-stored-list
kind: story
status: active
title: A caller attribute is compared with each element of a stored list
summary: A when_subject quantifier over a stored list refuses a binder compared with caller.<attribute> as ESS-COMMAND-003 (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:34Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:35Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

A downstream adopter reports against 0.90.0, and it reproduces on 0.93.0 (`ess/32`): a
`when_subject` guard can compare a stored scalar with the caller (`owner != caller.participant_id`),
but not each element of a stored list with the caller. The rule "the owner or a member may end the
session" needs the element form. Minimal specification (domain `demo.s`), entity
`demo.s.Session {session_id, owner: ParticipantRef, members: List<ParticipantRef>}`, actor
`demo.s.Participant` with `attributes: [{name: participant_id, type: demo.s.ParticipantRef}]`:

```yaml
  - name: demo.s.End
    input:
      - {name: session_id, type: demo.s.SessionId}
    outcomes:
      - name: outsider
        when_subject:
          predicate:
            all:
              - owner != caller.participant_id
              - {not: {exists: {in: members, as: m, that: m == caller.participant_id}}}
        error: demo.s.Refused
      - name: ended
        moves: demo.s.Session.end
        instance: session_id
```

```text
error[ESS-COMMAND-003]: `caller.participant_id` reads the caller, and a caller attribute is compared with `==` or `!=` against a field and nothing else
  `ess-domain` refuses this as `unobservable_fact`
```

The binder `m` is an element of the stored field `members`, so the comparison is still a caller
attribute against a stored value. The refusal comes from the binder not being counted as a field.

Spelling that works on 0.93.0 today: an input field bound to the caller, with the quantifier
comparing against the input. `ess specify validate` exits 0, and `ess verify conform run --target
interpreted` passes 6 of 6 scenarios:

```yaml
    input:
      - {name: session_id, type: demo.s.SessionId}
      - {name: requester, type: demo.s.ParticipantRef}
    outcomes:
      - name: impersonated
        when: requester != caller.participant_id
        error: demo.s.Refused
      - name: outsider
        when_subject:
          predicate:
            all:
              - owner != input.requester
              - {not: {exists: {in: members, as: m, that: m == input.requester}}}
        error: demo.s.Refused
```

The same report asks for the second form, "a related row holding the caller exists", through a
row-set selector. `story:caller-attributes-are-readable-in-a-when-related-predicate` already covers
that form. Two more limits seen in the reproduction belong to other stories, not to this one:
- With the input workaround, a row-set guard on a command that moves an existing subject validates,
  but synthesis gives its branches no scenario (`ESS-SYNTH-001`).
- An owner check (`when_subject`) beside a row-set guard in one command is refused as
  `ESS-COMMAND-009`.

## Acceptance

- `ess specify validate` on the minimal specification above, at the new source format, exits 0.
  Under an earlier header the same guard is refused as `unsupported_format_version` naming the new
  format, no longer as `ESS-COMMAND-003`.
- Inside a quantifier over a stored list or map in `when_subject:`, a binder compared with `==` or
  `!=` against `caller.<attribute>` validates. An ordering, or a comparison with a literal, stays
  `unobservable_fact`, as in `when:`.
- Synthesis witnesses the branch on both sides: the arranged row holds the caller's value as an
  element that is neither first nor last, and the row holds no element equal to it. The interpreted
  target runs both scenarios and passes. A target that reads only the first element, or only
  `owner`, fails one of them.
- The compiled IR marks the branch as decided by the caller, so `ess generate openapi` lists
  `participant_id` under `x-ess-caller`.
- `docs/design/language-grids.md`, grid R, row `when_subject:`, column caller states the quantifier
  binder form. `docs/design/caller-values.md` and `website/docs/reference/predicates.md` (comparing
  a stored field with the input / caller) state it.

## Grid

Grid R, row `when_subject:`, column caller: today "yes (`ess/16`)" for a stored scalar, refused for
a quantifier binder over a stored list. The story makes the cell whole.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).
