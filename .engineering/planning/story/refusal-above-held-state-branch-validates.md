---
format: aep.planning-md/3
id: story:refusal-above-held-state-branch-validates
kind: story
status: implemented
title: An external branch can be declared to answer before held-state selection
tags:
- adopter-report
- design-first
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:create-or-update-selects-update-branches-by-stored-fields
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T07:15:03Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-16T07:15:03Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-16T07:15:17Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A command can declare that an `external:` refusal answers before existence, held-state and
related-row selection, at the step input-guarded refusals take, so a model can require a check the
implementation makes before it reads the row (a credential, a lookup elsewhere) to answer first,
and synthesis injects that external cause first.

Design first: a section under `docs/design/` before code.

Closes https://github.com/beyond10x/ess/issues/534.

## Evidence

Two adopters, the same need.

1. https://github.com/beyond10x/ess/issues/534 (ess 0.59.0, ess/24): `demo.desk.SignIn` checks a
   credential (`external: secret is not the digest the service derives from user_id`) before it
   reads the seat, then selects `reopened-busy`, `reopened`, `opened` by the held state, with an
   `unknown_instance:` creating branch. Validation refuses each held-state branch:

   ```
   [conflicting_declaration] command.demo.desk.SignIn.outcomes.reopened-busy: `reopened-busy` is selected by the held state, which answers before the external branch `bad-credential` declared above it
   ```

   Declaring the external branch after them makes it unreachable for every stored row.

2. A revocation command on ess 0.56.0, input `{token, client_id}`, entity
   `Token {client_id: String, state Active|Revoked}`:

   ```yaml
   outcomes:
     - name: invalid-client
       external: client authentication failed
       error: demo.revoke.InvalidClient
     - name: invalid-token
       unknown_instance: true
       refuses: false
     - name: issued-to-another-client
       when_subject: {predicate: client_id != input.client_id}
       error: demo.revoke.TokenNotIssuedToClient
     - name: already-revoked
       when_subject_state: Revoked
     - name: revoked
       moves: demo.revoke.Token.revoke
   ```

   ESS-COMMAND-004 refuses `issued-to-another-client` and `already-revoked` the same way. RFC 7009
   § 2.1 and RFC 6749 §§ 3.2.1, 4.1.3 authenticate the client before they look at the token; with
   held state first, a server answering "not issued to you" to an unauthenticated caller conforms,
   and that answer says whether a token exists.

## Fit review

1. **Need.** An external refusal the implementation decides from the request alone, before it
   loads the addressed row, must answer before existence (step 3) and the held state (step 4) of
   `docs/design/cross-record-and-stored-field-guards.md` "The precedence order"; today every
   external branch is step 6. Requesters' syntax (theirs): `precedes: held_state` on the branch, or
   an ordered `precedence:` list.
2. **Class: gap.** The order is normative (one order for every command, same page); the #486 rule
   (`crates/specify/ess-domain/src/command.rs:3431`, pinned by
   `crates/specify/ess-domain/tests/held_state_order.rs:183-203`) refuses the only declaration
   that would say it.
3. **Already expressible?** No. An input guard cannot compute a digest or a lookup; `external:` is
   the construct for an uncomputable cause and it is fixed at step 6.
4. **Fit.** Preferred design: a key on the external branch that places it at step 2, among the
   input-guarded refusals in declaration order, for example `external: {cause: <text>, answers:
   before_row}` or a sibling key `before_row: true`; spelled once on the page. It must state: how it
   composes with the #486 rule (a step-2 external is declared before every step-3..6 branch, so the
   rule's refusal does not apply to it, and declaring it after them is refused), with
   `unknown_instance:` (answers before it, so an unauthenticated caller learns nothing about
   existence), with `when_related:` step 1 (the page decides whether step 1 or the external comes
   first; recommendation: the external, since it reads no row), with an accepting external branch
   (refused: only a refusal reads no row), and with `story:external-branch-narrowed-by-subject-guard`
   (a narrowed external reads the row, so it can never be `before_row`; the two keys are refused
   together). An ordered `precedence:` list is refused as a design: it reopens the one-order rule
   for every step. Every target: interpreter (`interpret::execute`), synthesis
   (`synthesize::sibling_refusals`, `synthesize.rs:6789`), Entity Runtime lowering (input refusals
   decided before any row load, `crates/generate/ess-entity-runtime/tests/refusal_beside_state.rs`),
   Rust, Go and TypeScript generated targets, `ess verify diff`, generated docs.
5. **Second adopter.** Already two (above).
6. **Cost.** One authored key: source format `ess/25`; one `ess-diff` classification; no new
   diagnostic code (refusals reuse `conflicting_declaration`).
7. **Considered.** (a) change nothing: the model cannot state authentication first; (b) ordered
   `precedence:` list: general, but breaks the single order; (c) `precedes: held_state`: names one
   step only, leaving existence before the external; (d) step-2 placement (chosen).

## Decisions

Accept, redesigned: the external refusal is placed at step 2 by a key on the branch (spelled on
the design page); the requesters' `precedes:` and `precedence:` forms are not taken, because step 2
answers existence too and keeps one order.

## Acceptance

Each line names the failing test written first.

- Design section in `docs/design/cross-record-and-stored-field-guards.md` "The precedence order"
  names the key and its composition with #486, `unknown_instance:`, `when_related:` and the
  narrowed external.
- `crates/specify/ess-domain/tests/held_state_order.rs`
  `external_refusal_before_row_validates_above_held_state_branches_at_ess_25` (the #534 model and
  the revocation model validate) and
  `external_refusal_without_the_key_keeps_the_486_refusal` (unchanged message).
- same file, `before_row_on_an_accepting_external_or_below_ess_25_is_refused`.
- `crates/verify/ess-conformance/tests/external_beside_held_guard.rs`
  `before_row_external_is_witnessed_on_a_row_held_state_would_also_select`: synthesis writes the
  `bad-credential` scenario on a `Busy` seat and on an identity nothing stores, and the
  interpreted, Rust, Go and TypeScript targets all answer `bad-credential` there.
- `crates/generate/ess-entity-runtime/tests/refusal_beside_state.rs`
  `before_row_external_lowers_before_the_row_load` or a refusal by name.

## Scope

- cited: `crates/specify/ess-domain/src/command.rs:3400-3440` (#486 rule),
  `crates/verify/ess-conformance/src/synthesize.rs:6789` (`sibling_refusals`),
  `docs/design/cross-record-and-stored-field-guards.md`.
- inferred: `crates/verify/ess-conformance/src/interpret/execute.rs` (`refused_by_input`),
  `crates/specify/ess-domain/src/wire.rs` and `schemas/generated/ess.schema.json`
  (`RawSpecFile`, `cargo xtask schema`), `crates/specify/ess-compiler/src/ir.rs`,
  `crates/generate/ess-synth/src/{rust,go}/`, `crates/verify/ess-diff/`.

## Plan

- depends_on: the unit that opens `ess/25`.
- du estimate: 10G.
- adversary: yes (changes the precedence order).
- shared surfaces: `command.rs` and `interpret/execute.rs` with
  `story:create-or-update-selects-update-branches-by-stored-fields`; `wire.rs`/`RawSpecFile` with
  every ess/25 key in the wave. `story:external-branch-narrowed-by-subject-guard` stays draft and
  out of this wave; its design must name this key.
