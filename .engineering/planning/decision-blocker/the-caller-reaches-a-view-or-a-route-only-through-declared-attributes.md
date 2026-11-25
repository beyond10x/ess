---
format: aep.planning-md/3
id: decision-blocker:the-caller-reaches-a-view-or-a-route-only-through-declared-attributes
kind: decision-blocker
status: open
title: 'How authority from a credential reaches a view or a route: the caller as a declared attribute only, the row that describes the caller, a caller without a port (HC-D1 to HC-D3)'
summary: Security-marked decisions on a parameter or row taken from the caller; the owner decides, the fail-closed option is A in each
tags:
- gap-reports-2026-11-24
- security
relations:
- blocks: story:a-view-may-answer-the-row-that-describes-the-reading-actor
- blocks: story:a-view-selects-rows-by-a-caller-attribute
revision: 1
---
HC-D1 to HC-D3 are the owner's. They are security-marked: each answers how authority that comes from a credential reaches the model. A downstream application asks (twentieth gap report, shape 1) for a way to bind a view or command parameter, or a row, from the caller: a tenant or account that the real service takes from the bearer token. The fail-closed option is named in each decision; none is taken without the owner.

## Why this is open

`ess-http/1` binds a model parameter only from the path, the query or the body, so a parameter such as `account_id` that the real service ignores on the wire becomes a query parameter a caller can set to any value. The model already has a caller (actor `attributes:` since `ess/16`, 0.38.0; `docs/design/caller-values.md:43-44`): a command reads it as a `sets:` or `payload:` source `{caller: <attribute>}` and in a `when:` or `when_subject:` guard with `==` or `!=`. A view filter cannot read it (`ESS-VIEW-003`, "`caller.desk` reads `caller`, which is not a declared observable root", `crates/specify/ess-domain/src/expression.rs:990`; `story:a-view-selects-rows-by-a-caller-attribute`, draft). The caller reaches a generated server through the authentication port the binding names, never through a request header or a parameter mapping (`architecture-decision-record:a-binding-authenticates-the-caller-through-a-port`, accepted; `story:an-http-binding-declares-how-a-caller-authenticates`, active; AC-D3, open, default A: a credential header is never an input).

The report has two cases of one class: (a) a view or command parameter the real service takes from the token; (b) a "who am I" view over an entity describing the caller that no command writes, so any executor answers it empty (`story:a-view-may-answer-the-row-that-describes-the-reading-actor`). No option below makes a credential, a header or a token claim an ordinary input.

## HC-D1: how a parameter or a selection that comes from the caller is stated

- A. Through declared actor attributes only. A view filter reads `caller.<attribute>` (`story:a-view-selects-rows-by-a-caller-attribute`), a command reads `{caller: <attribute>}` and the guards that exist (released since 0.38.0, `caller-values.md:29`, `:44`), and the HTTP binding carries no source for it: the route has no parameter for the attribute, OpenAPI lists none and names the attribute under `x-ess-caller` (`caller-values.md:86`). The attribute arrives through the authentication port. Cost: a retrofit models the account as an attribute and drops the parameter; a model that kept the parameter keeps a free input.
- B. Option A plus a binding key that maps a path or query parameter to a caller attribute (`parameters: {account_id: caller.account_id}`). Cost: the binding then states caller semantics the model does not, a suite can set the parameter to any value while the route fixes it, a new source namespace in `ess-http/1` and `ess-http-ir/1`, and a rule for which attribute types a segment can carry.
- C. Decline: a retrofit marks the parameter unmapped and an adapter on the consumer side supplies it.

Fail-closed: A. The caller stays a declared attribute that a credential is turned into at the authentication port, and nothing is added to the binding. Needs the owner.

## HC-D2: the row that describes the caller (case b)

- A. No new construct: a command the model declares writes the caller's row from `{caller: <attribute>}` sources (a registration step the system actor issues), and a view of it filters by `caller.<attribute>` once HC-D1 A is built. The row exists only after that command ran, as every row does. Cost: the specification states how the row comes to exist, which a retrofit of a service that never stores it has to invent.
- B. A view may take the reading actor itself as its source: one row built from the actor's declared attributes, no entity and no command. Cost: a new source kind in the view grammar (a new `ess/N`), synthesis and the interpreter read two callers and expect two rows, generated servers answer it from the caller context, and the conformance runners need a caller on every read of it.
- C. An entity row the authentication port fills on first sight of a caller. Cost: the port writes model data, which the accepted ADR does not give it.

Fail-closed: A. Needs the owner.

## HC-D3: where an executor with no authentication port gets its caller

The reporter's executors have no port and want "a configured fixed caller".

- A. The document states no caller. A conformance run names the caller on each step as it does today (`caller-values.md:98`, `AbsentInputRequest::caller`), and an executor takes its caller from its own configuration.
- B. The binding or the specification states a fixed development caller. Cost: a document that holds an identity the executor will trust; refused as a credential in a specification.
- C. The interpreted target takes the caller from an option on its command line (a runtime flag outside every document), and nothing else changes. Cost: a flag, and a second way to name a caller beside the per-step one.

Fail-closed: A. Needs the owner.
