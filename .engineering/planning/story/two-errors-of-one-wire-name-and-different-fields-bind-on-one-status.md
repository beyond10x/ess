---
format: aep.planning-md/3
id: story:two-errors-of-one-wire-name-and-different-fields-bind-on-one-status
kind: story
status: active
title: Two errors of one wire name and different fields bind on one status
summary: 'ESS-HTTP-025 refuses the pair without alike: (cannot be told apart) and with it (the wire tells them apart) on 0.93.0 (adopter report)'
tags:
- backlog
- defect
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
- informed_by: story:two-outcomes-answered-alike-may-share-a-status-and-body
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:23Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:25Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Defect

Two outcomes of one command refuse with errors that share a wire name and differ in their fields.
On one status with `{error: error}` no `ess-http/1` binding of the pair validates on 0.93.0, and
the two refusals give opposite reasons: without `alike:` the bodies "cannot be told apart", with
`alike:` "the wire tells them apart".

Model (`format: ess/37`, validates):

```yaml
errors:
  - name: demo.t.Boom
    naming: {wire: internal_error}
    fields:
      - {name: message, type: String}
  - name: demo.t.BoomMore
    naming: {wire: internal_error}
    fields:
      - {name: message, type: String}
      - {name: details, type: String}
```

Command `demo.t.Rename` has outcomes `boom` (`error: demo.t.Boom`) and `boom-more`
(`error: demo.t.BoomMore`). Binding:

```yaml
routes:
  - command: demo.t.Rename
    method: put
    path: /api/items/{id}
    parameters: {id: input.id}
    outcomes:
      no-such-item: {status: 404, body: {error: error}}
      boom: {status: 500, body: {error: error}}
      boom-more: {status: 500, body: {error: error}}
      renamed: {status: 200, body: response}
```

`ess specify http validate --path model --binding binding.yaml`:

```
error[ESS-HTTP-025]: routes[0].outcomes.boom: outcome `boom` of command `demo.t.Rename` answers `500` with `{error: error}`, and so does `boom-more` (`{error: error}`), with a body a reader cannot tell apart; two outcomes on one status are told apart by two `envelope` bodies, by `{<member>: error}` on one member with distinct error wire names, or by two redirects, one always writing a parameter the other never writes or both writing one parameter from `error.wire` with distinct error wire names
```

The same binding with `alike: [[boom, boom-more]]` on the route:

```
error[ESS-HTTP-025]: routes[0].alike: `alike[0]` (`[boom, boom-more]`): `boom` and `boom-more` answer `500` with `{error: error}`, which writes each error's own fields beside the member, and their errors' fields differ in name, wire name, type or order, so the wire tells them apart
```

## Acceptance

- One rule decides whether a reader tells the two bodies apart, and both checks use it: the pair
  above validates either without `alike:` (the field sets differ, so the answers are distinct) or
  with it (the reader is not promised which of the two it got). The scoper states which and why.
- The binding that stays refused is refused with a message that does not contradict the other
  check, and names the form that validates.
- OpenAPI describes both bodies on the `500` response; conformance over the binding maps each
  body to its outcome, or to the set where the binding declares them alike.
- A pair with one wire name and identical fields keeps today's answers (refused without `alike:`,
  admitted with it); a binding without such a pair keeps its bytes.

## Evidence

Both refusals were rerun for this story on `ess 0.93.0` and are quoted verbatim. The pair came
from a downstream adopter's retrofit, which counts 11 open notes on it. The rules read are the
acceptance of `story:two-outcomes-answered-alike-may-share-a-status-and-body`.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).
