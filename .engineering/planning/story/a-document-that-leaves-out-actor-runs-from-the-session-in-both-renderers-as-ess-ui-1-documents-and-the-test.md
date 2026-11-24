---
format: aep.planning-md/3
id: story:a-document-that-leaves-out-actor-runs-from-the-session-in-both-renderers-as-ess-ui-1-documents-and-the-test
kind: story
status: active
title: Both renderers run a document without an actor from the session, and take the actor from one place
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:a-running-terminal-app-forks-into-a-copy-that-shares-no-mutable-state-with-it-and-keeps-file-placed-state-in
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:18:59Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-24T01:19:00Z", actor: "human:timo", revision: 5}
---
## Need

ESS documents one meaning for a document without `actor`: it runs from the session.

- The schema: `actor: {type: {enum: [from_session, anonymous]}, default: from_session, note: whose
  grants decide what is visible}` (`schemas/ui/ess-ui.schema.yaml:166`). The generated reference
  repeats it (`website/docs/reference/ess-ui.md:154`).
- The checker reads an absent value as `from_session`:
  `let grants_apply = matches!(document.actor, None | Some(ActorSource::FromSession));`
  (`crates/ui/ess-ui-check/src/model.rs:442`).
- The test reference fixes the actor of a fixture run: "The signed-in actor of a fixture run is the
  first row of the first view the home page's shell preloads. With no preload, `actor.account_id`
  is null. A shell `preload` of the session view therefore sets it, and `actor: from_session` signs
  it in." (`website/docs/reference/ess-ui-test.md:351-353`).

Both renderers contradict this, and they also contradict each other:

- **React runs an absent value as anonymous** (`crates/ui/ess-ui-react/src/emit.rs:3322-3325`).
  For `from_session` it reads a view named `session.Me` that is hard-coded in the runtime, and
  signs in only when a row arrives (`templates/runtime/data.ts.tmpl:943-964`). The schema and the
  reference name no such view.
- **The terminal runs an absent value signed out** (`crates/ui/ess-ui-tui/src/app.rs:4660-4662`).
  For `from_session` it signs in whether or not a row exists. It takes the row from the shell of
  the page on screen (`app.rs:4657`), not from the home page's shell, which is where `actor_ids`
  reads it (`app.rs:4876-4885`).

So a `from_session` document whose fixtures have no actor row is signed in in the terminal and
anonymous in React. Filling the default without fixing the renderers would spread that
disagreement to every document that leaves `actor` out, which is design finding D11. This unit
fixes the defect in both renderers. It lands before `g03-resolved-json`, so `ess-ui-resolved/1`
never publishes an identity default that a stock renderer contradicts.

Count, at `bcfc51555`:
- 228 test sources and fixture files under `crates/ui/*/tests` and `crates/edge/ess-cli/tests`
  hold an `ess-ui/1` document. One writes `actor:` (`ess-ui-check/tests/view_grant_security.rs`).
- Six read an `actor.` field:
  - four in `ess-ui-check`, which already reads absent as `from_session`;
  - `ess-ui-react/tests/adversary_pass1.rs:186` and `adversary_pass2.rs:251`, `:327`.
- One has a shell `preload`, in `ess-ui/tests`, which runs no renderer.
- `examples/partner-portal/ui.yaml` writes `actor: from_session`. Its shell preloads
  `session.Me` first (`:84-87`), and its fixtures answer that view (`fixtures/index.yaml:15`).

## Outcome

- **The fill.** The actor pass (`crates/ui/ess-ui/src/fill/actor.rs`) writes the row of its `HELD`
  list, `actor: from_session`, where the document leaves it out. The list stays, so the constant
  pass never writes it.
- **One actor rule in both renderers** (P0-D18, chosen by the coordinator on 2026-11-24). It takes
  the reference's row source with React's sign-in condition, so it is a choice, not the reference
  as written today:
  - The actor row is the first row of the first view the home page's shell preloads.
  - Under `from_session`, `actor.signed_in` holds when that row exists and the user has not signed
    out.
  - With no preload, or no row, nobody is signed in: the grants are nobody's, and
    `actor.account_id` is null.
  - Under `anonymous`, nobody signs in.
- **Before and after, per renderer:**

  | case | terminal before | terminal after | React before | React after |
  |---|---|---|---|---|
  | `actor` absent | signed out | the rule above | anonymous | the rule above |
  | `from_session`, shell preloads a view with a row | signed in as that row (the current page's shell) | signed in as that row (the home page's shell) | signed in as the `session.Me` row | signed in as that row |
  | `from_session`, no preload or no row | signed in, empty row | nobody signed in | anonymous | nobody signed in |
  | `anonymous` | anonymous | anonymous | anonymous | anonymous |

- **React.** The emitter passes the home page's shell's first preloaded view to `AppRoot`, which
  passes it to `useActor`. `useActor` reads that view in place of `session.Me`, and
  `actor: anonymous` reads nothing.
- **The test runner** has no actor code (`git grep actor -- crates/ui/ess-ui-test/src` finds
  none). It follows the terminal, and its Playwright specs follow React.
- **Test documents.** A test document of the count above that has no preload runs anonymous in
  both renderers, before and after. None of them needs `actor: anonymous` written.
- **The reference.** `website/docs/reference/ess-ui-test.md:351-353` is rewritten. Read literally,
  it says today that `actor: from_session` signs in even with no preload, which matches only the
  terminal's present rule (`app.rs:4655-4662`). The new text: "The actor of a run is the first row
  of the first view the home page's shell preloads. Under `actor: from_session` that actor is
  signed in; with no preload, or no row, nobody is signed in, the grants are nobody's, and
  `actor.account_id` is null. The terminal and the generated React project follow the same rule."
- **The schema** keeps `default: from_session`, and its note (`:166`, "whose grants decide what is
  visible") needs no clause: with nobody signed in, nobody's grants decide. So this unit edits
  neither the schema nor the generated reference, which `g03-model-defaults` edits in the same
  round. `ess-ui-check` does not change.

## Acceptance

New test files `crates/ui/ess-ui-tui/tests/actor_default.rs` and
`crates/ui/ess-ui-react/tests/actor_default.rs`. Each test is expected under 5 s, or 10 s where it
runs the generated runtime, and is timed; none may run over 60 s, and each crate's test step stays
within 120 s. Each file joins its crate's single test binary of
`story:the-test-step-meets-the-test-budget` if that has landed; otherwise it is a new file, which
that story moves.

The fixture document `D` has:
- no `actor`;
- a shell preloading `session.Me` first, with one fixture row `{id: u1, organization: {id: a1}}`;
- a home page guarded by `when: "not actor.signed_in"`, which redirects to a sign-in page.

1. `an_absent_actor_runs_from_the_session`. Red today.
   - The terminal: after `open`, the screen shows the home page's title, not the sign-in page's.
   - React: the emitted `src/App.tsx` passes the view `session.Me` to `AppRoot`. Run against the
     fixture adapter, the home page renders, not the sign-in page.
2. `the_actor_view_is_the_first_preloaded_view`. `D` with a shell that preloads `team.Lead`
   first, whose row is `{id: u2}`:
   - in both renderers, `actor.user_id` is `u2`;
   - React reads no `session.Me`: the run's read log holds no request for it.
3. `no_row_means_anonymous_in_both`. `D` without the preload: both renderers redirect to the
   sign-in page. Red today in the terminal, which signs in with an empty row.
4. `anonymous_signs_in_nobody`. `D` with `actor: anonymous`: both renderers redirect.
5. `the_filled_tree_holds_from_session`. `load_resolved` of `D` holds `actor: from_session`.
6. `the_reference_states_the_rule`. `ess-ui-test.md` holds the new sentence "with no preload, or no
   row, nobody is signed in", and no longer holds "`actor: from_session` signs it in".
7. Existing tests: the test sets of `ess-ui-tui`, `ess-ui-react`, `ess-ui-test` and
   `ess-ui-check` pass with no expectation changed. Only `ess-ui-react/tests/adversary_pass1.rs`
   and `adversary_pass2.rs` may move; each moved line names this story. Any other move stops the
   unit and is reported.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui -p ess-ui-tui -p ess-ui-react --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui -p ess-ui-tui -p ess-ui-react -p ess-ui-test -p ess-ui-check --locked`;
- `task test-xtask` (`website/` changes).

`CHANGELOG.md`, under Changed:
- "**Breaking for a generated React project**: a document without `actor` runs from the session,
  as `ess-ui/1` documents (`ess ui check` already read it so); the actor is the first row of the
  first view the home page's shell preloads, where it was the view `session.Me`; write
  `actor: anonymous` for the earlier behaviour."
- "The terminal runs a document without `actor` from the session. A `from_session` terminal run
  whose home page's shell preloads nothing, or whose first preloaded view has no row, now runs
  with nobody signed in, where it ran signed in with an empty actor: a guard on
  `actor.signed_in` redirects there, and `actor.may(…)` is false."

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3, with the owner's order "the loader fills every default". The operator ruled on 2026-11-24
that `actor` is an identity default. The design was read for its intended meaning: the schema
(`:166`), the checker (`model.rs:442`) and the test reference (`ess-ui-test.md:351-353`) all say
"from the session". So this is a defect fixed in P0.

1. **Need.** One meaning of an absent `actor` in the checker, both renderers and an own generator
   reading `ess-ui-resolved/1`.
2. **Class.** It is a *defect*: both renderers contradict the documented default, and they disagree
   with each other on where the actor comes from.
3. **Already expressible?** Writing `actor: from_session` works today, but the two renderers then
   take the actor from different places.
4. **Fit.** No new key and no format version. The row source is the reference's; the sign-in
   condition is React's, chosen over the reference's literal wording, which is rewritten to match.
   The hard-coded view name in the React runtime is dropped, since the schema does not back it.
5. **Second adopter.** Any signed-in back office that leaves `actor` out and expects its grants
   to decide what is visible, as `ess ui check` already assumes.
6. **Cost.**
   - A visible change for a document that leaves `actor` out and has an actor row: it now runs
     signed in.
   - A served generated React app whose home shell's first preload is not its session view now
     reads its actor from that first view. The reference example and the partner portal preload
     `session.Me` first (`website/docs/reference/ess-ui.md:351`, `ui.yaml:87`).
   - No test document needs a new line.
7. **Considered.**
   - (a) Correct the schema to `anonymous` (the P0-D17 B treatment). Rejected by the operator's
     ruling: the design says "from the session", and `ess-ui-check` would change what it checks
     for every document.
   - (b) React's rule in both renderers (P0-D18 B).
   - (c) The terminal's rule in both renderers (P0-D18 C).
   - (d) Chosen: the reference's row source with React's sign-in condition, in both renderers,
     and the reference rewritten (P0-D18 A).

## Decisions

- **P0-D18: Which actor rule do both renderers follow?**
  - (A) The reference's row source with React's sign-in condition: the first row of the first view
    the home page's shell preloads, signed in under `from_session` only when that row exists; with
    no preload or no row, nobody is signed in. React stops hard-coding `session.Me`, the terminal
    stops signing in without a row, and `ess-ui-test.md:351-353` is rewritten to say so.
  - (B) React's rule in both: the view `session.Me`, signed in when it answers a row. The
    terminal moves, and the reference text is corrected to name a view that the schema does not
    know.
  - (C) The terminal's rule in both: signed in under `from_session` whether or not a row exists,
    and React's fixture build signs in without a row. Then every test document without `actor`
    runs signed in in both renderers, and each that relied on anonymous must write it. That is
    unknown without a run, and at most the two React files above among those that read `actor.`.

  **Settled: (A)**, chosen by the coordinator on 2026-11-24.

## Scope

- `crates/ui/ess-ui/src/fill/actor.rs`: the actor pass writes its `HELD` row; the list stays.
- `crates/ui/ess-ui-tui/src/app.rs`: 4655-4662, signed in only with a row; `actor_row` at 4855
  takes the home page's shell, as `actor_ids` at 4878 does.
- `crates/ui/ess-ui-react/src/emit.rs`: 3322-3325, pass the actor view.
- `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl`: `AppRoot` 125-129, a `view` prop.
- `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`: `useActor` 943-964.
- `website/docs/reference/ess-ui-test.md`: 351-353.
- new: `crates/ui/ess-ui-tui/tests/actor_default.rs`, `crates/ui/ess-ui-react/tests/actor_default.rs`.
- may move: `crates/ui/ess-ui-react/tests/adversary_pass1.rs`, `adversary_pass2.rs`.
- `CHANGELOG.md`.
- Size: M, about 260 changed lines: code about 70, tests about 190.

## Depends on

- `g03-loader-fills-defaults`: `fill/actor.rs` and `Loaded`.
- `g01-app-forks`, which edits `app.rs` in round 1.

Round 2, beside `g03-model-defaults`, with which it shares no file.
