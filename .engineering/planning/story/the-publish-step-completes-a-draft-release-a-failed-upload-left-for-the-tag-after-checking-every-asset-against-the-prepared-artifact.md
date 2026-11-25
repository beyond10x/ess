---
format: aep.planning-md/3
id: story:the-publish-step-completes-a-draft-release-a-failed-upload-left-for-the-tag-after-checking-every-asset-against-the-prepared-artifact
kind: story
status: draft
title: The publish step completes a draft release a failed upload left for the tag, after it has checked every asset against the prepared artifact
summary: Publish run 45651358803 for 0.99.4 failed on an asset that already existed in a draft of the tag; gh release create cannot succeed while a draft exists, so a re-run repairs nothing
tags:
- ci
- release
relations:
- serves: vision:O2
- informed_by: story:bot-release-publication
revision: 1
---
## Context

On 2026-11-25 the publish run 45651358803 for tag 0.99.4 failed in its step "Publish as the bot"
with `HTTP 422 ... ReleaseAsset.name already exists` on the archive
`ess-0.99.4-aarch64-unknown-linux-gnu.tar.gz`, and then `cleaning up draft failed: HTTP 500`.

- The failed run left a draft release for the tag. It held all four archives and `SHA256SUMS`, and
  each asset's digest was equal to the prepared artifact's. Its body had one extra trailing empty
  line against `notes.md`.
- The step is in `.github/workflows/publish.yml` (read at `origin/main`, lines 105 to 125). Lines
  112 to 115 exit 0 when the release of the tag is already public. Otherwise lines 121 to 125 call
  `gh release create` with the tag, the notes file and the five assets. That call cannot succeed
  while a draft of the tag exists, so a re-run of the failed publish repairs nothing: it fails the
  same way, on the first asset the draft already holds.
- The step "Read the release back" (lines 127 to 140) checks author, draft state and the count of
  five assets, and runs only after a successful publish.
- The draft was reconciled by hand under the rule of `AGENTS.md` ("create or reconcile a draft
  Release on that exact tag ... verify every remote asset, then make the draft public"): the
  digests were checked against the run's artifact, the Linux x86_64 binary was smoke-run, and the
  draft was made public through the bot's wrapper.

## Outcome

When a draft of the tag exists, the publish step compares each of the five expected assets (the
four archives and `SHA256SUMS`) with the prepared artifact by SHA-256, uploads a missing one, and
publishes the draft when all five match. A re-run of a failed publish ends in the public release.
A release that is already public is left as it is, as today.

## Acceptance

A test in `crates/edge/ess-xtask/tests/ci_lanes.rs` runs the step against canned API answers and
fails when any case publishes unverified bytes. The cases:

1. no release and no draft of the tag: the release is created with the five assets;
2. a draft with all five assets, each matching the prepared artifact: the draft is published;
3. a draft with one asset missing: the missing asset is uploaded, then the draft is published;
4. a draft with one asset whose digest differs from the prepared artifact: the step follows the
   pick of PB-D1 and publishes nothing from that draft until the asset is settled;
5. a release that is already public: nothing is changed.

## Decisions

- PB-D1 (open): a draft asset whose digest differs from the prepared artifact.
  - A: the step stops red and names the asset; a person decides (fails closed; costs a manual step
    in the rare case).
  - B: the step deletes that asset and uploads the prepared one (no manual step; the step then
    writes to a release that something else touched).
  - Recommended: A.
- PB-D2 (open): the release body of an existing draft.
  - A: it is rewritten from `notes.md` (the published body is always the prepared one).
  - B: it is left (keeps a hand edit; the body can differ from `notes.md`, as the one-line
    difference of 0.99.4's draft did).
  - Recommended: A.

## Scope

Cited: `.github/workflows/publish.yml` (the step "Publish as the bot") and
`crates/edge/ess-xtask/tests/ci_lanes.rs` (the test of the workflow's steps). `AGENTS.md` describes
the manual reconciliation of a draft and says the step reconciles it once this lands. The
preparation workflow `release.yml` and the bot's read-back step are not changed.
