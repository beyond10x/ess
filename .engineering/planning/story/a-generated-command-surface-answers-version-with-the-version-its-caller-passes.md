---
format: aep.planning-md/3
id: story:a-generated-command-surface-answers-version-with-the-version-its-caller-passes
kind: story
status: draft
title: A generated command surface answers --version with the version its caller passes, lists the flag in help and completions, and leaves the existing entry point unchanged
summary: The generated CLI answers --version with a cli_parse refusal though the binding reserves the flag; the version is the caller's, so it arrives as an argument of a second entry point beside the existing one (CV-D1 default A)
tags:
- external-request
- gap-reports-2026-11-24
- generated-cli
relations:
- serves: vision:O2
- informed_by: story:cli-presentation-binding
- informed_by: story:cli-parse-refusal-names-the-declared-arguments
revision: 1
---
## Summary

A generated command surface (`ess generate cli`, binding `ess-cli/1` and `ess-cli/2`) answers
`--version` with a `cli_parse` refusal and exit 2, though the binding reserves the flag name
`version`. Reported on 0.98.0 by a downstream application; main at 0.99.0 holds the same code. The
caller of the generated entry point is the only place that knows the version it ships (its own
package version, set by its release commit), and the generated crate cannot know it: its package
version is the generator's constant `0.0.0`, and a version written into the binding would tie every
release to a binding edit and a regeneration. So the version has to arrive as an argument of the
generated surface, and the surface answers `--version` when it has one.

## Reproduction (read; the refusal was observed by the reporter on 0.98.0)

Any valid binding. `runtime::command` builds `clap::Command::new(plan.binary.clone())` with no
`.version(..)` (`crates/generate/ess-cli-project/src/runtime.rs:257`), so clap rejects `--version`
as an unknown argument and `runtime::run` answers it as `cli_parse`, exit 2. `run` already maps
clap's `DisplayVersion` to exit 0 with the rendering on stdout (runtime.rs, the `try_get_matches_from`
arm of `run`), so the answer path exists; only the version string and the flag are missing. The
generated `src/lib.rs` template (`crates/generate/ess-cli-project/src/lib.rs:21`) wraps it as
`run(args, sources, handler, dynamic)`, and the generated `Cargo.toml` writes `version = "0.0.0"`.

## Acceptance

1. A caller that passes a version string to the generated surface gets, for `<binary> --version`,
   stdout `<binary> <version>` and a newline, stderr empty, exit 0, with no handler call and no
   source acquired; the flag is `--version` only, no `-V`.
2. The help of the root command lists `--version`; the generated completions file lists it.
3. A caller of the existing, unversioned entry point (an already generated crate's caller) compiles
   and behaves as before: `--version` stays a `cli_parse` refusal there (CV-D1 A).
4. `--version` answers the way help does (stdout, exit 0) also when the selected output mode is
   JSON; the flag belongs to the root command only.
5. A test of `ess-cli-project` pins 1 to 4 on a generic binding, once with a version and once
   without; the committed bytes of the generated crate, completions included, move with the change
   and the pin names the cause.
6. `docs/design/cli-presentation-binding.md` states the flag, its form and who supplies the version.

## Fit review

1. **Need.** A consumer's release check reads the version from the binary; today the consumer
   answers `--version` in its own `main.rs` ahead of the generated runtime and removes it when the
   generator emits it.
2. **Class.** A gap in a generated surface; not a defect: the design page reserves the flag name
   and says nothing about answering it. No conflict with a design page.
3. **Expressible today.** Only outside the surface, by answering the flag in the caller before the
   generated `run` (the reporter's workaround).
4. **Proposed by the reporter, as evidence of the need.** An added argument on the generated entry
   point. That changes the signature every caller of every generated crate compiles against, so the
   shape is decision CV-D1, not taken here.
5. **Rejected for the version's source.** The generated package version (the generator writes
   `0.0.0`; the consumer's release commit never edits the excluded generated crate) and a binding
   key (each release would edit the binding and run the generator; the drift check would tie a bump
   to a generation).
6. **Borders.** `story:cli-presentation-binding` (the binding and its reserved names),
   `story:cli-parse-refusal-names-the-declared-arguments` (the same refusal path),
   `story:components-sharing-a-cli-binary-generate-one-tree-with-every-group` (one binary, one
   version: the argument is per entry point, not per group).
7. **A second adopter.** Any generated CLI shipped as a binary whose version lives in the shipping
   package.
8. **Cost.** Every regeneration of an existing crate changes `src/runtime.rs`, `src/lib.rs` and
   the completions file, so a consumer's drift check fires once at its next regeneration; under A
   no caller is edited.

## Decisions

- CV-D1, default A: how the version reaches the surface. A adds a second entry point beside the
  existing (`command_versioned(version)` and `run_versioned(args, version, sources, handler,
  dynamic)` in the generated crate, the same pair on `runtime`); the existing `command` and `run`
  keep their signatures and behaviour, so no caller of an already generated crate breaks. B adds a
  `version` parameter to `run` and `command`: every caller of an already generated crate fails to
  compile when the crate is regenerated. C adds an options value (`run_with(args, &Options { version,
  .. }, ..)`): additive like A, one more public type, room for later options. Recommended A. Not
  architecture and not a security boundary; taken on default if unanswered.
- CV-D2, default A: the answer's form. A pins `<binary> <version>` plus newline on stdout, exit 0,
  the long flag only, by a test (clap's own rendering with `-V` disabled). B leaves clap's default
  rendering, `-V` included, unpinned. Recommended A.
- CV-D3, default A: the completions file lists `--version`. A: the generator builds its completions
  from the versioned command with a fixed placeholder, so the file names the flag and holds no
  version. B: the file stays as today and lacks the flag. Recommended A.
- Accept as reported: the need, with no new binding syntax.

## Scope

- `crates/generate/ess-cli-project/src/runtime.rs` (`command` 255-257, `run` 758), `src/lib.rs`
  (the `src/lib.rs` template, line 21, and the completions generation beside it), its tests
  (`crates/generate/ess-cli-project/tests/`), `docs/design/cli-presentation-binding.md`
  (reserved names, line 73), `CHANGELOG.md`; the committed byte pins and consumer-coverage entries
  that hold the generated crate (`crates/edge/ess-xtask/src/consumer_coverage/`), moved with the
  change and named.
