---
format: aep.planning-md/3
id: review-result:ui-live-apps-acceptance-round-1
kind: review-result
status: active
title: Plan critic acceptance, round 1, epic:ui-live-apps
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

- story:served-store-and-entry — the acceptance registers a gatepass visit and expects 17/17 "once every gatepass command is generated", but RegisterVisit stays owed. The plan keeps it an obligation (`generated/go/gatepass/PLAN.md:59`, ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/generated/go/gatepass/PLAN.md), and the story's own Decisions make an entry point with an owed command refuse to start. The tests as written cannot pass, and "once every command is generated" names no work that produces that state. The 17/17 line also names no test and says "keeps", so it reads the same before and after the work. Cite ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-store-and-entry.md:76-77.
- story:ui-binding-contract — the acceptance lists the S2, S3, S4 and S5 tests that four other stories also list, so two stories claim one outcome and can be marked done on the same test. `…_go_server` is an ellipsis, not a test name. None of the S1 tests exercises the typed classification of command answers the Outcome promises. Cite ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md:58-64.
- story:ui-react-live-binding — "plus one end-to-end test that runs the generated, built app's adapter against a synthesized gatepass server" names no test, file or function, so it cannot be run red first or checked for existence. Cite ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-react-live-binding.md:38.
- story:ui-tui-app-generator — "plus a run of the built crate against a synthesized gatepass server reading a view" names no test, so the only thing proving the crate works, as opposed to building, is unnamed. Cite ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-tui-app-generator.md:35.
- story:related-guard-behaviour — `a_stored_reference_guard_is_generated_once_ess_21_has_it` asserts the named refusal until ess/21 lands and generation afterwards. One name carries two opposite checks, and it duplicates `a_stored_reference_guard_stays_an_obligation_with_its_contract`. "Plans byte-identical for models without `when_related`" names no test. Cite ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-guard-behaviour.md:67-68.
- story:related-via-optional-input — the acceptance carries the stored-reference tests that story:related-via-stored-reference also claims, so both stories can be closed on the same tests. These are `a_stored_reference_via_validates_under_ess_21`, `a_stored_reference_via_is_refused_on_creates_and_without_a_subject`, `a_stored_reference_lowers_to_resolved_related_via_subject`, all five `related_guard_stored_reference.rs` tests, and `a_stored_reference_guard_stays_an_obligation_with_its_contract`. Cite ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-via-optional-input.md:58 against ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-via-stored-reference.md:38.
- story:go-generated-behaviour — the acceptance says "Go tests return early without a toolchain", so on a machine without Go every listed test is green without running. The criteria do not say that a done run must be on a host that has Go. Cite ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/go-generated-behaviour.md:52 and ~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/examples/gatepass-realization/tests/conformance.rs:853-856.

**Read:** all 11 stories in full with `aep plan artifact show`, plus `epic:ui-live-apps`, `aep plan artifact kinds` and the story lifecycle. In the tree I grepped for the named tests and read `declared_behaviour.rs:552`, `partner_portal.rs:138`, `conformance.rs:60-75,852-858`, and gatepass `PLAN.md` and `TARGET.md`.

**Could not establish:**
- Whether the "new" test names collide with existing ones. A grep for the new names across `crates` and `examples` found none already present.
- Whether `bundle.rs` skips without esbuild or `node`, because the file does not exist yet.
- Out of my lane (design): `story:ui-tui-live-binding` lists `depends_on story:ui-react-live-binding`, while its Sequencing and the epic table say the two run in parallel.
- Out of my lane (design): `story:related-guard-behaviour` has no `depends_on` edge to `story:related-via-stored-reference`, although its Sequencing needs one.

```findings
[
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/served-store-and-entry.md",
    "line": 77,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the acceptance registers a gatepass visit and expects 17/17 \"once every gatepass command is generated\", but RegisterVisit stays owed (generated/go/gatepass/PLAN.md:59) and the story's own Decisions make an entry point with an owed command refuse to start, so the tests cannot pass as written; the 17/17 line also names no test and reads the same before and after the work"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-binding-contract.md",
    "line": 58,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the acceptance lists the S2, S3, S4 and S5 tests that four other stories also list, so two stories claim one outcome and can be marked done on the same test; `…_go_server` is an ellipsis, not a test name; no S1 test exercises the typed classification of command answers the Outcome promises"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-react-live-binding.md",
    "line": 38,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"plus one end-to-end test that runs the generated, built app's adapter against a synthesized gatepass server over HTTP\" names no test, file or function, so it cannot be run red first or checked for existence"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/ui-tui-app-generator.md",
    "line": 35,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "\"plus a run of the built crate against a synthesized gatepass server reading a view\" names no test, so the only proof the crate works rather than merely builds is unnamed"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-guard-behaviour.md",
    "line": 67,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "`a_stored_reference_guard_is_generated_once_ess_21_has_it` asserts the named refusal until ess/21 lands and generation afterwards, so one name carries two opposite checks and duplicates `a_stored_reference_guard_stays_an_obligation_with_its_contract`; \"Plans byte-identical for models without `when_related`\" names no test"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/related-via-optional-input.md",
    "line": 58,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the acceptance carries the stored-reference tests that story:related-via-stored-reference also claims (the three `a_stored_reference_*` validation, refusal and lowering tests, all five related_guard_stored_reference.rs tests, and `a_stored_reference_guard_stays_an_obligation_with_its_contract`), so both stories can be closed on the same tests"
  },
  {
    "file": "~/.local/state/worktree/trees/b10x/ess/ess-ui-live-apps-plan/.engineering/planning/story/go-generated-behaviour.md",
    "line": 52,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the acceptance says Go tests return early without a toolchain, so on a host without Go every listed test is green without running and the criteria do not require a Go-equipped run as the evidence of done"
  }
]
```
