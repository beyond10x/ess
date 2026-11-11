---
format: aep.planning-md/3
id: review-result:zero-issues-scope-round-1
kind: review-result
status: active
title: Scope critic, round 1, epic zero-open-issues-2026-11
relations:
- reviews: epic:zero-open-issues-2026-11
- reviews: story:a-binding-condition-fed-by-a-subject-payload-is-arranged
- reviews: story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view
- reviews: story:conform-author-and-run-say-what-they-need-before-reading-nothing
- reviews: story:format-gated-construct-is-named-at-validation
- reviews: story:generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects
- reviews: story:verify-diff-decides-what-the-model-settles
- reviews: story:requirement-refs-reach-suite-and-report
- reviews: story:synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields
- reviews: story:a-command-binds-to-json-rpc-and-grpc-beside-http
revision: 1
---
needs-revision
epic:zero-open-issues-2026-11 — The epic promises "Every GitHub issue of `beyond10x/ess` open on 2026-11-09 is resolved", but no child claims two items of #723: B16 (the HTTP adapter unwrapping responses and doing bearer auth with refresh on 401 from each read's `endpoint:`; implemented story:a-generated-ui-calls-the-routes-its-http-binding-serves covers only path, method and parameters) and G25 ("two sections cannot share one card"), and no story declines either; a new ess-ui story or story:a-record-groups-fields-under-headings-and-a-group-may-start-collapsed would naturally take them — .engineering/planning/epic/zero-open-issues-2026-11.md:16 and `gh issue view 723 --repo beyond10x/ess`
epic:zero-open-issues-2026-11 — The epic promises "Every non-terminal story or task whose body describes a defect in shipped behaviour", but story:go-explorer-generates-alphabet-constrained-inputs is a draft tagged `defect` with no `decomposes` edge to this epic and no record that it no longer reproduces; it should get that edge or a 0.92.0 non-reproduction record — .engineering/planning/epic/zero-open-issues-2026-11.md:32
story:a-binding-condition-fed-by-a-subject-payload-is-arranged — The story extends #705's fix to a binding cause condition (`when: {event:, where: event.<member>}`), but #705 reports only the set guard (its § A); no defect draft existed before this story was drafted, so it reaches past both the issue and the defect clause — .engineering/planning/story/a-binding-condition-fed-by-a-subject-payload-is-arranged.md:30
story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view — The body classes itself "Class: gap" and cites no open issue, which is what the epic excludes as "Feature drafts that cite no open issue" — .engineering/planning/story/a-branch-may-leave-its-addressed-record-unchanged-without-a-view.md:90
story:conform-author-and-run-say-what-they-need-before-reading-nothing — The body classes itself "gap (UX). Neither behaviour contradicts a documented rule" and cites no open issue, so it is a feature draft the epic excludes — .engineering/planning/story/conform-author-and-run-say-what-they-need-before-reading-nothing.md:83
story:format-gated-construct-is-named-at-validation — The body says "Class: gap ... so this is not a defect" and cites no open issue, so it is a feature draft the epic excludes — .engineering/planning/story/format-gated-construct-is-named-at-validation.md:81
story:generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects — The body says "Gap, not defect: ... the behaviour is not wrong, it is missing" and cites no open issue, so it is a feature draft the epic excludes — .engineering/planning/story/generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects.md:71
story:verify-diff-decides-what-the-model-settles — The body classes itself "Gap" and cites no open issue, so it is a feature draft the epic excludes — .engineering/planning/story/verify-diff-decides-what-the-model-settles.md:65
story:requirement-refs-reach-suite-and-report — The story carries closed #503 part B, a feature request, and classes it "Gap"; only its diff half (a refs-only edit rated `unclassified-changed`) is a defect, so everything else in it reaches past the epic — .engineering/planning/story/requirement-refs-reach-suite-and-report.md:86
story:synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields — The body classes part A "gap (arrangement through creators only)" and only part B a defect, so part A reaches past the epic's defect clause — .engineering/planning/story/synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields.md:86
story:a-command-binds-to-json-rpc-and-grpc-beside-http — The summary still claims "JSON-RPC 2.0 and gRPC bindings", but #650 and its comment ask for WebSocket and JSON-RPC only, and the body already splits gRPC out; the summary should say JSON-RPC only — .engineering/planning/story/a-command-binds-to-json-rpc-and-grpc-beside-http.md:7

What I read: the epic first, then the 283 children found through the graph, then the 48 open issues and their comments. Commands used: `aep plan artifact show`, `aep plan artifact graph --format json` (to find children, refs and non-child artifacts carrying the same issues), `gh issue list` and `gh issue view` for all 48 issues and the closed ones the children cite (#472, #503, #562, #563, #565, #579), `rg` over the store, and `git show 786d569a9e`. Of the 51 promises a story can claim (48 issues, the defect-draft clause, the triage-form clause and the non-reproduction clause), 49 trace in full. #723 and the defect clause trace only in part. Every issue has at least one child carrying its ref. The open-issue list today is exactly the epic's 48. Every child that carries an issue ref has `## Fit review` and `## Decisions`.

What I could not establish:
- I did not read every one of the 141 children without an issue ref in full; I sampled 25 of them by body and by their own Class line. Only defect-tagged work outside the epic was checked for gaps, so an untagged defect draft outside the epic would not show up.
- I mapped #723's roughly 60 items by item id and keywords, not by reading every story line by line.
- story:generated-behaviour-reads-several-related-rows classes itself only "Generator coverage". Whether that counts as a defect is unclear, so I left it out of the findings.
- Out of my lane (design): the epic says a draft that does not reproduce "leaves this epic", yet the 9 drafts that approval-record:zero-issues-archive-unreproduced-drafts archived or closed still decompose it. Also, story:a-command-binds-to-json-rpc-and-grpc-beside-http keeps its `depends_on` on the protobuf story although its Decisions move that edge to the gRPC story.
- Out of my lane (parallel-safety): the order, one integration branch and one pull request are execution promises that no story can claim.

```findings
- file: .engineering/planning/epic/zero-open-issues-2026-11.md
  line: 16
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "The epic promises \"Every GitHub issue of `beyond10x/ess` open on 2026-11-09 is resolved\", but no child claims two items of #723: B16 (the HTTP adapter unwrapping responses and doing bearer auth with refresh on 401 from each read's `endpoint:`; implemented story:a-generated-ui-calls-the-routes-its-http-binding-serves covers only path, method and parameters) and G25 (\"two sections cannot share one card\"), and no story declines either; a new ess-ui story or story:a-record-groups-fields-under-headings-and-a-group-may-start-collapsed would naturally take them"
- file: .engineering/planning/epic/zero-open-issues-2026-11.md
  line: 32
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "The epic promises \"Every non-terminal story or task whose body describes a defect in shipped behaviour\", but story:go-explorer-generates-alphabet-constrained-inputs is a draft tagged `defect` with no `decomposes` edge to this epic and no record that it no longer reproduces; it should get that edge or a 0.92.0 non-reproduction record"
- file: .engineering/planning/story/a-binding-condition-fed-by-a-subject-payload-is-arranged.md
  line: 30
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The story extends #705's fix to a binding cause condition (`when: {event:, where: event.<member>}`), but #705 reports only the set guard (its § A); no defect draft existed before this story was drafted, so it reaches past both the issue and the defect clause"
- file: .engineering/planning/story/a-branch-may-leave-its-addressed-record-unchanged-without-a-view.md
  line: 90
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The body classes itself \"Class: gap\" and cites no open issue, which is what the epic excludes as \"Feature drafts that cite no open issue\""
- file: .engineering/planning/story/conform-author-and-run-say-what-they-need-before-reading-nothing.md
  line: 83
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The body classes itself \"gap (UX). Neither behaviour contradicts a documented rule\" and cites no open issue, so it is a feature draft the epic excludes"
- file: .engineering/planning/story/format-gated-construct-is-named-at-validation.md
  line: 81
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The body says \"Class: gap ... so this is not a defect\" and cites no open issue, so it is a feature draft the epic excludes"
- file: .engineering/planning/story/generated-rust-behaviours-cover-expressions-distinct-members-and-filtered-effects.md
  line: 71
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The body says \"Gap, not defect: ... the behaviour is not wrong, it is missing\" and cites no open issue, so it is a feature draft the epic excludes"
- file: .engineering/planning/story/verify-diff-decides-what-the-model-settles.md
  line: 65
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The body classes itself \"Gap\" and cites no open issue, so it is a feature draft the epic excludes"
- file: .engineering/planning/story/requirement-refs-reach-suite-and-report.md
  line: 86
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The story carries closed #503 part B, a feature request, and classes it \"Gap\"; only its diff half (a refs-only edit rated `unclassified-changed`) is a defect, so everything else in it reaches past the epic"
- file: .engineering/planning/story/synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields.md
  line: 86
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The body classes part A \"gap (arrangement through creators only)\" and only part B a defect, so part A reaches past the epic's defect clause"
- file: .engineering/planning/story/a-command-binds-to-json-rpc-and-grpc-beside-http.md
  line: 7
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The summary still claims \"JSON-RPC 2.0 and gRPC bindings\", but #650 and its comment ask for WebSocket and JSON-RPC only, and the body already splits gRPC out; the summary should say JSON-RPC only"
```
