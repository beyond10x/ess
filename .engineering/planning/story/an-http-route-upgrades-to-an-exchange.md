---
format: aep.planning-md/3
id: story:an-http-route-upgrades-to-an-exchange
kind: story
status: draft
title: An HTTP route upgrades to an exchange
summary: 'ess-http upgrade: route key carries the handshake and the 101; size M'
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- depends_on: story:an-exchange-is-declared-in-the-specification
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

A `GET` whose accepting answer is `101 Switching Protocols`, after which the socket carries an
exchange, cannot be bound. `ess-http/1` refuses `101` as an outcome status, rightly: it is an
interim answer, not the outcome's answer (`docs/design/http-presentation-binding.md:405`). What is
missing is a route key saying the route opens an exchange (ADR
`messages-are-exchanged-independent-of-the-wire` § 2, binding C; F3 (a): this route carries the
handshake, `ess-websocket/1` the framing).

Minimal reproduction on 0.92.0 (a `Connect` command bound to `GET /calls`):

```text
$ ess specify http validate --path model --binding http-101.yaml
error[ESS-HTTP-023]: routes[0].outcomes.connected.status: outcome `connected` answers `101` is a `1xx` status, an interim answer no outcome gives
exit 1

$ ess specify http validate --path model --binding http-upgrade.yaml
error: reading http-upgrade.yaml as HTTP binding YAML: routes[0]: unknown field `upgrade`, expected one of `command`, `view`, `operation_id`, `method`, `path`, `parameters`, `request`, `outcomes`, `errors`, `alike`, `body`, `empty`, `refusals` at line 11 column 5
exit 1
```

## Acceptance

- Over a specification declaring `example.calls.Session` opened by `example.calls.Connect`, a route
  `{command: example.calls.Connect, method: get, path: /calls, request: {encoding: query},
  upgrade: {exchange: example.calls.Session, protocol: websocket}}` passes
  `ess specify http validate` (exit 0), and `ess specify http compile` writes `Route.upgrade` into
  `ess-http-ir/1`; the opening outcome's answer is `101` through the key.
- `101` as an outcome `status:` stays refused as `ESS-HTTP-023`.
- Refused by name, each with a new `ESS-HTTP-0NN` code at `routes[i].upgrade`: an exchange the
  specification does not declare; a route whose command is not that exchange's `opens`; a view
  route; a method other than `get`; a `protocol` outside the closed set `{websocket}`; a `status:`
  on the opening outcome beside `upgrade:`; two routes upgrading to one exchange.
- A binding without `upgrade:` compiles to the same IR bytes as before (existing `compile.rs` byte
  tests unchanged); tests in `crates/specify/ess-http/tests/upgrade.rs` (new).
- `docs/design/http-presentation-binding.md` gains a section "Upgrade to an exchange" stating the
  key, its refusals and that it is additive in `ess-http/1`.

## Scope

- cited: crates/specify/ess-http/src/lib.rs (`RouteSpec` 1544, `Route` 2087, status refusal 1499-1508, diagnostic table 3095)
- cited: crates/specify/ess-http/src/status.rs
- inferred: crates/specify/ess-http/tests/upgrade.rs (new)
- cited: docs/design/http-presentation-binding.md
- inferred: crates/edge/ess-cli/tests/http.rs (one CLI case)
confidence: high

## Fit review

1. **Need.** Bind the handshake of a socket that then carries an exchange: method, path, the
   refusals before the upgrade, and that acceptance is `101`. Reproduction above
   (`triage/650/repro/run.txt`). Requester's proposal: an outcome `status: 101` (issue body), which
   the issue itself agrees is wrong.
2. **Class: gap.** No `ess-http/1` key states an upgrade (`RouteSpec`,
   `crates/specify/ess-http/src/lib.rs:1544`, and the unknown-field error above).
3. **Already expressible?** No. Binding the opening command to a `201` misstates the wire, so a
   generated server and the OpenAPI document would both be wrong.
4. **Fit.** Not a grid R/O cell (`docs/design/language-grids.md`); a binding key. It reuses the
   route's existing `method`, `path`, `parameters` and refusal keys; only the accepting answer
   changes. Siblings: `ess-websocket/1` frames the messages (F3 (a), ADR line 166); no other
   binding opens an HTTP upgrade. Targets: OpenAPI in `story:asyncapi-and-openapi-project-an-exchange`;
   generated Rust and Go HTTP servers refuse a route with `upgrade:` by name until a story builds
   them; the HTTP conformance target (stage 3, `http-presentation-binding.md:759`) skips such a
   route by name.
5. **Second adopter.** A collaborative editor: `GET /documents/{id}/session` upgrades to an exchange
   following the `Document` row.
6. **Cost.** One additive key in `ess-http/1` and `ess-http-ir/1`, as `operation_id` and `servers`
   were added (`docs/design/http-presentation-binding.md:712-716`, stage 2 rule at 549-552): no
   format bump, an older reader refuses the IR field by name. About seven codes. No migration.
7. **Alternatives.** (a) Change nothing: no socket is bindable. (b) Admit `101` as an outcome
   status: makes an interim answer final and says nothing about what follows; refused. (c) `ess-http/2`:
   not needed, the additive rule covers it. (d) Chosen: `upgrade:` naming the exchange, ADR binding C.

## Decisions

**accept, redesigned.** The requester's `status: 101` is refused; the handshake is the route key
`upgrade: {exchange, protocol: websocket}` of ADR `messages-are-exchanged-independent-of-the-wire`
(accepted 2026-10-28), additive within `ess-http/1` by the rule the stage-2 keys and `operation_id`
used. The dependency on the WebSocket binding in the earlier body is reversed: this key needs only
the exchange, and the WebSocket binding reads this route to decide whether it needs a path.

## Source

github:beyond10x/ess#650
