---
format: aep.planning-md/3
id: story:a-command-runs-the-release-its-specification-pins-from-any-working-directory
kind: story
status: draft
title: A command runs the release its specification pins from any working directory, and ESS_TOOLCHAIN=self runs the release called
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: README.md
- confidence: cited
  path: crates/edge/ess-cli/src/input_discovery.rs
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/requires.rs
- confidence: cited
  path: crates/edge/ess-cli/src/toolchain.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/requires_release.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/toolchain.rs
- confidence: cited
  path: docs/design/specification-requires-release.md
- confidence: cited
  path: website/docs/guides/commit-generated-files.md
- confidence: cited
  path: website/docs/guides/specify/layout-and-validation.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 14
---
## Outcome

One rule decides which `ess` release runs a command. The pin is read from the specification the
command reads, so `ess specify validate --path spec` runs the same release from the repository
root, from a sibling directory and from inside `spec/`. `ESS_TOOLCHAIN=self` is a documented way to
run the release that was called, with the pin compared as before: an older pin warns, and
`--strict-requires` makes that a refusal. Every page that describes `requires:`, plus `--help` and
the README, says the same rule.

## Need

Two adopters on 0.97.3 reported the dispatcher (`toolchain::delegate`, `requires::check`,
`toolchain::unfollowed_pin` at tag `0.97.3`).

- **An adopter**: a 0.97.1 on `PATH`, `spec/ess-inputs.yaml` pins `ess 0.97.3`, and a task runs
  from the repository root with `--path spec`. The call is refused and nothing is delegated. The
  adopter now sets each task's working directory to `spec/`.
- **A second adopter**: pin `ess 0.51.0`, and a negative-control test expects plain
  `ess specify validate` to warn only. On 0.97.3 it is refused with "did not switch to it". Their
  workaround is `ESS_TOOLCHAIN_DELEGATED=1`, which `--help` and the release README do not name.
  They ask for a documented "run as this release" switch or a README note.

Reproduced with a generic model (`system: shop`, one `system.yaml`, `repo/spec/ess-inputs.yaml`
with the pin). Each case had its own scratch `HOME`, releases came from a local `file://` mirror
(`ESS_TOOLCHAIN_BASE_URL`), and nothing ran on the network. Runs: `model/run.txt`, script
`tools/cases.sh`.

```text
f1a  ess 0.97.1, pin 0.97.3, cwd repo/:  ess specify validate --path spec
error: spec/ess-inputs.yaml requires ess 0.97.3 and this is ess 0.97.1, and no ess-inputs.yaml at or above the working directory pins an exact release, so ess did not switch to it; `cd spec` and run ess there, or run it as `ESS_TOOLCHAIN=0.97.3 ess …`, to use ess 0.97.3
exit 1
f1b  same, cwd repo/spec:  ess specify validate --path .
note: ess 0.97.1 is the dispatcher; delegating to ess 0.97.3 (pin: …)   -> valid, exit 0
c06  ess 0.97.3, pin 0.51.0, cwd repo/:  ess specify validate --path spec
error: … requires ess 0.51.0 and this is ess 0.97.3, … so ess did not switch to it; …   exit 1
c08  same with ess 0.97.1 already cached in that HOME: the same refusal, exit 1
c20  ess 0.97.3, pin 0.51.0, cwd other/ (a sibling):  ess specify validate --path ../repo/spec
warning: ../repo/spec/ess-inputs.yaml requires ess 0.51.0 and this is ess 0.97.3, which is newer; continuing   exit 0
c21  same with pin 0.99.0: error: … which is older; install the required release with `b10x upgrade` …   exit 1
c11  ESS_TOOLCHAIN_DELEGATED=1, case c06:  warning … which is newer; continuing, exit 0
c09  ESS_TOOLCHAIN=0.97.3, case c06:  the same warning, exit 0; with --strict-requires (c10) exit 1
c24  ESS_TOOLCHAIN=self:  error: ESS_TOOLCHAIN=self: `self` is not an exact release `X.Y.Z`   exit 1
```

Findings from the runs and the source:

- The pin is read only from the working directory upwards (`input_discovery::nearest_pin` from
  `current_dir`), before clap parses anything. `--path` is never consulted for the release.
- The refusal in finding 2 does not depend on a missing toolchain. It fires for any exact pin of
  another release in a manifest below the working directory (c08: the release was cached and the
  call was still refused).
- A manifest beside the working directory is compared, not refused (c20, c21). The same
  specification therefore warns, refuses, or points at `b10x upgrade` depending only on the
  directory `ess` was started from. c21 still prints the "upgrade" advice that 0.95.0 removed for
  manifests below the working directory.
- History: 14d5a3e949 (story:a-pin-below-the-working-directory-is-not-silently-ignored, first in
  0.95.0) turned the older-pin warning into a refusal for a manifest below the working directory.
  0.94.0 warned, exit 0 (`h94a`); `--strict-requires` refused (`h94b`). 0.95.0 refuses both
  (`h95a`, `h95b`). In the tagged 0.95.0 `CHANGELOG.md` the entry sits under `### Fixed` with no
  `**Breaking**` mark. The release body that lists it under Breaking ("a `requires:` pin of another
  release refuse cases they accepted") is `release-notes/0.95.0.md`, which was written 2026-11-21 in
  759cfd71d7, eight days after the release.
- Before 0.95.0 the second adopter's negative control passed only because the call came from above
  `spec/`. From inside `spec/`, 0.94.0 already delegated to 0.51.0 and failed to install it
  (`h94c`).

Where the documentation contradicts the runs, or says nothing:

- `ess --help`, `--strict-requires`: "Refuse, rather than warn, where `ess-inputs.yaml` `requires`
  an older `ess` release". This implies that a plain run warns. c06 refuses without the flag.
- `website/docs/reference/formats.md:138`: "a newer one warns once per command, and the global
  `--strict-requires` makes that a refusal". It mentions neither delegation nor the refusal.
- `website/docs/guides/commit-generated-files.md:15`: "an older `ess` refuses it;
  `--strict-requires` refuses a newer one". It also mentions neither.
- `docs/design/specification-requires-release.md:96-112`: "delegation, chosen before any argument
  is read, cannot follow a `--path`"; "A minor line, an override and a manifest elsewhere keep the
  table above". This states finding 1 as a design limit and leaves the gap c20/c21 open.
- `ESS_TOOLCHAIN_DELEGATED` is named only in that design page (`:80`, `:89`), which is never
  published. `--help`, the release README and `website/docs/` do not name it, and nothing names a
  way to run the release called without spelling its version.
- `website/docs/reference/cli.md:102-108` and `guides/specify/layout-and-validation.md:112-115`
  document the refusal correctly.

## Fit review

Decision: **accept, redesigned**. Both findings are covered by one rule. The second adopter's
proposal (a documented "do not dispatch" switch, or a README note) is taken in substance as
`ESS_TOOLCHAIN=self`, not as a new flag or a published `ESS_TOOLCHAIN_DELEGATED`.

1. Need, apart from syntax. An adopter needs the release that a specification pins to run whatever
   directory the command is started from. Today the result depends on the working directory, not
   on the specification. A second adopter needs to run the release they called against an older
   pin and get the documented comparison (warn, or refuse under `--strict-requires`) without
   spelling the release.
2. Class.
   - (a) `--path` not consulted: a **gap** that is documented as a design limit. The earlier story
     rejected a second argument reader for cost (its `## Decisions`, bullet 1).
   - (b) Older-pin refusal: **intended** (same story, bullet 2) and documented in
     `cli.md`/the guide. It is contradicted by `--help`, `formats.md` and
     `commit-generated-files.md`, and was not marked breaking when 0.95.0 shipped.
   - The sibling case c20/c21 is a **defect** against the 0.95.0 rationale: the same pin reached
     through `--path ../spec` still warns or points at `b10x upgrade`.
   - (c) The undocumented variable: a **missing construct**. It is an internal marker, and no public
     "run the release called" switch exists.
3. Can it be expressed today? Partly. `ESS_TOOLCHAIN=<release>` runs a named release from any
   directory (`workarounds/run.txt`, a1-pin, a2-plain, a2-strict), and `cd spec` follows the pin
   (a1-cd). Both work, but the caller must spell the release or change directory, and the second
   form contradicts the adopter's task layout. That is a convenience gap on top of the defect, so
   the idiom alone is not enough.
4. Fit with what exists. It extends `ESS_TOOLCHAIN`, the existing "release for this command"
   override, with one value. It reuses the existing upward walk, starting from the specification
   directory instead of the working directory. No authored surface, no format, no target is
   involved.
5. A second, unrelated adopter: a monorepo task runner that runs `ess` from the root for several
   specification directories that pin different releases. Or a CI job whose checkout root is not
   the specification directory.
6. Cost. No `ess-inputs` or `ess/N` change. Needs a pre-parse reader of `--path`, one new
   `ESS_TOOLCHAIN` value, one new option on `toolchain which`, and documentation in six places.
   Breaking for some callers (see Design).
7. Considered and rejected:
   - change nothing and only document (leaves c20/c21 and finding 1);
   - publish `ESS_TOOLCHAIN_DELEGATED` (an internal marker whose name says something else, removed
     from the environment at startup);
   - a global flag `--no-dispatch` (a CLI-contract change, refused by every older release);
   - delegating at manifest acquisition by exec from `requires::check` (the dispatcher's own clap
     must accept the arguments first, so an older dispatcher refuses a newer flag; and an exec
     mid-command skips destructors of temporary directories).

## Design

Which release runs, first match wins (replaces the "Choosing" table in
`docs/design/specification-requires-release.md`):

1. `ESS_TOOLCHAIN_DELEGATED` set: this release. Internal; documented as such.
2. `ESS_TOOLCHAIN=X.Y.Z`: `X.Y.Z`. `ESS_TOOLCHAIN=self`: this release (new). Any other value
   refuses, as today.
3. The specification directories the command names: every `--path <dir>` and `--path=<dir>`,
   read before clap, as `manages_toolchain` and `asks_version` already scan arguments. For each,
   the nearest `ess-inputs.yaml` at or above `<dir>` is read. With no `--path`, the working
   directory is used, which is today's walk, since `--path` defaults to `.`. Exact pins that name
   different releases refuse, exit 1, naming each manifest and release.
4. An exact pin of another release: exec it from the cache, installing it first (unchanged). A
   release that cannot be installed refuses, exit 1 (unchanged), and the refusal also names
   `ESS_TOOLCHAIN=self`.
5. No exact pin, a minor line, `self`, an override or a delegated run: this release, with the
   comparison table (an older `ess` refuses; a newer one warns; `--strict-requires` refuses).

`unfollowed_pin` keeps only one case: an exact pin of another release in a manifest the dispatcher
did not read (one reached through `--scenarios`, `--service KEY=PATH` or another input flag) is
refused by name wherever it lies. The "below the working directory" condition is removed, and so is
the sibling gap. `ess specify toolchain which` gains `--path <dir>` and answers for that directory.
The `--strict-requires` help text says what it does: "Refuse, rather than warn, where the running
`ess` is newer than an `ess-inputs.yaml` `requires`".

Refusals added: "the specifications this command reads pin different releases (…)". Changed: the
install refusal names `ESS_TOOLCHAIN=self`. No `ValidationCode`. Format consequence: none, because
no persisted field, name or envelope changes (`AGENTS.md` § Determinism and formats). `ess-inputs/2`
is read as before.

Breaking, and for whom:

- A call from above or beside the specification directory with an exact pin of another release now
  runs that release (downloading it on first use). Today it is refused (below) or compared (beside:
  warns, exit 0, or refuses with "upgrade"). Breaking for callers who rely on the warning, as the
  second adopter's negative control would if it ran from a sibling. Offline callers still get exit
  1, from the install refusal.
- A call from inside specification A with `--path B` now runs B's pin, not A's.
- `ESS_TOOLCHAIN=self` is additive. Releases before it refuse the value (c24), so a script that uses
  it needs this release or later on `PATH`.
- CLI contract: `toolchain which --path` and the reworded `--strict-requires` help move the
  generated table in `website/docs/reference/cli.md` (`cargo xtask cli-reference --check` in
  `task ci-lint`). The agent plugin's command check (`agentplugins-check tools`) runs only commands
  the plugin spells. It spells `ess specify toolchain which` and `install --pin`, which are
  unchanged, so it stays green. Its `specifying` skill text ("An older `ess` then refuses, a newer
  one warns") is a follow-up in that repository.
- Announced under `**Breaking**` in `CHANGELOG.md` and in `release-notes/<version>.md`.

## Acceptance

Each item is a test in `crates/edge/ess-cli/tests/toolchain.rs` (`cargo test -p ess-cli --locked
--test toolchain`) with stand-in releases under `ESS_TOOLCHAIN_BASE_URL`, no network, each under
5 s. The two tests of the earlier story that expect a refusal below the working directory are
rewritten to expect delegation.

- `a_path_below_the_working_directory_runs_its_pinned_release`: from `project/` with `spec/`
  pinning the stand-in, `ess specify validate --path spec` exits 0. Stderr has the delegation note
  naming `spec/ess-inputs.yaml`, and the stand-in's marker is on stdout.
- `a_path_beside_the_working_directory_runs_its_pinned_release`: the same from a sibling with
  `--path ../project/spec`, and with `--path=spec` from `project/`.
- `paths_that_pin_different_releases_are_refused`: two `--path` values whose manifests pin
  different releases exit 1, and stderr names both manifests and both releases.
- `self_runs_the_release_called`: `ESS_TOOLCHAIN=self`, pin older than this `ess`, gives a warning
  and exit 0. With `--strict-requires`, exit 1. With a newer pin, exit 1 naming `b10x upgrade`. No
  delegation note in any of the three.
- `an_uninstallable_pin_names_self`: a pin whose release the base URL lacks exits 1, and stderr
  names `ESS_TOOLCHAIN=self`.
- `toolchain_which_answers_for_a_path`: `ess specify toolchain which --path spec` from `project/`
  prints the pinned release with reason `pin: … spec/ess-inputs.yaml`.
- `a_pin_read_through_another_input_is_refused_by_name`: a `--scenarios` directory whose manifest
  pins another release exits 1 naming it, from below and from beside the working directory.
- Documentation: `website/docs/reference/cli.md` (both paragraphs), `guides/specify/
  layout-and-validation.md`, `reference/formats.md:138`, `guides/commit-generated-files.md:15`,
  `README.md` and the design page all state rules 1–5 and name `ESS_TOOLCHAIN=self`.
  `cargo xtask cli-reference --check` and `task test-xtask` pass.

## Decisions

- **D1, how the pin follows the specification.** A: a pre-parse reader of `--path` in
  `toolchain::delegate` (recommended). B: exec from `requires::check` at acquisition. C: keep the
  refusal and document it only.
- **D2, the run-as-called switch.** A: `ESS_TOOLCHAIN=self` (recommended). B: a global flag
  `--no-dispatch`. C: publish `ESS_TOOLCHAIN_DELEGATED`.
- **D3, which manifest a `--path` directory pins from.** A: the nearest at or above it, the walk the
  working directory gets today (recommended). B: only the manifest in that directory, the one input
  selection reads.
- **D4, a followed pin that cannot be installed.** A: refuse as today, naming `ESS_TOOLCHAIN=self`
  (recommended). B: fall back to the release called with a warning. B is rejected because an exact
  pin names one release, and the user would believe it ran.
- **D5, announcement.** A: `**Breaking**` in the changelog and release notes, plus a reply to both
  adopters (recommended). B: under `Changed`.
