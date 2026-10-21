---
format: aep.planning-md/3
id: story:restore-blog-lag-three
kind: story
status: draft
title: Restore BLOG_LAG to three when the 0.67.0 release post merges
revision: 2
---
# Story: restore-blog-lag-three

## Outcome

The docs lane again refuses a release whose newest release post trails it by more than three
minors, so a held post cannot widen the gap a second time unnoticed.

## Context

0.67.0 shipped without its release post: a public announcement waits on its own approval. The post
is kept on branch `blog/0.67.0`. `cargo xtask docs` refused the release at a lag of four
(`crates/edge/ess-xtask/src/docs.rs`, `BLOG_LAG`), so the constant was raised to 4 in the commit
that dropped the post from the release.

## Acceptance

- The commit that merges the 0.67.0 post (or a later one) sets `BLOG_LAG` back to 3 and drops the
  doc-comment sentence about the held post.
- `cargo xtask docs` passes on that commit.

## Out of Scope

Changing how the lag is measured or which releases need a post.

## Ambiguities

- `requires-stakeholder-input`: whether the post is published at all; Timo decides.
  `decision-blocker:publish-0670-release-post` blocks this story.

## Open Questions

None.
