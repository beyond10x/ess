---
format: aep.planning-md/3
id: review-result:zero-issues-scope-round-2
kind: review-result
status: active
title: Scope critic, round 2, epic zero-open-issues-2026-11
relations:
- reviews: epic:zero-open-issues-2026-11
- reviews: story:synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields
revision: 1
---
needs-revision

epic:zero-open-issues-2026-11 — The epic promises "Every non-terminal story or task whose body describes a defect in shipped behaviour". But story:a-list-input-key-conflict-is-refused-after-the-existence-check is a draft whose body classes its part 2 as a "defect (synthesis witness for a pair over one list)": a nested `exists` guard validates and gets no scenario (ESS-SYNTH-003). It has no `decomposes` edge to this epic, no epic child claims that witness, and nothing records that it no longer reproduces. Part 2 should be split into an epic child, or get a 0.92.0 non-reproduction record. — .engineering/planning/epic/zero-open-issues-2026-11.md:32 and .engineering/planning/story/a-list-input-key-conflict-is-refused-after-the-existence-check.md:57

story:synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields — The title ("over optional and state-filtered fields") and the summary ("ESS-SYNTH-017 for where: on Optional fields ... items 20, 24b") still claim item 20 as an outcome. The body says "Not a defect: item 20" and "needs no change". So the title and summary should name item 24b, the state-filtered counts, only. — .engineering/planning/story/synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields.md:6-7 against :54 and :85

**Round-1 findings, all 11 closed:**
- **#723 B16** is now taken by the new story:a-generated-ui-reads-the-answers-its-http-binding-declares. Its Fit review and Decisions are complete, and it names its omissions ("the terminal gets no renewal").
- **#723 G25** has been merged into story:a-record-groups-fields-under-headings-and-a-group-may-start-collapsed (lines 7, 63, 226).
- **go-explorer** now has the epic edge. Its reproduction at 4ab2ea1449 counts as 0.92.0: `explore.go` is unchanged since the `0.92.0` tag.
- **Six gap stories** no longer decompose the epic: a-binding-condition, a-branch, conform-author, format-gated, generated-rust-behaviours and verify-diff.
- **requirement-refs** is narrowed to the diff defect, and its feature half went to a new draft outside the epic.
- **synthesis-witnesses** split item 22 off into a new backlog draft outside the epic.
- **JSON-RPC story**: its summary now says JSON-RPC only.

**Coordinator decisions:** I checked both escalated decisions against the evidence and do not re-raise either.

**What I read:**
- The epic first. I extracted 51 promises: 48 issues, the defect-draft clause, the triage-form clause and the non-reproduction clause. 50 trace to items; the defect clause traces only in part.
- The round-1 scope result.
- The graph: 282 children, all 48 issues carrying refs, `#736` implemented and closed on 2026-11-09.
- The three new story files, the two narrowed stories and the go-explorer story, read in full.
- The title, summary, ref and edge changes across all 135 changed stories since 3bd1b6e9d2. I skimmed the five largest diffs, and in the case of story:the-rust-crate-layout-is-rustfmt-and-clippy-clean, its added workspace layout is the same defect from the same emitter.
- A body scan of open drafts outside the epic for the word "defect", and `gh issue list` for the open issues and for issues created since 2026-11-08.
- Commands: `aep plan artifact show`, `aep plan artifact graph --format json`, `git diff 3bd1b6e9d2 HEAD`, `git log 0.92.0..4ab2ea1449`, `gh issue list`, `gh issue view 723`/`736`, `grep`.

**What I could not establish:**
- A commit (b58772deb0) landed while I was reading. The tree is now clean, so I read HEAD, not uncommitted changes.
- I did not read all 135 revised acceptance sections in full. I checked them only for changes to title, summary, ref and edges, plus the five largest diffs.
- The defect scan outside the epic was a keyword pass. A body that describes a defect without the word "defect" would not show up.
- I left out task:consumer-accounting-aggregate-reference-closure ("candidate defect awaiting an executable reproduction") and task:consumer-accounting-nested-execution. Both describe defects in internal accounting mechanism work, and I could not tell whether that counts as shipped behaviour.
- The five gap stories that left the epic still carry the `defect` tag. The epic's clause goes by body, not tag, so I did not raise it.
- story:a-binding-condition-fed-by-a-subject-payload-is-arranged keeps its `#705` ref outside the epic, with nothing in its body saying it left.
- Out of my lane (design): 19 epic children still carry `## Schedule` "Backlog: not waved". That reads against the epic acceptance "Every story decomposing this epic is `implemented` or `archived`" and against moving every story to `active`. Example: .engineering/planning/story/a-duration-member-is-refused-as-its-format-refuses.md:159.

```findings
- file: .engineering/planning/epic/zero-open-issues-2026-11.md
  line: 32
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "The epic promises \"Every non-terminal story or task whose body describes a defect in shipped behaviour\", but story:a-list-input-key-conflict-is-refused-after-the-existence-check is a draft whose body classes its part 2 a \"defect (synthesis witness for a pair over one list)\" (a nested `exists` guard validates and gets no scenario, ESS-SYNTH-003), has no `decomposes` edge to this epic, no epic child claims that witness, and nothing records it no longer reproducing; part 2 should become an epic child or get a 0.92.0 non-reproduction record"
- file: .engineering/planning/story/synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields.md
  line: 6
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The title (\"over optional and state-filtered fields\") and summary (\"ESS-SYNTH-017 for where: on Optional fields ... items 20, 24b\") still claim item 20 as an outcome, but the body says \"Not a defect: item 20\" and \"needs no change\", so the title and summary should name item 24b, the state-filtered counts, only"
```
