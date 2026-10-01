---
format: aep.planning-md/3
id: review-result:ui-live-apps-scope-round-1
kind: review-result
status: active
title: Plan critic scope, round 1, epic:ui-live-apps
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

ui-binding-contract — its Decisions (S2–S5), Acceptance (S2–S5 tests) and Scope list the work that ui-react-live-binding, ui-tui-live-binding, ui-tui-app-generator and served-view-params each claim, so five items claim the same outcomes and all will be marked done; trim the body to S1, which is the only part its Outcome states — ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md:50-64
related-via-optional-input — its Acceptance lists the stored-reference tests (`a_stored_reference_via_validates_under_ess_21`, `a_stored_reference_via_is_refused_on_creates_and_without_a_subject`, `a_stored_reference_lowers_to_resolved_related_via_subject`, all of `related_guard_stored_reference.rs`, `a_stored_reference_guard_stays_an_obligation_with_its_contract`) that related-via-stored-reference also claims, while its Outcome claims only the Optional input via; drop them and keep the `issue_304_*` tests — ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-via-optional-input.md:58
related-guard-behaviour — the headline blocked-by case is not claimed by any acceptance. The epic says "commands guarded by `when_related` are generated in the code targets" and the todo-app's blocker is an Optional stored reference. This story's Decisions cover only a row read by `get` and `exists: false`, with no absent-reference case or subject via. Its only stored-reference test is a placeholder asserting the named refusal. The story has no `depends_on` edge to related-via-stored-reference or related-via-optional-input, and the epic orders it at 4, before both. Add the edges and acceptance for generating the stored-reference and absent-reference guard, or record that it is owed by name — ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-guard-behaviour.md:56-67 (epic ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/epic/ui-live-apps.md:28-31)
ui-tui-live-binding — the epic promises "A refusal the model declares reaches the user where they acted", and the contract shows a refusal on the "form/confirm/action that sent it". This story narrows the TUI to "a refusal shows on the open form overlay", with only `a_refusal_shows_on_the_open_form` as a test. A TUI refusal from a confirm or action, which is where completing a blocked task would be refused, has no owner. Cover confirm and action, or record the narrowing — ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-tui-live-binding.md:32 (epic :20)
ui-binding-contract — the epic promises running apps with nothing hand-written, and the generated React app is dev-served by esbuild with no proxy. The contract excludes CORS and sends the Go `Serve` wrap gap "to #314". story:go-generated-behaviour claims no such work and lists "the Go server contract" under Must not change, and no other story in the set claims it. As drafted, a cross-origin run needs a hand-written reverse proxy. Either claim the wrap or CORS gap in a story or record it as owed by name — ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md:48 (epic :17)

What I read: the epic and all 11 stories, whole bodies, via `aep plan artifact show`, plus `aep plan artifact graph` (head only, since the output was long). I extracted 9 promises from the epic and traced 6 fully to a story. The 3 not fully traced are the refusal shown where the user acted in the TUI (narrowed), generated blocked-by behaviour (conditional on a placeholder test), and running with nothing hand-written (the proxy question).

What I could not establish: I did not read the graph past the first 80 lines, so I did not check whether older items outside the set already claim any of these. The remaining items below are outside my lane and did not set the verdict.
- Design lane: ui-tui-live-binding has `depends_on ui-react-live-binding`, but its own Sequencing and the epic table say it is parallel and depends only on ui-binding-contract.
- Design lane: go-generated-behaviour's Decisions say the store and `main` are "declined", which contradicts the epic's 2026-10-01 reversal that served-store-and-entry implements.
- Acceptance lane: ui-react-live-binding's test `command_answers_classify_as_the_contract_declares` checks classification only, not that a refusal renders on an action.

```findings
[
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md",
    "line": 50,
    "category": "scope",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "its Decisions (S2-S5), Acceptance (S2-S5 tests) and Scope carry the work that ui-react-live-binding, ui-tui-live-binding, ui-tui-app-generator and served-view-params each claim, so five items claim the same outcomes and all will be marked done; trim the body to S1, the only part its Outcome states"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-via-optional-input.md",
    "line": 58,
    "category": "scope",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "its Acceptance lists the stored-reference tests (a_stored_reference_via_*, related_guard_stored_reference.rs, a_stored_reference_guard_stays_an_obligation_with_its_contract) that related-via-stored-reference also claims, while its Outcome claims only the Optional input via"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-guard-behaviour.md",
    "line": 56,
    "category": "scope",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the epic promises when_related-guarded commands generated in the code targets (the blocked-by case, an Optional stored reference), but this story's Decisions cover only a get-read row with exists:false, no absent-reference case or subject via, its stored-reference test is a placeholder asserting a refusal, and it has no depends_on edge to related-via-stored-reference or related-via-optional-input; add the edges and acceptance for generating the stored-reference and absent-reference guard, or record it as owed by name"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-tui-live-binding.md",
    "line": 32,
    "category": "scope",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the epic promises a refusal reaches the user where they acted and the contract names form, confirm and action, but this story narrows the TUI to the open form overlay, so TUI refusals from a confirm or action have no owner; cover them or record the narrowing"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md",
    "line": 48,
    "category": "scope",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "it sends the Go Serve wrap gap that blocks a cross-origin run to #314, but story:go-generated-behaviour claims no such work and forbids changing the Go server contract, so no item in the set claims it and the generated apps need a hand-written proxy to run against the generated server; claim it in a story or record it as owed by name"
  }
]
```
