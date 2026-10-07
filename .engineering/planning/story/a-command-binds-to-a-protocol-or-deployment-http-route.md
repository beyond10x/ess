---
format: aep.planning-md/3
id: story:a-command-binds-to-a-protocol-or-deployment-http-route
kind: story
status: draft
title: A command or view binds to the HTTP method and path a protocol fixes or a deployment chooses
refs:
- provider: github
  reference: beyond10x/ess#493
relations:
- serves: vision:O2
revision: 1
---
## Request

https://github.com/beyond10x/ess/issues/493 (2026-10-07): a specification of a published HTTP
protocol has operations whose method and path are not ESS's to choose.

- Fixed by a standard: a discovery document served at one well-known path.
- Chosen by each deployment: a protocol that leaves an endpoint's location to the server, so two
  implementations of one specification serve the same command at different paths.

ESS 0.55.0 derives every route (`POST /{domain wire}/commands/{command wire}`,
`GET /{domain wire}/views/{view wire}`, `crates/generate/ess-gen/src/http.rs`) and refuses any
routing key on a command (`unknown field`). An `ess-realization/2` entrypoint carries one base URL,
not a route per command. An external implementation's conformance adapter holds the
command-to-URL mapping in hand-written code that nothing checks against the model.

The requester's proposed syntax, labelled as theirs: `http: {method: POST, path: /token}` beside
`naming:` on the command.

## Fit review

Not written yet. Written with `.agents/skills/assessing-external-requests/SKILL.md` before this
story is dispatched. Questions it has to answer, beyond the seven:

- where a protocol-fixed route lives (the specification) and where a deployment-chosen route lives
  (the realization or a deployment document), and whether one construct covers both;
- whether the sibling `ess-cli/1` presentation binding is the shape to follow for an HTTP binding;
- views as well as commands (a fixed `GET` discovery document is a view);
- what the OpenAPI projection, the conformance runner and every generated target do with a declared
  route, or which of them refuse it by name.

## Acceptance (draft, settled by the fit review)

- A specification can declare the HTTP method and path of a command or view that a protocol fixes,
  and a deployment can declare the path of one the protocol leaves open; `ess specify validate`
  accepts both and refuses a route two operations share.
- `ess generate --kind openapi` emits the declared method and path instead of the derived route.
- The conformance runner reaches an implementation at the declared route, so the mapping is checked
  against the model rather than held in adapter code.
- Every target that cannot honour a declared route refuses it by name.
