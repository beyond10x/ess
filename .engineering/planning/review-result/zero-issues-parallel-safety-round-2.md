---
format: aep.planning-md/3
id: review-result:zero-issues-parallel-safety-round-2
kind: review-result
status: active
title: Parallel-safety critic, round 2, epic zero-open-issues-2026-11
relations:
- reviews: epic:zero-open-issues-2026-11
- reviews: story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for
- reviews: story:a-not-defined-branch-narrows-an-optional-in-later-branches
- reviews: story:a-related-row-may-answer-before-input-guards
- reviews: story:conformance-reads-a-view-no-row-holds
- reviews: story:the-generated-rust-server-reads-a-multipart-request
revision: 1
---
needs-revision

story:a-not-defined-branch-narrows-an-optional-in-later-branches — its acceptance reads the format version that admits the narrowing, which story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs opens as `ess/37`, but it has no depends_on edge to that story (directly or through another story) or to story:format-rule-for-relaxations, which its Decisions section points to, so one wave can hold the reader and the story that produces the format — .engineering/planning/story/a-not-defined-branch-narrows-an-optional-in-later-branches.md:87, :110; .engineering/planning/story/ess-37-and-conformance-64-65-are-open-for-the-epics-constructs.md:44
story:the-generated-rust-server-reads-a-multipart-request — its scope names `crates/generate/ess-synth/tests/http_binding.rs`, which does not exist, instead of `tests/http_binding_rust.rs`, the file whose `#[path]` lines declare every `http_binding/*` module including the new `multipart.rs` (inferred), so waves cannot see that it collides with story:generated-rust-and-go-readers-agree-on-leading-zeros, which cites that file — .engineering/planning/story/the-generated-rust-server-reads-a-multipart-request.md:21, :57; .engineering/planning/story/generated-rust-and-go-readers-agree-on-leading-zeros.md:26; crates/generate/ess-synth/tests/http_binding_rust.rs:14
story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for — one of its scope entries is the word `committed`, taken from "committed suites and golden files of any fixture whose guard has this shape", so the files it would re-pin are not a scope path and nothing was assessed for them — .engineering/planning/story/a-connective-child-masked-by-an-earlier-branch-is-not-asked-for.md:20, :128
story:a-related-row-may-answer-before-input-guards — its scope entry `crates/edge/ess-xtask` (inferred, "typed diagnostic snapshots, hint text") names a whole crate as one string, so waves cannot see that it collides with the 4 children that name `consumer_coverage/macro-guards.json` or `entry-classifications.json` inside that crate — .engineering/planning/story/a-related-row-may-answer-before-input-guards.md:21, :116
story:conformance-reads-a-view-no-row-holds — its scope entry `crates/verify/ess-conformance/src/synthesize/` (inferred, "a sourceless-view module") is a directory, so its module edit collides unseen with the 70 children that name files under it — .engineering/planning/story/conformance-reads-a-view-no-row-holds.md:21, :53

**Round 1, finding by finding:**
- **Fixed:** the Go-closedness pair now has its edge (generated-servers-ignore-undeclared-json-members-and-check-echoed-path-fields.md:15, body :56).
- **Fixed:** the `the` and `synthesis` scope words are gone.
- **Fixed:** all 13 directory findings of round 1 now name files.
- **Escalated decision 1 holds on the evidence:** the epic's "Shared surfaces" section names the coordinator-reconciled files (zero-open-issues-2026-11.md:38-56). The ess-37 story opens `ess/37` and the suite pair, empty, and 49 stories depend on it. No story in the set still has the bytes table, its reader, `CHANGELOG.md` or a generated output as a scope entry; only the ess-37 story itself names `spec-versions.md` and `ess.schema.json`. The 4 bytes-table findings and the 5 `FORMAT_HISTORY` findings are answered by it.
- **Answered:** the 7 active-beside-draft findings are answered by moving the whole set to active together.

**What I read:** 271 non-terminal children (264 draft, 7 active).
- Commands: `aep plan artifact show/list/waves/validate` (validate reports `valid`; draft waves gives 55 waves, 5,223 collision lines, no cycle), every child's frontmatter `scope` and `relations`, `git diff 3bd1b6e9d2`, and `ls`/`git grep` over the tree.
- Surfaces: 265 items have at least one cited entry, 6 have inferred entries only, 0 could not be placed.
- Acceptance reads: 28 acceptance reads of another story in the set traced to their producer. 26 are covered by a depends_on edge (24 direct, 2 through another story). 2 run the other way and are written as "whichever lands second", an acceptance-lane matter noted in round 1. Format-version reads: 49 stories have an edge to the ess-37 story, and 1 has none (finding 1).

**What I could not establish:**
- 18 more stories have a test or fixture directory as a scope entry: `ess-domain/tests/` ×5, `ess-ui-test/tests/` ×3, `ess-ui/tests/` ×3, `ess-conformance/tests/fixtures/` ×3, and one each of `ess-ui-tui/tests/`, `ess-ui-react/tests/`, `ess-gen/tests/` and `ess-synth/tests/`. Two more name the wider `ess-conformance/tests/` (113 children under it) and `ess-compiler/tests/`. I did not raise these. Each body says it adds a new test case or file, and a new file collides with no other story's file. That holds only as long as the implementor does not edit an existing test file there.
- story:an-event-channel-may-be-bound-to-server-sent-events depends on story:a-response-stream-follows-one-row-until-a-terminal-state, a draft outside this set. I did not judge it, because it is outside the set.
- The ess-jsonrpc and ess-websocket stories each create a new crate. Both name the root `Cargo.toml`, so waves orders them. Neither names `Cargo.lock`, and I did not check whether the coordinator treats it as reconciled.
- I did not check collisions inside a single file (below file level), and I did not read the 5,223 file-level collision pairs one by one.

```findings
- file: .engineering/planning/story/a-not-defined-branch-narrows-an-optional-in-later-branches.md
  line: 87
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance reads the format version that admits the narrowing, which story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs opens as `ess/37`, but it has no depends_on edge to that story (directly or through another story) or to story:format-rule-for-relaxations, which its Decisions section points to, so one wave can hold the reader and the story that produces the format"
- file: .engineering/planning/story/the-generated-rust-server-reads-a-multipart-request.md
  line: 21
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "inferred surface: its scope names `crates/generate/ess-synth/tests/http_binding.rs`, which does not exist, instead of `tests/http_binding_rust.rs`, the file whose `#[path]` lines declare every `http_binding/*` module including the new `multipart.rs`, so waves cannot see that it collides with story:generated-rust-and-go-readers-agree-on-leading-zeros, which cites that file"
- file: .engineering/planning/story/a-connective-child-masked-by-an-earlier-branch-is-not-asked-for.md
  line: 20
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "one of its scope entries is the word `committed`, taken from \"committed suites and golden files of any fixture whose guard has this shape\", so the files it would re-pin are not a scope path and nothing was assessed for them"
- file: .engineering/planning/story/a-related-row-may-answer-before-input-guards.md
  line: 21
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "inferred surface: its scope entry `crates/edge/ess-xtask` (\"typed diagnostic snapshots, hint text\") names a whole crate as one string, so waves cannot see that it collides with the 4 children that name `consumer_coverage/macro-guards.json` or `entry-classifications.json` inside that crate"
- file: .engineering/planning/story/conformance-reads-a-view-no-row-holds.md
  line: 21
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "inferred surface: its scope entry `crates/verify/ess-conformance/src/synthesize/` (\"a sourceless-view module\") is a directory, so its module edit collides unseen with the 70 children that name files under it"
```
