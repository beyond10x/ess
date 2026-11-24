---
format: aep.planning-md/3
id: story:two-crossed-route-templates-may-be-served-in-an-order-the-binding-declares
kind: story
status: archived
title: Two crossed route templates may be served in an order the binding declares
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
transitions:
- {from: "draft", to: "archived", at: "2026-11-24T01:46:16Z", actor: "human:timo", revision: 2}
---
## Need

Two routes of one component and one method whose templates cross (each holds a literal where the
other holds a variable, so one request path matches both) are refused:

- `ESS-HTTP-011`, `crates/specify/ess-http/src/lib.rs:7534-7556`: "`<a>` and `<b>` cross on component
  `<c>`: both match `<path>`, and neither is more specific …". The clash is decided on the template
  text alone (`clash`, `lib.rs:7404-7425`), never on the variables' types.
- Deliberate: `docs/design/http-presentation-binding.md` § "Path templates" (crossed templates "stay
  refused" because "any order between them would decide silently which identifier values one route
  cannot reach") and § "Decided for stage 1", answer 3, superseded in 0.89.0 for nested templates
  only.

A downstream application had two event-ingest routes that cross, and left both on ESS's derived
paths. Its specification then states paths the service does not serve (third report, finding 4).

Minimal reproduction (written fresh; not run): commands `demo.ingest.IngestForTenant(tenant,
body)` and `demo.ingest.IngestFromSource(source, body)` on one network component.

```yaml
routes:
  - {command: demo.ingest.IngestForTenant, method: post, path: "/{tenant}/events", parameters: {tenant: input.tenant}}
  - {command: demo.ingest.IngestFromSource, method: post, path: "/events/{source}", parameters: {source: input.source}}
```

```console
$ ess specify http validate --path spec --binding ingest.http.yaml
error[ESS-HTTP-011]: routes[1]: … cross on component `ingest`: both match `/events/events`, and neither is more specific …
```

## Outcome

Under **AC-D6**, option A (the default), a route may declare `precedes: [<operation>, …]`, naming the
operations of routes it crosses. For a request path both match, the route that precedes answers.
The order is the binding's own statement, not a silent choice, so `ESS-HTTP-011` no longer refuses a
crossed pair whose order is declared. It still refuses an undeclared one, and its message names
`precedes` as the way out. `ess_gen::http::dispatch_order` honours the declared order, so both
generated servers try the preceding route first. OpenAPI, which has no route order, records it as
`x-ess-precedes` on the path item, with a description sentence naming the paths the other route
cannot reach.

## Acceptance

- The reproduction with `precedes: [demo.ingest.IngestFromSource]` on the first route validates and
  compiles. `ess-http-ir/1` writes `precedes` only where declared, so every existing binding
  compiles to the same bytes. Cases go in `crates/specify/ess-http/tests/compile.rs`, starting from
  the refused document above.
- Refused, `ESS-HTTP-011` widened, each naming the key: `precedes` naming an operation the binding
  does not route, one on another method or component, one the route does not cross (nested or
  disjoint: the order is already decided or not needed), the route itself, or a cycle; both routes of
  a pair declaring `precedes` over each other.
- The generated Rust and Go servers send `POST /events/events` to the preceding route and every
  other path to the route that alone matches it (`http_binding_rust`, both alike). Every existing
  byte pin passes unmodified, because nested pairs keep their computed order.
- `ess generate --kind openapi --http` writes `x-ess-precedes` and the description sentence. A
  binding without `precedes` keeps its bytes.
- § "Path templates" and answer 3 state the declared order and why it is not the silent order the
  rule refused.

## Fit review

1. **Need.** A real router's order between two crossed routes cannot be stated, so the specification
   moves two real routes onto paths the service does not serve. Report: third report, finding 4
   ("`ESS-HTTP-011` forced two event-ingest routes onto ESS-chosen paths").
2. **Class: gap under a deliberate rule** (§ "Path templates", stage-1 answer 3).
3. **Already expressible?** No. Refining the templates so they nest changes the paths.
4. **Fit.** The refusal's reason is that the order would be silent. A declared order answers that
   reason without guessing (`docs/design/linker-never-chooses.md` holds: the author chooses).
   Dispatch already has one ordering function both servers use. Siblings: `ess-cli/2` has no paths,
   and the RPC drafts route by method name.
5. **Second adopter.** A multi-tenant API with `/{tenant}/status` beside `/admin/{tool}`, where
   `/admin/status` belongs to the admin route.
6. **Cost.** One optional key in `ess-http/1` and `ess-http-ir/1`, so no version. OpenAPI cannot say
   order, so a third-party router built from the projected document may route the shared paths the
   other way. That cost is written into the document. No new code.
7. **Considered.** See AC-D6. Option C, admitting a crossed pair whose variable's declared type
   cannot hold the other's literal (an enum without that member, an `Integer` beside a non-numeric
   literal), needs no key and is the right rule where it applies. It does not reach two free-text
   variables, which event-ingest routes likely are (not established).

## Decisions

**AC-D6**, default A. Option C can be added beside A later, as an addition that refuses nothing.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`Clash` 7385-7400, `clash` 7404, `surface` 7470-7560)
- inferred: `crates/generate/ess-gen/src/http.rs` (`dispatch_order`)
- cited: `crates/generate/ess-gen/src/openapi.rs`
- cited: `docs/design/http-presentation-binding.md`
- Size: M, about 400 lines with tests.

## Depends on

Nothing.

## Source

A downstream application, third report, finding 4 (ESS 0.98.0).
