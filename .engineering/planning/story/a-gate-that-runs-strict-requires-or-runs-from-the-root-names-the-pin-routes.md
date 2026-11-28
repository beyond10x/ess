---
format: aep.planning-md/3
id: story:a-gate-that-runs-strict-requires-or-runs-from-the-root-names-the-pin-routes
kind: story
status: draft
title: 'A refusal of a line pin under --strict-requires, and a pin below the working directory, name the routes: an exact pin, cd, ESS_TOOLCHAIN'
summary: The released rules stand (D116, D117); the tool and the pages name the exact pin, cd and ESS_TOOLCHAIN as the routes for a gate on several machines or run from the root
tags:
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-command-runs-the-release-its-specification-pins-from-any-working-directory
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/requires.rs
- confidence: cited
  path: docs/design/specification-requires-release.md
- confidence: cited
  path: website/docs/guides/specify/layout-and-validation.md
revision: 2
---
## Summary

A gate that runs `ess --strict-requires` on machines whose installed `ess` differs meets a repository
with a minor-line pin (`requires: ess 0.98`) as a refusal on every machine whose release is newer than
the line, and a line the older machine cannot serve. The released rule is right and stays: a minor line
names no single release, so a line pin never delegates, and an exact pin is the pin that decides which
release runs. What is missing is that the refusal and the documentation say so. The same holds for a
gate that runs from the repository root with `--path <dir>`: the pin below the working directory is not
followed, and the route is `cd <dir>` or `ESS_TOOLCHAIN`. This story makes the tool and the pages name
those routes.

## Facts (tag 0.99.1, read)

1. An exact pin delegates, a line pin does not: "When it names another release, the running `ess` execs
   that release from its cache ... A minor-line pin or no pin at all runs this release, exactly as
   before" (`crates/edge/ess-cli/src/toolchain.rs:3-9`).
2. A line pin is compared by the release on `PATH` alone: `Required::Line(x, y)` compares major and
   minor only (`crates/edge/ess-cli/src/requires.rs:98-101`); only an exact pin may be unfollowed
   (`:102-108`). A newer release under `--strict-requires` refuses with "`<manifest>` requires ess 0.98
   and this is ess 0.99.1, which is newer, and --strict-requires refuses a newer release"
   (`requires.rs:120-122`); an older release refuses and names `b10x upgrade` (`:116-119`). Neither
   text names the exact pin.
3. The pages say `--strict-requires` "is the spelling for CI" without saying that a gate on several
   machines must pin an exact release: `website/docs/guides/specify/layout-and-validation.md:125-129`
   ("A minor line, `requires: ess 0.34`, leaves the release on your `PATH` to run ... `--strict-requires`
   refuses instead, which is the spelling for CI") and the table at
   `docs/design/specification-requires-release.md:39-45` (the row "newer, with `--strict-requires` |
   refusal, exit 1"). The reporter's count on one host: 30 repositories with exact pins, 9 with line
   pins.
4. The decision is taken (D116, the coordinator, 2026-11-25): the released rule stands; a gate that
   runs `--strict-requires` uses an exact pin; no format key is added.
5. The same shape with an exact pin: a pin in `<dir>/ess-inputs.yaml` does not delegate when `ess`
   runs from the repository root as `ess specify validate --path <dir>`. The walk for the pin is
   upwards from the working directory only, and delegation is chosen before any argument is read, so it
   cannot follow a `--path` (`docs/design/specification-requires-release.md:99-101`, `:76`). `ess`
   refuses, naming the manifest, `cd <dir>` and `ESS_TOOLCHAIN=X.Y.Z`
   (`docs/design/specification-requires-release.md:104-112`). The reporter's consumer sets
   `ESS_TOOLCHAIN` once in its gate and checks it against the pin.

## Acceptance

1. `website/docs/guides/specify/layout-and-validation.md` (the paragraph at lines 125-129) and
   `docs/design/specification-requires-release.md` (the table at lines 39-45) say: a gate that passes
   `--strict-requires` and runs on machines with different installed releases pins an exact release,
   because only an exact pin decides which release runs.
2. The refusal that `--strict-requires` prints for a line pin against a newer installed release names
   both routes: `requires: ess X.Y.Z`, with the newest release of that line the tool knows of only when
   it can name one without a network request (otherwise the form alone), and `ESS_TOOLCHAIN=X.Y.Z` for
   one command. The test `a_line_pin_under_strict_requires_names_the_exact_pin_and_the_toolchain_variable`
   (a case in `ess-cli`'s existing `requires` tests) pins the text, and a second case pins that the
   refusal of an exact pin and the warning without `--strict-requires` keep their words.
3. Not built: delegation for a line pin; reading `--path` before the release is chosen; a new key in
   `ess-inputs/2`.
4. The same guide paragraph names the route for a gate that runs from the repository root with
   `--path <dir>`: `cd <dir>`, or `ESS_TOOLCHAIN=X.Y.Z` set once and equal to the pin; and it says that
   `--path` is not read for the pin. It rests on the design rule in
   `docs/design/specification-requires-release.md`, the paragraph "A pin below the working directory"
   (lines 99-101: "delegation, chosen before any argument is read, cannot follow a `--path`"), and on
   "Choosing" (line 76: "Before the command line is parsed, `ess` decides which release runs").
5. Each test runs within 60 s and each test step within 120 s; the cases start no download and no
   network request.

## Fit review

1. Need: an author whose gate runs `--strict-requires` on a line pin learns from the tool and the
   documentation that the exact pin is the route.
2. Class: documentation and a diagnostic; no rule and no format changes.
3. Covers: item 11 of the twentieth gap report, as relayed by the coordinator.
4. Borders: `story:a-command-runs-the-release-its-specification-pins-from-any-working-directory` (draft; it
   also rewrites the pages that describe `requires:` and the `--strict-requires` help, so the two
   stories share `website/docs/guides/specify/layout-and-validation.md` and
   `docs/design/specification-requires-release.md`), `story:a-pin-below-the-working-directory-is-not-silently-ignored`
   (implemented; the refusal for a pin below the directory). A search of the store found no story on a
   line pin under `--strict-requires`.
5. A second adopter: any repository whose gate runs on a developer machine and on a runner with another
   base release.

## Decisions

- D116: the released rule stands, taken by the coordinator on 2026-11-25; a warning from
  `ess specify validate` for a line pin is not added: a line pin is valid outside strict gates.
- D117: `--path <dir>` is not read for the pin; the documentation names `cd <dir>` and `ESS_TOOLCHAIN`
  as the routes for a root-run gate. Taken by the coordinator on 2026-11-25. Not built: reading `--path`
  before the release is chosen; it would change the design page's choosing rule (the release is chosen
  before the command line is parsed, so an older release never has to parse a newer command line) and
  is the owner's if asked again. The draft
  `story:a-command-runs-the-release-its-specification-pins-from-any-working-directory` asks for that
  change and so stands against D117 until the owner decides.

## Scope

- `crates/edge/ess-cli/src/requires.rs` (the refusal text, 116-122) and its tests
- `website/docs/guides/specify/layout-and-validation.md`, `docs/design/specification-requires-release.md`
