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
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/input_discovery.rs
- confidence: cited
  path: crates/edge/ess-cli/src/requires.rs
- confidence: cited
  path: crates/edge/ess-cli/src/toolchain.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/toolchain.rs
- confidence: cited
  path: website/docs/guides/specify/layout-and-validation.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 10
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

Each case is a test in `crates/edge/ess-cli/tests/toolchain.rs`, run with `cargo test -p ess-cli
--locked --test toolchain`, on the reproduction layout above with a stand-in release cache
(`ESS_TOOLCHAIN_DIR`), so nothing is downloaded; each is red on 0.92.0 except the last.

- `a_pin_below_newer_than_this_ess_is_refused_by_name`: from `project/` with pin 0.93.0,
  `ess specify validate --path spec` exits 1, its stderr names `spec/ess-inputs.yaml`,
  `ESS_TOOLCHAIN=0.93.0` and `cd spec`, and does not contain `b10x upgrade`.
- `a_pin_below_older_than_this_ess_is_refused_by_name`: from `project/` with pin 0.91.0,
  `ess specify validate --path spec` exits 1 with the same three names, instead of today's warning
  and exit 0.
- `toolchain_which_names_a_single_pin_below`: from `project/` with pin 0.91.0,
  `ess toolchain which` prints `ess 0.92.0` and a reason line that names `spec/ess-inputs.yaml`,
  `requires: ess 0.91.0` and `ESS_TOOLCHAIN=0.91.0`.
- `toolchain_which_refuses_two_disagreeing_pins_below`: from `project/` with
  `a/ess-inputs.yaml` pinning 0.91.0 and `b/ess-inputs.yaml` pinning 0.93.0, `ess toolchain which`
  exits 1 and its stderr names both manifests and both releases.
- The tests already in `tests/toolchain.rs` (a pin at or above the working directory) pass
  unchanged.

## Decisions

- A command whose `--path` selects an exact pin that differs from this release, run where no pin is
  at or above the working directory, refuses by name rather than running the pinned release:
  delegation is chosen in `toolchain::delegate` before clap parses any argument, so running the pin
  would need a second argument reader for `--path`, and the refusal gives the exact command that
  runs it.
- The refusal covers an older pin too (today a warning): an exact pin names one release, and
  continuing on another leaves the user believing they run the pinned one.
- A minor-line pin (`requires: ess X.Y`) and a run under `ESS_TOOLCHAIN` keep today's checks.

## Scope

- cited: crates/edge/ess-cli/src/toolchain.rs
- cited: crates/edge/ess-cli/src/input_discovery.rs
- cited: crates/edge/ess-cli/src/requires.rs
- inferred: crates/edge/ess-cli/tests/toolchain.rs
- cited: website/docs/reference/cli.md
- cited: website/docs/guides/specify/layout-and-validation.md

confidence: medium

`toolchain.rs` gains the below-the-directory search beside `choose_unchecked` and the
`Reason::NoManifest` text; `requires.rs` gains the refusal; the two pages describe the pin below
the working directory ("The release a project runs" and the `requires:` paragraph). Shared
surfaces: moves none.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.
