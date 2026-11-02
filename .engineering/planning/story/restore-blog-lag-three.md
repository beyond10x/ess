---
format: aep.planning-md/3
id: story:restore-blog-lag-three
kind: story
status: draft
title: Restore BLOG_LAG to three when the next release post merges
revision: 5
---
# Story: restore-blog-lag-three

## Outcome

The docs lane again refuses a release whose newest release post trails it by more than three
minors, so a skipped post cannot widen the gap a second time unnoticed.

## Context

0.67.0 shipped without its release post: Timo skipped it ("we can skip blog post this time",
`decision-blocker:publish-0670-release-post`). The unpublished draft stays on branch `blog/0.67.0`.
`cargo xtask docs` refused the release at a lag of four (`crates/edge/ess-xtask/src/docs.rs`,
`BLOG_LAG`), so the constant was raised to 4 in the commit that dropped the post.

## Acceptance

- The next release that carries a release post sets `BLOG_LAG` back to 3 and drops the
  doc-comment sentence about 0.67.0.
- `cargo xtask docs` passes on that release's commit.

## Out of Scope

Changing how the lag is measured or which releases need a post. Each post still needs Timo's word
as a public announcement.

## Ambiguities

- `inferable`: the 0.67.0 post is not published; `decision-blocker:publish-0670-release-post`
  records Timo's answer.

## Open Questions

None.

## Reconcile note (2026-11-02)

Store hygiene H0 of wave W1, section 2i: The title and body name 0.67.0 and a lag of four; the constant is 6 since 0.69.0, whose post was skipped as 0.67.0's and 0.68.0's were (crates/edge/ess-xtask/src/docs.rs:39-42), and 6 is intended. Retitled: the acceptance is unchanged, the next release that carries a post sets BLOG_LAG back to 3.
