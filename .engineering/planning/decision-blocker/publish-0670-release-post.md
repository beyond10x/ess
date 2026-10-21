---
format: aep.planning-md/3
id: decision-blocker:publish-0670-release-post
kind: decision-blocker
status: open
title: Decide whether the 0.67.0 release post (branch blog/0.67.0) is published
relations:
- blocks: story:restore-blog-lag-three
revision: 2
---
# Decide whether the 0.67.0 release post (branch blog/0.67.0) is published

## Decision needed

Whether the release post on branch `blog/0.67.0`
(`website/blog/2026-10-21-1200-absent-values-and-narrowed-externals.md`) goes public. A post is a
public announcement, which needs Timo's word.

## Options

- Publish as written: merge the post and restore `BLOG_LAG` to 3 in the same change.
- Publish after a rewrite: same, with the post edited first.
- Do not publish: the lag constant stays at 4 until a later release post closes the gap.

## Decider

Timo.
