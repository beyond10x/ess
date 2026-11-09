---
format: aep.planning-md/3
id: story:a-pin-below-the-working-directory-is-not-silently-ignored
kind: story
status: draft
title: A requires pin below the working directory is found or named, never silently ignored
summary: From the repo root ess stays on the installed release though spec/ess-inputs.yaml pins another (downstream report 2026-11-08 item 2)
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-medium
- zero-open-issues
relations:
- depends_on: story:ess-manages-its-toolchain
- decomposes: epic:zero-open-issues-2026-11
revision: 5
---
## Need

An adopter bumped the `requires:` pin in `spec/ess-inputs.yaml` and ran `ess` from the repository
root. ess looks for a pin only in the working directory and the directories above it
(`input_discovery::nearest_pin`, `crates/edge/ess-cli/src/input_discovery.rs:182`, called from
`toolchain::choose_unchecked`, `crates/edge/ess-cli/src/toolchain.rs:155-218`). From the root it
found none and stayed on the installed release. `ess toolchain which` said only
`no ess-inputs.yaml in … or any directory above it`. Only `ESS_TOOLCHAIN=X.Y.Z` switched it. A user
who bumps `requires:` believes they upgraded, and they did not (a downstream report, 2026-11-08, item 2).

story:ess-manages-its-toolchain and story:specification-declares-its-ess-release (both implemented)
cover a pin found in the working directory or above it, not a pin below the working directory.

Reproduction on 0.92.0, with `project/spec/ess-inputs.yaml` (`format: ess-inputs/2`,
`requires: ess <pin>`, one specification file), run from `project/`:

```text
$ ess toolchain which                       # pin 0.91.0
ess 0.92.0
reason: self: no ess-inputs.yaml in …/project or any directory above it
$ ess specify validate --path spec          # pin 0.91.0 (older than this ess)
warning: spec/ess-inputs.yaml requires ess 0.91.0 and this is ess 0.92.0, which is newer; continuing (--strict-requires refuses)
$ ess specify validate --path spec          # pin 0.93.0 (newer than this ess)
error: spec/ess-inputs.yaml requires ess 0.93.0 and this is ess 0.92.0, which is older; install the required release with `b10x upgrade`, or `/ess:upgrade` in an agent session
$ cd spec && ess toolchain which            # pin 0.91.0
ess 0.91.0
reason: pin: `requires: ess 0.91.0` in …/project/spec/ess-inputs.yaml
```

So a command that reads the manifest through `--path` names the pin (`requires.rs:109`, `:118`), but
it never runs the pinned release. The refusal tells the user to upgrade the installed ess, while
`ess` started inside `spec/` would fetch and run the exact pinned release. `ess toolchain which` from
the root says nothing about the pin below.

## Acceptance

- From a directory with no `ess-inputs.yaml` at or above it, a command whose `--path` selects a
  manifest with an exact `requires: ess X.Y.Z` that differs from this release runs `X.Y.Z`, as it
  would from that manifest's directory. If that is not adopted, the command refuses by name, says
  where the pin is and gives the command that runs it (`ESS_TOOLCHAIN=X.Y.Z ess …` or `cd <dir>`).
  The `b10x upgrade` hint is not shown for an exact pin.
- `ess toolchain which` from such a directory, given a single `ess-inputs.yaml` below it inside the
  same repository, names that pin and how to run against it, instead of only "no ess-inputs.yaml".
- Two pins below the working directory that disagree are refused by name, listing both.
- A pin at or above the working directory keeps today's behaviour (`tests/toolchain.rs` stays green).
- Tests: cases in `crates/edge/ess-cli/tests/toolchain.rs` using a stand-in release cache
  (`ESS_TOOLCHAIN_DIR`), so nothing is downloaded.

## Scope

- cited: `crates/edge/ess-cli/src/toolchain.rs` (`choose_unchecked`, `Reason::NoManifest`)
- cited: `crates/edge/ess-cli/src/input_discovery.rs` (`nearest_pin`)
- cited: `crates/edge/ess-cli/src/requires.rs` (the older/newer messages)
- inferred: `crates/edge/ess-cli/tests/toolchain.rs`
- inferred: `website/docs` (installing and upgrading)

confidence: medium (the pick between delegating and refusing is still open)

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.
