---
format: aep.planning-md/3
id: review-result:ui-composition-acceptance-round-1
kind: review-result
status: active
title: 'Acceptance critic, round 1: list-of-records and union form fields'
relations:
- reviews: story:a-form-field-edits-a-list-of-records
- reviews: story:a-form-field-over-a-union-draws-the-chosen-variants-fields
revision: 1
---
needs-revision
story:a-form-field-over-a-union-draws-the-chosen-variants-fields — the first acceptance bullet submits `{<tag>: <variant>, <content>: {…}}` without naming the content key, so a reviewer cannot tell which key the test must read, and the Fit review leaves that key open — .engineering/planning/story/a-form-field-over-a-union-draws-the-chosen-variants-fields.md:112

**What I read:** 2 of 2 ids. I ran `aep plan artifact show` on each in the ess-w091-int tree. I also read `kinds`, `lifecycle story`, the sibling `story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`, `docs/design/union-unit-variants.md`, `website/docs/reference/ess-ui-test.md`, and `crates/ui/ess-ui-check/src/model.rs:2037-2068`.

**What I could not establish:**
- Whether `ess ui test` can drive add, remove and move on a list. That is feasibility, outside my lane, and the "in an `ess ui test` case" bullets are observable as written.
- On the guard story as a missing edge:
  - Out of my lane (design): P2's schema, check and renderers do not need the guard story. The content key is today fixed by `ess_gen::schema::union_content_key` as `value`, or `content` when the tag is `value` (`docs/design/union-unit-variants.md:57`). Guard reads of a union tag are a separate concern.
  - In my lane: the acceptance bullet should pin the key to that fixed rule. Today it would be silently invalidated if the guard story later adds `content:`.
- Not flagged: the P1 acceptance is checkable and has no gap. Its bullets are multi-clause, and I judged that the store's norm, as in the guard story.

```findings
- file: .engineering/planning/story/a-form-field-over-a-union-draws-the-chosen-variants-fields.md
  line: 112
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the first acceptance bullet submits `{<tag>: <variant>, <content>: {…}}` without naming the content key, so a reviewer cannot tell which key the test must read, and the Fit review leaves that key open"
```
