---
format: aep.planning-md/3
id: story:the-generated-react-and-terminal-apps-send-the-select-members-and-nested-members-a-bound-route-declares-or
kind: story
status: draft
title: The generated React and terminal apps send the select members and nested members a bound route declares, or refuse the binding by name
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:several-command-routes-may-share-one-method-and-path-each-chosen-by-literal-values-of-members-of-the-json-body
- depends_on: story:a-json-request-may-carry-an-input-field-in-a-nested-member-of-its-body-validated-compiled-and-projected
revision: 2
---
## Need

A generated app sends a command over its HTTP binding: `crates/ui/ess-ui/src/binding.rs` reads the
binding, `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl` and `crates/ui/ess-ui-tui/src/http.rs`
write the request (the bare-body form of 0.89.0 is pinned in `crates/ui/ess-ui-react/tests/bare_body.rs`
and `crates/ui/ess-ui-tui/tests/bare_body.rs`). After `ae1a` and `ae1d`, a route may require
constant members (`select`) and nested members (`request.members`). An app that sends the flat input
object would reach the catch-all or `malformed`, silently.

## Outcome

Both apps write, for a route with `select`, the first alternative's members as constants, and for
`request.members`, each field under its path. The written body is the one the server selects for that
command. A catch-all route's command is sent with no constant. Where an app cannot write a form
(none is expected), it refuses the binding at load by name, naming the route and the key. A silent
misread is not acceptable.

## Acceptance

1. Red today: a binding with `select: {kind: closed, data.reopen: true}` and
   `members: {ticket_id: data.ticket_id}` on `ReopenTicket` is loaded by each app (cases beside
   `bare_body.rs` in both crates). The request body the app writes for the command is asserted to be
   `{"kind": "closed", "data": {"reopen": true, "ticket_id": "t-1"}}`. This fails.
2. After: both apps write that body. A route without either key writes today's bytes (existing
   `bare_body.rs` cases unmodified).
3. `crates/ui/ess-ui-check` (`src/model.rs`) accepts such a binding. A route whose command the
   document invokes and whose select the app cannot write is reported by name (no case expected, the
   check exists for later keys).

## Scope

- cited: `crates/ui/ess-ui/src/binding.rs`, `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`,
  `crates/ui/ess-ui-tui/src/http.rs`, `crates/ui/ess-ui-check/src/model.rs`
- cited: `crates/ui/ess-ui-react/tests/bare_body.rs`, `crates/ui/ess-ui-tui/tests/bare_body.rs`

## Owned files

The six files above. No other UI file.

## Not yours

Server dispatch (`ae1c`, `ae1e`), validation (`ae1a`, `ae1d`), OpenAPI (`ae1b`).

## Tests first

Acceptance 1 in both crates (string comparison of the written body, no browser, milliseconds).

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-ui -p ess-ui-react -p ess-ui-tui -p ess-ui-check --all-targets --locked -- -D warnings
cargo test -p ess-ui-react --locked --test bare_body
cargo test -p ess-ui-tui --locked --test bare_body
cargo test -p ess-ui-check --locked --test binding
```

## Depends on

`ae1a-several-operations-share-one-route-by-body-members`, `ae1d-a-json-request-reads-an-input-from-a-nested-member`.
Beside `story:a-generated-ui-reads-the-answers-its-http-binding-declares` (same `binding.rs`;
whichever lands second rebases).

## Changelog

- The generated React and terminal apps send the selecting members and nested members a bound
  route declares.

## Fit review

As `ae1a` and `ae1d`. Question 4, targets: every reader of `ess-http-ir/1` handles the keys or
refuses them by name. This unit closes the app readers.

## Decisions

Accept, redesigned. No open question.

- D134: until this unit lands, every generator that does not read `request.members` or `select`
  refuses a binding that uses them, by name; this unit lifts its refusal. Taken by the
  coordinator on 2026-11-25.

## Size

S, about 250 lines with tests.
