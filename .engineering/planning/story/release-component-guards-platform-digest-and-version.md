---
format: aep.planning-md/3
id: story:release-component-guards-platform-digest-and-version
kind: story
status: draft
title: Component release refuses an index without linux/amd64 and a malformed version
revision: 2
---
# Story: release-component-guards-platform-digest-and-version

## Outcome

A component release refuses an image index without a `linux/amd64` manifest and a version that is
not `MAJOR.MINOR.PATCH`, instead of recording a wrong platform digest or a malformed version.

## Context

Review 2026-10-21, findings 4 and 7, in `.github/actions/release-component/release.sh`:

- When `imagetools inspect --raw` returns an index (`has("manifests")`) with no `linux/amd64`
  entry, `image_platform_digest` is empty and falls back to the index digest, which is then recorded
  as `platforms."linux/amd64"`. The fallback is right only for a single-platform manifest.
- The version guard `*.*.*` admits any string with two dots.

## Acceptance

- An index without a `linux/amd64` manifest exits non-zero naming the image; a single manifest
  still records its own digest.
- `COMPONENT_VERSION` must match `^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$`; anything else exits 2.

## Out of Scope

Other platforms.

## Ambiguities

None.

## Open Questions

None.
