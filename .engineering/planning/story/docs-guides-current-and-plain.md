---
format: aep.planning-md/3
id: story:docs-guides-current-and-plain
kind: story
status: draft
title: The guides and CLI reference name current releases and read without internal shorthand
relations:
- decomposes: epic:public-docs-overhaul
scope:
- confidence: cited
  path: website/docs/guides/synthesize.md
- confidence: cited
  path: website/docs/guides/track-change.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 2
---
## What

The guides and the CLI reference stop naming 0.27.0 as current, stop calling shipped CLI bindings
unreleased, drop the internal deferral note in the synthesis guide, move the Binary64 digression out
of "Validate early", and state directory input selection without suite-version shorthand.

## Acceptance

- `grep -rn '0\.27\.0' website/docs/guides website/docs/reference/cli.md` prints nothing.
- `guides/synthesize.md` names no wave, W-number or `docs/plan` path.
- `cargo xtask docs` and `task site-build` pass.

## Scope

- website/docs/guides/write-a-specification.md
- website/docs/guides/synthesize.md
- website/docs/guides/track-change.md
- website/docs/reference/cli.md
