---
format: aep.planning-md/3
id: decision-blocker:http-surface-scope
kind: decision-blocker
status: cleared
title: Should ESS describe an HTTP surface whose wire shape it does not choose?
relations:
- blocks: story:a-command-binds-to-a-protocol-or-deployment-http-route
- serves: vision:O2
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-08T02:40:39Z", actor: "human:timo", revision: 3}
---
## Question

Should ESS describe an HTTP surface whose wire shape it does not choose (https://github.com/beyond10x/ess/issues/493)?

| option | what it does | cost |
|---|---|---|
| (a) route only | an authored binding moves the method and path of a command or view; bodies, statuses and encoding stay ESS's own | about 6 units; a protocol such as the issue's still projects wrong bodies and statuses, and gains no conformance |
| (b) presentation binding, staged | one digest-pinned `ess-http/1` binding beside the specification, like `ess-transport`: the route first; request encoding, response body, status per outcome and redirects next; an HTTP conformance target last; one design page covers the whole format first | about 15 to 17 units over three stages; brings back HTTP-shape scope that was excluded on 2026-09-15 |
| (c) not now | answer the requester with the conformance adapter as the idiom | 0 units |

The fit review on `story:a-command-binds-to-a-protocol-or-deployment-http-route` recommends (b).
Whatever the answer, the requester's `http:` key on a command is refused, and the wire-name defect
it found is fixed separately (`story:a-wire-name-in-a-path-segment-refuses-slash-and-dot-segments`).

## Decided (2026-10-08)

**(b), the staged `ess-http/1` presentation binding.** ESS describes an HTTP surface whose wire shape it does not choose: method, path, bodies and statuses fixed by a protocol (RFC 8414, OIDC Discovery) or by a deployment. Scope is 15 to 17 units in three stages, planned after 0.57.0 ships, with one design page for the whole format first. The requester's input is in scope: HTTP status codes and response headers, including `WWW-Authenticate` with its `error` attribute. https://github.com/beyond10x/ess/issues/493 stays open until stage 2 (request encoding, response bodies and statuses) ships.
