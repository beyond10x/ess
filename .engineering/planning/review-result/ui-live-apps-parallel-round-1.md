---
format: aep.planning-md/3
id: review-result:ui-live-apps-parallel-round-1
kind: review-result
status: active
title: Plan critic parallel, round 1, epic:ui-live-apps
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

- story:served-store-and-entry — collides with story:related-guard-behaviour (both depend only on go-generated-behaviour and neither names the other): both type `crates/generate/ess-synth/src/go`, `generated` and `website/docs/guides/synthesize.md` (cited), and `rust/behaviour.rs` (inferred, since the Rust storage port lives there). Say so in Sequencing with an edge, or split the surface — `aep plan artifact waves` only keeps them apart by derived wave — `.engineering/planning/story/served-store-and-entry.md:81,85` (other side `related-guard-behaviour.md:72,76`; `aep plan artifact waves` prints `collision: story:related-guard-behaviour story:served-store-and-entry …/src/go` and `… generated`)
- story:related-guard-behaviour — types the whole directory `crates/generate/ess-synth/src/go` while its body names only `src/go/behaviour.rs`. That directory contains served-view-params' `go/http.rs` and `go/port.rs`, and `aep plan artifact waves` puts the two in the same wave 2 because it does not match a directory against a file inside it. The same applies to served-store-and-entry's `src/go/` and `src/rust/`. Narrow the typed paths to files, or add an ordering edge (cited by typed containment; whether a real file is shared is inferred) — `.engineering/planning/story/related-guard-behaviour.md:72` (against `served-view-params.md:44`)
- story:related-guard-behaviour — shares `crates/generate/ess-synth/tests/declared_behaviour.rs` with story:related-via-optional-input (cited, both Acceptance sections) and the Related arm of `plan.rs` (cited on the optional-input side, inferred for "plan rendering"). The Sequencing line orders only the stored-reference half after related-via-stored-reference, and there is no edge to optional-input — `.engineering/planning/story/related-guard-behaviour.md:66,72,76` (against `related-via-optional-input.md:58,63`; `crates/generate/ess-synth/src/plan.rs:722`)
- story:ui-binding-contract — the typed scope is S1's four paths, but the body's Scope and Sequencing name S2–S5 files: `ess-ui-react` `emit.rs`/`main.tsx`/`index.html`/`README`, `ess-cli/src/ui.rs`, `playwright.rs`, and `ess-synth` `http.rs`/`port.rs`. ui-react-plain is also order 1 with no edge and types the React crate and `ui.rs`. Trim the body to S1, or order the two. Its Sequencing also says "S2 and S3 in parallel" and "S5 independent", which the `depends_on` edges contradict (cited) — `.engineering/planning/story/ui-binding-contract.md:68,72` (against `ui-react-plain.md` Scope)
- story:ui-tui-live-binding — Sequencing says "parallel to story:ui-react-live-binding" while the frontmatter carries `depends_on: story:ui-react-live-binding`, and both edit `ess-cli/src/ui.rs`. The edge is right and the sentence is stale; the epic table's order-2 row repeats it — `.engineering/planning/story/ui-tui-live-binding.md:44`
- story:ui-tui-live-binding — its tests run against "the gatepass Rust server" and "the gatepass Go server". If those are the `examples/gatepass-realization` and `examples/gatepass-go-realization` servers (inferred, the story names no path), go-generated-behaviour rewrites `visit.go` and `linker.go` and served-store-and-entry replaces the hand-written server. The UI chain has no edge to either story. story:ui-react-live-binding's end-to-end test is in the same position — `.engineering/planning/story/ui-tui-live-binding.md:36` (against `go-generated-behaviour.md:56`, `served-store-and-entry.md:77`)
- story:ui-react-live-binding — `CHANGELOG.md` `[Unreleased]` is edited by every story that ships user-visible change. Only ui-react-plain calls it merge-time (cited); go-generated-behaviour and served-store-and-entry list it as inferred; this story and the other seven say nothing. Add a one-line Sequencing note to each, or record it once in the epic — `.engineering/planning/story/ui-react-live-binding.md:42,46` (against `ui-react-plain.md` Sequencing)

**What I read:** 11 stories, the epic, feature-request-282/287/310 and the planning-store graph, using `aep plan artifact show/graph/waves`. I also read `ci.yml`, `Cargo.toml`, `generated/`, `determined.rs`, `plan.rs`, `rust/behaviour.rs` and the `gaps-287` and `gaps-310` diffs. Surfaces: 10 cited, 1 partly inferred (ui-tui-app-generator), 0 unplaceable.

**Checked, no finding:**
- `ci.yml`: only ui-react-plain edits it, and the `test` job already has Go 1.25.10 and Node 22.
- `ess-cli/src/ui.rs`: the four stories that touch it are chained by `depends_on` edges.
- `Cargo.lock`: only the UI chain's dependency edits change it (contract adds `ess-gen` to `ess-ui-check`, react-live edits `Cargo.toml`), and the chain is serial.
- #287 (`synthesize.rs`, `synthesize/{caller,existence,related,related_guard,singleton}.rs`): it reaches only related-via-optional-input and related-via-stored-reference, both with an edge.
- #310 (`determined.rs`, `existence.rs`, `rust/behaviour.rs`, `rust/mod.rs`, `synthesize.md`): it reaches go-generated-behaviour, related-guard-behaviour and served-view-params with edges, and served-store-and-entry transitively.
- `ess-cli/src/ui.rs` and the other UI paths are untouched by both open units.

**Not established:**
- ui-tui-app-generator: the generator module's crate is "decided in the unit". If it becomes a new crate, root `Cargo.toml` and `Cargo.lock` change; if it goes into `ess-ui-tui`, it shares that crate with ui-tui-live-binding, which the edge already orders.
- `release/0.51.0` (`5e55862b5`) exists in `~/.local/state/worktree/trees/b10x/ess/ess-gaps-plan` and rewrites `Cargo.lock`, `determined.rs` and `synthesize.md`. This tree is on 0.50.0, so line cites into those files will move.

**Out of my lane:**
- ui-binding-contract's Acceptance repeats the test names of S2–S5, which four other stories also claim.
- related-via-optional-input and related-via-stored-reference both claim `a_stored_reference_guard_stays_an_obligation_with_its_contract`.
- "These target 0.51.0" reads stale now that 0.51.0 is cut.

**Scratch left behind:** mid-run Bash failed with ENOSPC on `~/.cache/claude-tmp`, so I finished with Read. I left a stale `/tmp/x` (an `aep plan artifact waves` dump, mine) and `~/.cache/uilab-todo/pscrit-waves.txt` (also mine). Both can be deleted. I wrote nothing to the store or the tree.

```findings
[
  {
    "file": ".engineering/planning/story/served-store-and-entry.md",
    "line": 85,
    "category": "parallel-safety",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "both this and story:related-guard-behaviour depend only on go-generated-behaviour and both land on crates/generate/ess-synth/src/go, generated and website/docs/guides/synthesize.md (cited, typed scopes) and rust/behaviour.rs (inferred); neither body names the other and no edge exists, so add an ordering edge recording the shared files or split the surface"
  },
  {
    "file": ".engineering/planning/story/related-guard-behaviour.md",
    "line": 72,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the typed path crates/generate/ess-synth/src/go (a whole directory, though the body names only go/behaviour.rs) contains story:served-view-params' go/http.rs and go/port.rs; aep plan artifact waves does not match directory against file, so it places both in wave 2; the same holds for served-store-and-entry's src/go and src/rust; narrow to files or add an edge (cited by containment, whether a real file is shared is inferred)"
  },
  {
    "file": ".engineering/planning/story/related-guard-behaviour.md",
    "line": 76,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "shares crates/generate/ess-synth/tests/declared_behaviour.rs (cited in both Acceptance sections) and the Related arm of ess-synth/src/plan.rs (cited on the related-via-optional-input side, inferred here as \"plan rendering\") with story:related-via-optional-input; Sequencing orders only the stored-reference half after related-via-stored-reference and no edge exists to optional-input"
  },
  {
    "file": ".engineering/planning/story/ui-binding-contract.md",
    "line": 68,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "typed scope is S1's four paths but the body Scope and Sequencing name the S2-S5 files (ess-ui-react emit.rs/main.tsx/index.html/README, ess-cli/src/ui.rs, playwright.rs, ess-synth http.rs/port.rs) and so collide on paper with story:ui-react-plain, which is order 1 with no edge; trim the body to S1 or order the pair; the line \"S2 and S3 in parallel, S5 independent\" contradicts the depends_on edges"
  },
  {
    "file": ".engineering/planning/story/ui-tui-live-binding.md",
    "line": 44,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "Sequencing says parallel to story:ui-react-live-binding while the frontmatter has depends_on story:ui-react-live-binding and both edit crates/edge/ess-cli/src/ui.rs; the edge is right and the sentence is stale, as is the epic table's order-2 row"
  },
  {
    "file": ".engineering/planning/story/ui-tui-live-binding.md",
    "line": 36,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "inferred, since the story names no server path - its tests run against the gatepass Rust and Go servers, which are likely examples/gatepass-realization and examples/gatepass-go-realization; story:go-generated-behaviour rewrites visit.go and linker.go there and story:served-store-and-entry replaces the hand-written server, and the UI chain (react-live's end-to-end test too) has no edge to either"
  },
  {
    "file": ".engineering/planning/story/ui-react-live-binding.md",
    "line": 46,
    "category": "parallel-safety",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "CHANGELOG.md [Unreleased] is edited by every story that ships user-visible change, but only story:ui-react-plain calls it merge-time (cited) and go-generated-behaviour and served-store-and-entry list it as inferred; this story and seven others say nothing, so add a Sequencing note to each or record it once in the epic"
  }
]
```
