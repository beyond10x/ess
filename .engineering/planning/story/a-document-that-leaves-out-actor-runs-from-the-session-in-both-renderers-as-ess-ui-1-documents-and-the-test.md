---
format: aep.planning-md/3
id: story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test
kind: story
status: draft
title: A document that leaves out actor runs from the session in both renderers, as ess-ui/1 documents, and the test documents that relied on an anonymous run say so
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
revision: 1
---
## Need

`Document.actor` defaults to `from_session` (`schemas/ui/ess-ui.schema.yaml:166`,
`website/docs/reference/ess-ui.md:154`). Both renderers run a document that leaves it out as
anonymous:

- React: `crates/ui/ess-ui-react/src/emit.rs:3322-3325` maps an absent value to `"anonymous"`, so
  `useActor` never reads the session view (`templates/runtime/data.ts.tmpl:940-950`).
- The terminal: `crates/ui/ess-ui-tui/src/app.rs:4660-4662` sets `actor.signed_in` only for
  `Some(FromSession)`, so an absent value runs signed out.
- `ess-ui-check` already treats an absent value as `from_session` for its grant checks
  (`crates/ui/ess-ui-check/src/model.rs:442`), so the checker and the renderers disagree.

Of the committed `ess-ui/1` documents, only `examples/partner-portal/ui.yaml` writes `actor:`. None
of the 88 documents under `crates/ui/**/tests/` does. Filling the default therefore changes the run
of nearly every test document, which is why this row of the defaults table
(`g03-loader-fills-defaults`) is a unit of its own.

## Outcome

- The fill step writes `actor: from_session` where the document leaves it out (P0-D17 A).
- Both renderers run such a document as `from_session`, by the rule each already has for an
  explicit `from_session`:
  - React reads the session view.
  - The terminal, in fixture mode, signs in as the first row of the first view its shell
    preloads (`app.rs:4655-4657`).
- Each test document whose test relied on an anonymous run writes `actor: anonymous`, so its test
  keeps its meaning. No test changes what it asserts.

## Acceptance

New cases in `crates/ui/ess-ui-tui/tests/page_actor.rs` and `crates/ui/ess-ui-react/tests/` (an
`actor_default.rs`). Each test is expected under 5 s, or 10 s where it runs the generated runtime,
and is timed. None may run over 60 s.

1. `an_absent_actor_runs_from_the_session`. Red today. A document with no `actor:` and a guard
   `when: "not actor.signed_in"` that redirects:
   - the terminal does not redirect;
   - the generated React project reads `session.Me`.

   The same document with `actor: anonymous` redirects in both (a control).
2. `ess ui check` reports the same findings as before for every corpus document. It already read
   absent as `from_session`.
3. Every test of `ess-ui-tui`, `ess-ui-react` and `ess-ui-test` passes. Each document that needed
   `actor: anonymous` to keep its test green is listed in the report, with the test.

Also:

- `CHANGELOG.md`, under Changed: "a document without `actor` runs from the session, as `ess-ui/1`
  documents; write `actor: anonymous` for the earlier behaviour".

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3, with the owner's order "the loader fills every default".

1. **Need.** One meaning of an absent `actor` in the checker, both renderers, and an own generator
   reading `ess-ui-resolved/1`.
2. **Class.** This is a *defect*: both renderers contradict the documented default, and the
   checker follows it.
3. **Already expressible?** Writing `actor:` explicitly works today. The defect is the absent case.
4. **Fit.** There is no new key and no format version.
5. **Second adopter.** Any signed-in back-office document that leaves `actor` out and expects its
   grants checked.
6. **Cost.**
   - A visible change for every adopter document that leaves `actor` out: it now runs signed in.
     The changelog says how to keep the earlier behaviour.
   - Test documents gain one line each, where needed.
7. **Considered.**
   - (a) Correct the schema to `anonymous`: P0-D17 (B). That would change what `ess-ui-check`
     checks for every document.
   - (b) Chosen: the schema's side.

## Decisions

This story follows P0-D17. Under (B) it becomes a schema and checker change instead: the default
becomes `anonymous` and `model.rs:442` changes.

Found and outside P0: in fixture mode the two renderers take a `from_session` actor from different
places. React reads `session.Me`, and the terminal reads the first row of the shell's first
preloaded view (`app.rs:4655-4657`). A document whose preload is not the session view is signed in
in the terminal and anonymous in React. This is a defect for its own story.

## Scope

- cited: `crates/ui/ess-ui/src/fill.rs` (one key), `crates/ui/ess-ui-react/src/emit.rs`
  (3322-3325: the fallback becomes unreachable and is removed in `g03-no-renderer-fallback`)
- test documents under `crates/ui/ess-ui-tui/tests/`, `crates/ui/ess-ui-react/tests/` and
  `crates/ui/ess-ui-test/tests/` that need `actor: anonymous` (one line each)
- new cases: `crates/ui/ess-ui-tui/tests/page_actor.rs`, `crates/ui/ess-ui-react/tests/actor_default.rs`
- `CHANGELOG.md`
- Size: S to M, about 220 changed lines. The count of test documents that need the line is not
  known before the run; the estimate assumes up to 60.

## Depends on

- `g03-loader-fills-defaults`

It can be built beside `g03-defaults-that-move-behaviour`, but the two touch different test files
only if no document needs both. Land this one second.
