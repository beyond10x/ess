---
format: aep.planning-md/1
id: task:release-0-21-0
kind: task
status: active
title: Prepare and publish ESS 0.21.0 from merged source
relations:
- serves: vision:O2
- derived_from: story:cli-presentation-binding
revision: 3
---
## Authorization and outcome

The operator requested updating ESS CHANGELOG.md, tagging the release, and pushing
the tag and main. The subsequent local merge and ESS worktree reconciliation are
complete at abf51add80d0f083c1f701330173cc10d609ad46; this task resumes publication
from that combined source. The additive CLI work and existing Unreleased changes
form the next minor release, 0.21.0.

## Scope and acceptance

- Preserve the existing Unreleased notes and add the missing CLI, recovery,
  ownership, primitive and diagnostic changes supported by current source.
- Align workspace package/dependency versions and Cargo-owned lockfiles, and
  regenerate affected owned projections and the current-source support block.
- Run task check, task site-lab and task site-build on the exact intended release
  source, with the pinned native qualification toolchain and bounded Cargo jobs.
- Commit and tag through clean remote-main Atlas bot authority. Keep all prior
  commits, merge remote main if it advances, and never force-update a public ref.
- Publish main and an annotated bare 0.21.0 tag, then verify their exact remote
  targets. Distinguish pushed source from pending or verified GitHub Release assets.
- Preserve gate evidence outside disposable managed trees, publish recovery,
  and finish/review exact-id cleanup. The separately unfinished Atlas foundation
  verification tree remains owned by that run.

This is release preparation and verification, not new provider, MCP or production
CLI implementation. It does not authorize Atlas catalog changes, consumer
promotion or documentation deployment. One bounded release task is selected;
there is no new multi-story decomposition to send to a critic panel.
