---
format: aep.planning-md/3
id: story:a-pin-below-the-working-directory-is-not-silently-ignored
kind: story
status: draft
title: A requires pin below the working directory is found or named, never silently ignored
summary: From the repo root ess stays on the installed release though spec/ess-inputs.yaml pins another (downstream report 2026-11-08 item 2)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- depends_on: story:ess-manages-its-toolchain
revision: 3
---
## Need

An adopter bumped the `requires:` pin in `spec/ess-inputs.yaml` and ran `ess` from the repository
root. ess looks for `ess-inputs.yaml` only in the working directory and the directories above it,
so from the root it found none and stayed on the installed release, saying only
`no ess-inputs.yaml in … or any directory above it`. Only `ESS_TOOLCHAIN=0.91.0` switched it. A user
who bumps `requires:` believes they upgraded and did not (a downstream report, 2026-11-08, item 2).

story:ess-manages-its-toolchain and story:specification-declares-its-ess-release (both implemented)
cover a pin found in the working directory or above, not a pin below the working directory.

## Acceptance

- Run from a directory with no `ess-inputs.yaml` at or above it, ess either finds the pin below it
  (for example a single `spec/ess-inputs.yaml` inside the same repository) and runs that release,
  or refuses by name and says where a pin was found and how to run against it.
- It never silently runs the installed release while the repository pins another one.
- Two pins below the working directory that disagree are refused by name, listing both.
- A pin at or above the working directory keeps today's behaviour.

## Scope

- inferred: the toolchain resolution in `crates/edge/ess-cli/src/toolchain.rs` and `crates/edge/ess-cli/tests/toolchain.rs`;
  website/docs (installing and upgrading).

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.
