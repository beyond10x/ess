---
format: aep.planning-md/3
id: story:a-view-may-answer-the-row-that-describes-the-reading-actor
kind: story
status: draft
title: A view may answer the row that describes the reading actor, so a "who am I" read is answered from the caller and not empty
summary: 'Security-marked: a view over an entity no command writes answers empty on every executor; the caller reaches a view through declared attributes (HC-D1 to HC-D3, with the owner)'
tags:
- external-request
- gap-reports-2026-11-24
- security
relations:
- serves: vision:O2
- informed_by: story:a-view-selects-rows-by-a-caller-attribute
- informed_by: story:an-actor-expression-reads-the-fields-of-the-session-view
revision: 1
---
## Summary

Security-marked: authority from a credential reaches the model here, so the decisions are the owner's
(`decision-blocker:the-caller-reaches-a-view-or-a-route-only-through-declared-attributes`, HC-D1 to HC-D3,
fail-closed A). A real service answers "who am I" from the authenticated session: one object describing
the caller, read from no stored row. A downstream application modelled it as a view over an entity
`CallerUser` that no command writes; any executor of the specification answers it empty. The story
records what the model can say today, what it cannot, and what a read of the caller needs.

## Facts (tag 0.99.1, read)

1. The model says who the caller is through an actor's declared `attributes:` (`ess/16`, 0.38.0). They
   are readable by a command as `{caller: <attribute>}` in `sets:` and `payload:` and in `when:` and
   `when_subject:` guards (`docs/design/caller-values.md:29`, `:43-44`); an `instance:` that names a
   caller attribute is `story:a-command-acts-on-the-row-a-caller-attribute-names` (active).
2. A view cannot read the caller: `caller.<attribute>` in a view filter is `ESS-VIEW-003`
   (`crates/specify/ess-domain/src/expression.rs:990`; `story:a-view-selects-rows-by-a-caller-attribute`).
   A view has a source entity and every field comes from it (`ViewSpec.source` is required,
   `decision-blocker:a-view-without-a-source`, cleared: a view may answer values no row holds, which
   reads settings and constants, not the caller).
3. So the "who am I" row is a row. Nothing in the model says how it comes to exist: an entity no command
   creates holds no rows, and the view over it answers an empty list on every target
   (`crates/specify/ess-http/src/lib.rs` `rows`; the reporter's observation).
4. The answer is one bare object, not a list: `body: single` needs the filter to select the source
   entity by identity against `param.<name>` (`selects_identity`, `lib.rs:8807-8824`), so a view that
   selects by `caller.<attribute>` is `ESS-HTTP-031` even after the caller story lands, unless the same
   widening as the literal case is made (`story:a-single-view-may-select-its-row-by-a-literal-identity-and-a-literal-route-serves-it`,
   HB-D2).
5. The UI side states the same need from the other end: a UI reads the actor from "the first row of the
   first view the home page's shell preloads" and checks no `actor.` path
   (`story:an-actor-expression-reads-the-fields-of-the-session-view`, draft).

## Today, by modelling

The row can be written by a command the model declares: a command the system actor issues (the
authentication port's registration step) with outcome `created` that sets the row's fields from
`{caller: <attribute>}` and names the identity in an event payload `{caller: <attribute>}`
(`caller-values.md:29`). The view then selects it by a parameter, which any caller could set to another
user's identifier: that is the shape the reporter wanted to leave. Without `story:a-view-selects-rows-by-a-caller-attribute`
the "my own row" read cannot be written without a free parameter. Read, not run: the creating outcome
reads its identity from the event it emits (`ESS-COMMAND-001`), which is
`decision-blocker:a-create-names-its-identity-without-a-published-event` for a component that publishes
nothing.

## Acceptance (provisional, after HC-D1 and HC-D2)

1. Under HC-D1 A and HC-D2 A: a view over an entity of the caller (`CallerUser`) whose filter is
   `user_id == caller.user_id`, with no parameter, validates for a system whose every actor that may read
   it declares `user_id`; `ess-http/1` routes it with no mapping; with `selects_identity` widened it is
   `body: single`.
2. Synthesis registers a caller through the declared registration command, reads the view as that
   caller and as a second caller, and expects each to see only its own row; a target that answers the
   other caller's row fails; a read with no caller is refused by name.
3. Under HC-D2 B instead: the view names the actor as its source; no entity and no registration command;
   the same two-caller read.
4. A model without such a view keeps its IR bytes and its suite bytes.
5. Each test runs within 60 s and each test step within 120 s.

## Fit review

1. Need: a read of the caller's own description that every executor answers from the caller, not empty.
2. Class: a gap, security-marked; the owner's under HC-D1 to HC-D3.
3. Covers: shape 1b of the twentieth gap report. Shape 1a is
   `story:a-view-selects-rows-by-a-caller-attribute`, amended.
4. Borders: `story:a-view-may-scope-rows-to-the-reading-actor` (draft, an older statement of 1a),
   `story:an-actor-expression-reads-the-fields-of-the-session-view` (the UI reading of the same row),
   `story:a-command-acts-on-the-row-a-caller-attribute-names` (active, command side),
   `decision-blocker:a-create-names-its-identity-without-a-published-event` (EV-D1, the creating outcome of
   the registration command), the AC-D3 rule that a credential header is never an input (not reopened
   here). A search of the store and the open issues found no story on a "who am I" view.
5. A second adopter: any service with a `/me` or `/session` endpoint.

## Decisions

HC-D1, HC-D2 and HC-D3 are on `decision-blocker:the-caller-reaches-a-view-or-a-route-only-through-declared-attributes`,
with the owner; fail-closed A in each.

## Scope

- `crates/specify/ess-domain/src/view.rs`, `src/expression.rs` (a caller operand in a view filter)
- `crates/specify/ess-http/src/lib.rs` (`selects_identity`)
- `crates/verify/ess-conformance/src/synthesize.rs` (two-caller reads of a view)
- `docs/design/caller-values.md`
