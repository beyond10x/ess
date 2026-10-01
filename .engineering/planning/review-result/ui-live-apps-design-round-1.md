---
format: aep.planning-md/3
id: review-result:ui-live-apps-design-round-1
kind: review-result
status: active
title: Plan critic design, round 1, epic:ui-live-apps
relations:
- reviews: story:ui-react-plain
- reviews: story:ui-binding-contract
- reviews: story:ui-react-live-binding
- reviews: story:ui-tui-live-binding
- reviews: story:ui-tui-app-generator
- reviews: story:served-view-params
- reviews: story:go-generated-behaviour
- reviews: story:served-store-and-entry
- reviews: story:related-guard-behaviour
- reviews: story:related-via-optional-input
- reviews: story:related-via-stored-reference
revision: 1
---
needs-revision

ui-binding-contract — its Decisions (:40), Acceptance (:58) and Scope (:66) still carry S2–S5, so it and ui-react-live-binding, ui-tui-live-binding, ui-tui-app-generator and served-view-params all claim the same tests and files; cut the body to S1 (the `ess_ui::binding` types, `ess ui check --model`, the answer classification) — `.engineering/planning/story/ui-binding-contract.md:58` against `story/ui-react-live-binding.md:36`

ui-binding-contract — the Outcome promises "a typed classification of command answers", but no S1 acceptance names a test for it and each renderer re-implements it (TS in S2, Rust in S3), so the half-abstraction has no owner; ship the classifier as data or a Rust function in `ess_ui::binding` with one shared vector file that S2 and S3 both run — `story/ui-binding-contract.md:24,58`

ui-tui-live-binding — its graph edge `depends_on story:ui-react-live-binding` contradicts its own Sequencing line ("parallel to story:ui-react-live-binding") and the epic table (depends only on ui-binding-contract). That edge turns the UI side into the 4-deep chain react-plain → react-live → tui-live → tui-app. If the edge records the shared `ess-cli/src/ui.rs`, the choice is keeping it and fixing the body and epic table, or moving the `--model` flag plumbing on `ess generate ui` into ui-binding-contract so S2 and S3 stop colliding — `aep plan artifact graph`, `story/ui-tui-live-binding.md:44`

related-via-optional-input — its Acceptance (:56) lists the stored-reference tests too (`a_stored_reference_via_validates_under_ess_21`, `related_guard_stored_reference.rs`, `a_stored_reference_guard_stays_an_obligation_with_its_contract`), which story:related-via-stored-reference also lists. These tests need #282's precedence step, which this story does not depend on, so both stories would claim them while only one can pass first; trim to the `issue_304_*` tests — `story/related-via-optional-input.md:56` against `story/related-via-stored-reference.md:36`

related-guard-behaviour — the stored-reference half of its outcome is gated on story:related-via-stored-reference ("the stored-reference half after…", and the test flips "once ess/21 has it"). No edge records that, and the epic puts this story at order 4 and the other at 5. This is the case uilab needs, so the story cannot be done alone. Either add `depends_on story:related-via-stored-reference` and move it to order 6, or split it into an input-via story and a stored-via story — `story/related-guard-behaviour.md:67,76`

served-store-and-entry — its witnesses cannot hold with this set. The Go entry point must serve gatepass with no hand-written code and register as Receptionist (202), yet RegisterVisit stays owed (go-generated-behaviour fit review point 1; `generated/go/gatepass/PLAN.md:59`), and the body says an entry point with an owed command refuses to start. No story in the set generates `creates:` commands. Use a fixture whose commands are all determined, or add the story that generates them — `story/served-store-and-entry.md:76-77`

served-store-and-entry — it reverses "Storage is a port the implementor provides, never a generated store" (`.engineering/planning/epic/generated-determined-behaviour.md:29`) with no relation to that epic and no amendment. Its Scope names only `synthesize.md`, while `website/docs/concepts/ess.md:203` ("ESS never generates a store") and `synthesize.md:312` would also contradict the shipped behaviour — `story/served-store-and-entry.md:79`

served-store-and-entry — `--callers actor-header` makes the generated entry point read an actor from the request, against `synthesize.md:273-274` ("the server reads no actor from the request itself") and D-2 (a synthesis target chooses nothing: `docs/design/linker-never-chooses.md`). The body should state why an operator-named flag is not the machinery choosing, and amend `synthesize.md:273` — `story/served-store-and-entry.md:59-70`

served-view-params — only half the abstraction is placed. The server decodes params and hands them to the port, but "generated queries for views with params stay obligations", and go-generated-behaviour also leaves "views with parameters or paging" owed. No story makes a param view generated (`view_query.rs:31-35` refuses it), so any param view the todo app reads stays hand-written against the epic's "nothing hand-written". Name the story that closes it, or state it as an exclusion in the epic — `story/served-view-params.md:34`

related-via-stored-reference — it "takes over acceptance line 1 of story:related-guard-vocabulary-aligns" with no edge between the two stories, so that story's line would be met by a story in another epic and nothing records it. Record the relation in the store's own vocabulary (`aep plan artifact relations`: `supersedes` or `informed_by`) or amend the vocabulary story — `story/related-via-stored-reference.md:34` against `story/related-guard-vocabulary-aligns.md`

**What I read.** 11 stories, the epic, epic:ess-ui-renderer-neutral-ui and related-guard-vocabulary-aligns, via `aep plan artifact show/relations/graph/validate`. I walked the graph's 40-odd edges outside the set too (feature-request-282/287/310 and the vision): it is acyclic, and `validate` printed only 25-day-old review-result notes, which are not mine. I also read ESS `AGENTS.md`, `docs/design/{linker-never-chooses,cross-record-and-stored-field-guards}.md` and `website/docs/guides/synthesize.md`. The dependencies `ess-ui-check`→`ess-gen` and `ess-ui`'s binding module with no dependencies hit no rule I could find. `ess-gen` does not depend on the ui crates, so there is no cycle.

**What I could not establish.**
- Whether uilab's todo model reads a view with `params:`, which decides whether the served-view-params finding is a blocker. The example does not exist yet.
- Whether there is a crate-boundary test for `ess-ui-check`→`ess-gen`: `AGENTS.md:111` names a "dependency boundary test" but I found only `ess-cli/src/main.rs` matching.

**Out of my lane.**
- Scope: nothing in the set runs the todo model end to end. The epic outcome needs related-via-stored-reference, related-guard-behaviour, served-store-and-entry and both clients, and no acceptance covers them together.
- Acceptance: ui-tui-app-generator leaves "crate placement decided in the unit".

```findings
[
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md",
    "line": 58,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "its Decisions, Acceptance and Scope still carry S2-S5, so it and ui-react-live-binding, ui-tui-live-binding, ui-tui-app-generator and served-view-params all claim the same tests and files; cut the body to S1"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md",
    "line": 24,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the Outcome promises a typed classification of command answers but no S1 acceptance names a test for it and S2 (TS) and S3 (Rust) each re-implement it; ship it as data or a Rust function in ess_ui::binding with one shared vector file that S2 and S3 both run"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-tui-live-binding.md",
    "line": 44,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "its depends_on story:ui-react-live-binding contradicts its own Sequencing line (parallel) and the epic table, making a 4-deep UI chain; if the edge records the shared ess-cli/src/ui.rs, either keep it and fix the body and epic table, or move the --model flag plumbing into ui-binding-contract so S2 and S3 stop colliding"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-via-optional-input.md",
    "line": 56,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "its Acceptance lists the stored-reference tests that story:related-via-stored-reference also lists, and they need #282's precedence step which this story does not depend on; trim to the issue_304_* tests"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-guard-behaviour.md",
    "line": 76,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the stored-reference half of the outcome is gated on story:related-via-stored-reference with no edge, and the epic orders this story before it; add depends_on story:related-via-stored-reference and move it to order 6, or split it into an input-via story and a stored-via story"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-store-and-entry.md",
    "line": 76,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the gatepass witness cannot hold because RegisterVisit stays owed (PLAN.md:59) so the entry point refuses to start, and no story in the set generates creates: commands; use a fully determined fixture or add the story that generates them"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-store-and-entry.md",
    "line": 79,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "it reverses epic:generated-determined-behaviour:29 with no relation or amendment and its Scope omits website/docs/concepts/ess.md:203 and synthesize.md:312 which say ESS never generates a store"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-store-and-entry.md",
    "line": 59,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "--callers actor-header reads an actor from the request against synthesize.md:273-274 and D-2; state why an operator-named flag is not the machinery choosing and amend synthesize.md:273"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-view-params.md",
    "line": 34,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "only the server half is placed (params reach the port) while queries for param views stay obligations here and in go-generated-behaviour, so no story makes a param view generated; name the story that closes it or state the exclusion in the epic"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-via-stored-reference.md",
    "line": 34,
    "category": "design",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "it takes over acceptance line 1 of story:related-guard-vocabulary-aligns with no relation between the two; record supersedes or informed_by, or amend the vocabulary story"
  }
]
```
