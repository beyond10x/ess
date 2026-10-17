---
format: aep.planning-md/3
id: story:view-declares-its-answer-for-an-unknown-identity
kind: story
status: draft
title: A view declares its answer for an identity no row carries
relations:
- depends_on: story:create-or-update-selects-update-branches-by-stored-fields
- depends_on: story:http-binding-response-body-and-status-per-outcome
revision: 1
---
## Outcome

A view read by identity declares its answer for an identity no row carries: the error it answers,
or a default row whose fields are literals; synthesis checks that answer with an identity nothing
stores.

Closes https://github.com/beyond10x/ess/issues/533.

## Fit review

1. **Need.** A per-record status read answers a default record for an identity no row carries
   (`200`, every flag false, only the identity set), where it used to answer an error. Minimal
   reproduction: entity `demo.status.Flag {id, paused: Boolean}`, view `FlagById` with parameter
   `id` and filter `id == param.id`; the model cannot say what a read of an unstored `id` answers,
   so a suite reads nothing for it and both behaviours pass. Requester's syntax: none; "the error
   it answers, or a default row whose fields are literals".
2. **Class: gap.** A view lists the rows its source holds
   (`crates/specify/ess-domain/src/view.rs:371`, filter over one source row and parameters); an
   empty answer is the only answer for a missing identity. Commands have `unknown_instance:`
   (precedence step 3, `docs/design/cross-record-and-stored-field-guards.md`); views have no
   counterpart.
3. **Already expressible?** Partly, at the HTTP layer only: ess-http/1 stage 2 designs `body:
   single` with `empty: {status: 404}` (`docs/design/http-presentation-binding.md:314,387`), a
   status for a view that holds no row. That covers "answers an error status" on HTTP only, and
   not a default row; the model and its suite still cannot check it. A read-only command with
   `unknown_instance:` would answer it for commands, which is
   `story:read-only-command-answers-an-unknown-instance` (#535), not views.
4. **Fit.** Design on a page: a view key used only on a view whose filter selects by the source
   entity's identity from its parameters (the condition HTTP stage 2 answer 7 recommends for
   `single`), `absent: {error: <Error>}` or `absent: {row: {<field>: <literal>…}}`, with the
   identity field filled from the parameter. It must reuse the command vocabulary
   (`unknown_instance`'s meaning; literal grammar of `sets:`), compose with view grants, paging
   (refused beside paging: one row), aggregates (refused) and the HTTP binding (`single` on such a
   view reads `absent` instead of declaring its own `empty`, or both must agree; settled with
   stage 2). Targets: interpreter and synthesis answer it; Rust, Go, TypeScript generated view
   readers answer it or refuse by name; OpenAPI documents the error or the default row;
   `ess verify diff` classifies a change.
5. **Second adopter.** A feature-flag service answers a flag nobody set as `{enabled: false}`; a
   profile service answers an unknown user `NotFound`.
6. **Cost.** A new view key: source format `ess/25`; a suite expectation for a view read that
   answers an error needs an `ess-conformance` format bump (a view read today answers rows only),
   settled on the page; one `ess-diff` line; OpenAPI change on views that declare it.
7. **Considered.** (a) change nothing: the default answer stays unchecked; (b) only the HTTP
   `empty:` (stage 2): covers the status on one presentation, not the model; (c) the view key
   (chosen pending the page). No requester syntax to take or refuse.

## Decisions

Accept, redesigned: the need is adopted; the key is designed on a page and settled together with
ess-http/1 stage 2's `single`/`empty` so a view has one answer for a missing identity.

## Acceptance

Each line names the failing test written first.

- Design section in `docs/design/read-api-view-idioms.md` § "An identity no row carries" (ess/25)
  chooses the key, where it is admitted, the suite format and the HTTP `empty:` relation.
- `crates/specify/ess-domain/tests/view_absent_answer.rs`
  `absent_default_row_validates_on_an_identity_selecting_view_at_ess_25`,
  `absent_is_refused_on_a_view_not_selecting_by_identity`, `absent_is_refused_below_ess_25`.
- `crates/verify/ess-conformance/tests/interpreted_views.rs`
  `interpreter_answers_the_declared_default_row_for_an_unstored_identity`.
- `crates/verify/ess-conformance/tests/view_consistency.rs`
  `synthesis_reads_an_unstored_identity_and_expects_the_declared_answer`: the suite fails a target
  that answers no row, or the default row with a flag set.

## Scope

- cited: `crates/specify/ess-domain/src/view.rs` (view reader and filter check),
  `docs/design/http-presentation-binding.md:257-258, 314, 387, 464`.
- inferred: `crates/specify/ess-compiler/src/ir.rs` (view IR),
  `crates/verify/ess-conformance/src/interpret/` (view execution),
  `crates/verify/ess-conformance/src/synthesize.rs` (view scenarios),
  `crates/generate/ess-openapi/`, `crates/verify/ess-diff/`, the generated view readers in
  `crates/generate/ess-synth/`.

## Plan

- depends_on: the unit that opens `ess/25`; ess-http/1 stage 2's response body unit (`single` and
  `empty`), so the two land on one rule.
- du estimate: 10G.
- adversary: yes (new answer across interpreter, synthesis and generated readers).
- shared surfaces: `view.rs` with no other wave m story; `ess-openapi` with ess-http/1 stage 2
  (wave k), which lands first.
