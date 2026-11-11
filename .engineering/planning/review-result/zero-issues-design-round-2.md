---
format: aep.planning-md/3
id: review-result:zero-issues-design-round-2
kind: review-result
status: active
title: Design critic, round 2, epic zero-open-issues-2026-11
relations:
- reviews: epic:zero-open-issues-2026-11
- reviews: story:a-not-defined-branch-narrows-an-optional-in-later-branches
- reviews: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
revision: 1
---
needs-revision

story:a-not-defined-branch-narrows-an-optional-in-later-branches — its acceptance needs "the format version that admits" the narrowing, which is `ess/37` opened by story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs, and no `depends_on` edge runs from it to that story, directly or through another story — .engineering/planning/story/a-not-defined-branch-narrows-an-optional-in-later-branches.md:87, .engineering/planning/story/ess-37-and-conformance-64-65-are-open-for-the-epics-constructs.md:44
story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs — it opens `ess/37` and the suite pair but not the epic's one delta version `ess-diff/24`; seven stories add `ess verify diff` change kinds, only story:requirement-refs-reach-suite-and-report names `/24`, and that story has no edge to this one, so whichever story lands first opens `/24`, the same race round 1 found for `ess/37` (fix: open `ess-diff/24` here, or make requirement-refs the opener with edges to it) — .engineering/planning/story/ess-37-and-conformance-64-65-are-open-for-the-epics-constructs.md:44, .engineering/planning/story/requirement-refs-reach-suite-and-report.md:78, .engineering/planning/story/generated-targets-enforce-a-unique-tuple.md:110, .engineering/planning/story/projections-use-a-unions-declared-content-key.md:72

**Round 1, all nine design findings are fixed:**
- **Finding 1, no owner for the format bumps:** fixed by the format-opening story. 49 stories now have an edge to it.
- **`stored-field-equals`:** the conditional claim is gone (line 185), and it has an edge to the format story.
- **`when-subject`:** has an edge to `navigation-reaches-any-depth-in-view-filters-and-guards`.
- **`a-row-set-upserts`:** has an edge to `when-subject`.
- **`a-value-expression`:** has an edge to `text-functions`.
- **`an-invariant-compares`:** has an edge to `synthesis-offers`.
- **`a-text-operator`:** has an edge to `text-shape-operators-ignore-case`, and ownership is stated at text-shape-operators-ignore-case.md:63.
- **`a-command-route-reads`:** has an edge to `multipart` and says it owns the combination (line 138).
- **`expect-error`:** has an edge to `view-and-error-assertions`.

**The two escalated decisions:** decision 1 holds on the evidence and closes round-1 finding 1, apart from the delta version above. Decision 2 is acceptance-lane, so I did not judge it.

**What I read:** 271 non-terminal children (264 draft, 7 active).
- Commands: `aep plan artifact show` on the epic and the format-opening story, `relations`, `graph --format json`, `validate`, and a grep of every child's Acceptance section.
- Edges: I walked all 392 `depends_on` and 55 `blocks` edges in the store, outside the set as well. 151 run inside the set and 11 point outside it. There is no cycle and no serialising chain; depths are 169 at 1, 47 at 2, 33 at 3, 15 at 4, 6 at 5 and 1 at 6.
- Sibling reads: 29 acceptance lines name a sibling story. 26 are covered by an edge; the other 3 are statements by the owning side or an "adjacent" note, not reads.
- Format reads: 47 stories that name a source or suite format reach the format-opening story by an edge. `a-not-defined-branch` is the only one that does not.

**What I could not establish:**
- I did not confirm in code that each new diff change kind needs a new delta major. I inferred it from the per-feature `*_DELTA_FORMAT` constants (crates/verify/ess-diff/src/change.rs:816-876) and from requirement-refs-reach-suite-and-report.md:75-76.
- `a-not-defined-branch` says at line 110 that whether it is gated at all waits on story:format-rule-for-relaxations, and no edge records that. Its fit review is pending, so I left it as a note.
- Out of my lane (scope/acceptance): `view-and-error-assertions-carry-their-value-shape.md:82` names probes in `tests/suite.rs` that assert `ess-conformance/64` is unsupported. Those probes break when the format-opening story makes `/64` readable, and that file is not in its scope.
- Out of my lane (acceptance wording): four G2 stories still say "At G2's next source format" (a-guard-compares-fields-of-two-named-rows.md:77, a-view-filter-tests-rows-related-to-its-source-row.md:72, an-aggregate-view-reads-through-a-reference.md:76, navigation-reaches-any-depth-in-guards.md:66). Their edges are present.
- `validate` prints "valid". Its warnings are all review-results with no recorded outcome; none of them are my findings.

```findings
- file: .engineering/planning/story/a-not-defined-branch-narrows-an-optional-in-later-branches.md
  line: 87
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its acceptance needs 'the format version that admits' the narrowing, which is `ess/37` opened by story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs, and no `depends_on` edge runs from it to that story, directly or through another story"
- file: .engineering/planning/story/ess-37-and-conformance-64-65-are-open-for-the-epics-constructs.md
  line: 44
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it opens `ess/37` and the suite pair but not the epic's one delta version `ess-diff/24`; seven stories add `ess verify diff` change kinds, only story:requirement-refs-reach-suite-and-report names `/24`, and that story has no edge to this one, so whichever story lands first opens `/24`, the same race round 1 found for `ess/37` (fix: open `ess-diff/24` here, or make requirement-refs the opener with edges to it)"
```
