---
format: aep.planning-md/3
id: story:feature-request-460
kind: story
status: draft
title: 'CLI: show which specification formats a build implements, and what each format added'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#460
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-429
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-cli/src/formats.rs
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/specify_formats.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/format_history.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/main.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/format_history.rs
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 3
---
## Outcome
Resolve beyond10x/ess#460: CLI: show which specification formats a build implements, and what each format added.

## Origin
beyond10x/ess#460, filed 2026-10-05; an agent started a new specification on `ess/1` after copying a plugin skill's minimal example, and could learn the newest format only by writing a header that was too high.

## Fit review
1. Need: an author, often an agent with no network, asks the `ess` it is about to run two questions: which `format: ess/N` headers it admits (and which is newest), and what each `ess/N` admits that `ess/N-1` did not. Today the binary answers neither directly. Minimal reproduction, written fresh (`<fit-review scratch>/probe-460/spec/system.yaml`, header `format: ess/99`). On the installed `ess 0.53.0`, `ess specify validate` printed `[unsupported_format_version] system.format: this build implements specification format(s) [1, 2, …, 22], not ess/99` with the hint "upgrade the tooling that reads it" (`<fit-review scratch>/probe-460/validate.out`). That refusal is the only way to get the list, and nothing in the binary says what any version added. The requester proposed (theirs): `ess specify formats`, `--format json`, `--show ess/20`, `--since ess/15`, the same for `ess-inputs/N`, `ess-ui/N` and suite formats, and a refusal hint that names the command.
2. Class: gap in the CLI. No authored document is involved. The facts exist in four places the binary cannot answer from: `SUPPORTED_FORMATS` (`crates/specify/ess-domain/src/system.rs:53-55`), the refusal message (`system.rs:968-981`), the published table (`website/docs/reference/spec-versions.md:38-61`), and the release of each version, which is kept only in `FORMAT_RELEASES` inside the repository's own tooling (`crates/edge/ess-xtask/src/docs.rs:137-159`). That tooling is not part of the shipped `ess`. Starting on `ess/1` is not a defect by itself: the page says "Declare the lowest version that admits every construct the specification uses" (`spec-versions.md:33-36`), and the walkthrough writes the newest format (`crates/edge/ess-cli/tests/tutorial_page.rs:858-869`). The `ess/1` example the agent copied is in the agent-plugins repository, not here.
3. Existing idiom: none offline. Online there is the version-history page. `ess specify toolchain` lists cached releases, not formats (`ess specify toolchain --help`). `ess --version` prints a release, not the formats it implements.
4. Fit: build one catalogue and nothing else.
   - **Catalogue.** `FORMAT_HISTORY` in `crates/specify/ess-domain/src/system.rs`, beside `SUPPORTED_FORMATS`: one row per `ess/N` holding `major`, `release: Option<&str>` (`None` until a release ships it, as in `FORMAT_RELEASES`), `added: &[&str]` and `stricter: &[&str]` (rules that read differently from that version on, such as `returns: true` answering `200` from `ess/22`, `spec-versions.md:61`). A `const` assertion makes its majors equal `SUPPORTED_FORMATS`, so a row that is missing or extra is a build error. The CLI and the documentation both read this one source.
   - **Verb.** `ess specify formats`, under the area that reads `format:`. It is a clap derive variant of `SpecifyCommand` (`crates/edge/ess-cli/src/main.rs:127-197`) and reuses `Format` (text/yaml/json, `main.rs:450-454`). `--since ess/N` lists only later versions. The text output marks the newest.
   - **Page.** The `ess/` table on `spec-versions.md` becomes a generated block between marker lines, rendered by `cargo xtask` from the catalogue and held by `--check`. This follows `cli_reference.rs:1-10` and `diagnostics.rs:1-15` ("each crate … exports a catalogue … `--check` holds the committed page"). The prose paragraphs under the table stay hand-written.
   - **Releases.** The docs lane takes the `ess` family's release from `FORMAT_HISTORY`, and the 22 `("ess", N, …)` rows leave `FORMAT_RELEASES`. `SUPPORTED` (`docs.rs:68-73`) and `crates/edge/ess-xtask/tests/model_enums.rs:216-232` still read `SUPPORTED_FORMATS` by name, unchanged.
   - **Refusal hint.** The hint at `system.rs:976-979` names `ess specify formats`.
   - **Toolchain model.** `models/toolchain/domains/specify.yaml:40-42` already mirrors the admitted set and is not touched.
   - **Item 3 (other families).** Out of scope. Suites, diffs and reports are written by `ess`, which picks their version (`--suite-format`, `website/docs/reference/cli.md:1435`). The few hand-authored families (`ess-inputs/1-2`, `ess-ui/1`, `ess-composition/1-3`, `ess-scenario/1-4`) are listed on the page with their releases. The command's long help names the page for them.
   - **Not adopted.** `--show ess/N`: it is one row of `--format json` filtered, so it would be a second way to select.
5. Second adopter: a CI job in an unrelated repository checks that the pinned `ess` admits the header its specification declares before it bumps the header to use a new construct. It reads `ess specify formats --format json` instead of parsing a refusal. Another: an editor extension offering the newest header on a new `system.yaml`.
6. Cost: one new verb (regenerating `website/docs/reference/cli.md` with `cargo xtask cli-reference`), one public const and row struct in `ess-domain`, and one hint sentence. The `ess/` table on the page turns from hand-kept to generated, and `FORMAT_RELEASES` loses 22 rows. No source or suite format bump, no keyword, no new diagnostic code, and no diff classification. Size M.
7. Alternatives: (a) Change nothing and fix the plugin skill's example only. That is necessary but not sufficient: the binary still cannot answer offline, and the release table stays out of the shipped `ess`. (b) Put the newest format in `ess --version`. That changes a line scripts already parse and still says nothing about what each version added. (c) The requester's full surface, with `--show` and every family. `--show` duplicates a filter. Every family would mean moving `FORMAT_RELEASES` (about 200 rows, including `ess-execution-*` files the binary only writes) into a library, plus prose for each. That is deferred. (d) Derive the history from `changes/*.yaml`. Those are 49 release-feed entries, not one per format, so they were refused as the source. (e) Chosen: `ess/` only, one catalogue feeding the verb, the page table and the docs lane.

## Decisions
accept, redesigned. Build `ess specify formats [--since ess/N] [--format text|yaml|json]` over a new `FORMAT_HISTORY` catalogue in `ess-domain`, with its majors held to `SUPPORTED_FORMATS` at compile time. The `ess/` table of `website/docs/reference/spec-versions.md` is rendered from the catalogue and checked in `projection-check`. The docs lane reads `ess` releases from it, and the unsupported-format hint names the command. Not adopted: `--show` (use `--format json`), and `added` text for families other than `ess/` (the page lists them; deferred). The plugin skill's example is fixed in the agent-plugins repository after this release, because its `agentplugins-check tools` validates commands against the newest release (`AGENTS.md`, "Agent plugin").
No format bump. depends_on story:feature-request-429. That story owns `SUPPORTED_FORMATS`, `FormatVersion::V23`, the `("ess", 23, …)` row of `FORMAT_RELEASES` and the `ess/23` row of `spec-versions.md`, and this story moves all three into the catalogue. After this story lands, the `ess/23` sentences that #450, #452, #458 and #459 report for the row go into the catalogue's `added` list for 23 rather than into the page, and the coordinator regenerates the page.

## Acceptance
- specify_formats_lists_every_supported_format_newest_marked: `ess specify formats` prints one line per major in `SUPPORTED_FORMATS`, ascending, each with its release or `unreleased`, and marks only the highest as newest.
- specify_formats_json_carries_release_and_additions: `--format json` prints an array of `{format, release, newest, added, stricter}`. `format` is `ess/N`, `release` is a version string or `null`, and every row's `added` is non-empty.
- specify_formats_since_lists_only_later_formats: `--since ess/20` lists `ess/21` through the newest and nothing else. `--since ess/0` and `--since x` exit non-zero with clap's usage error.
- format_history_majors_equal_supported_formats: a unit test in `ess-domain` asserts the catalogue's majors equal `SUPPORTED_FORMATS`, in order. The `const` assertion makes a mismatch a build failure.
- spec_versions_ess_table_is_rendered_from_the_catalogue: the xtask `--check` for the page passes on the committed page. It fails when one `added` item is edited in the catalogue but not on the page, and when the page's `ess/` table is edited by hand. It runs in `projection-check` (`Taskfile.yml:265-290`).
- docs_lane_reads_ess_releases_from_the_catalogue: `cargo xtask docs` passes with no `("ess", …)` row in `FORMAT_RELEASES`, and refuses when a catalogue row's release differs from the page's "Introduced in" cell.
- unsupported_format_hint_names_specify_formats: validating `format: ess/99` is refused `unsupported_format_version`, and the hint contains "`ess specify formats`".
- cli_reference_lists_specify_formats: `cargo xtask cli-reference --check` passes with the new verb on `website/docs/reference/cli.md`.
- the_model_lists_every_admitted_specification_format (existing, `model_enums.rs:216-232`) and the_walkthrough_writes_the_newest_format_and_pins_its_toolchain (existing, `tutorial_page.rs:858`) stay green unchanged.

## Scope
- crates/specify/ess-domain/src/system.rs  cited — `SUPPORTED_FORMATS` (53-55) gains the `FORMAT_HISTORY` catalogue beside it; hint at 976-979
- crates/edge/ess-cli/src/main.rs  cited — `SpecifyCommand` (127-197) gains `Formats`; dispatch beside `Toolchain` (1563); `Format` (450-454) reused
- crates/edge/ess-cli/src/formats.rs  inferred — renders the catalogue as text, yaml and json
- crates/edge/ess-cli/tests/specify_formats.rs  inferred — the verb's acceptance cases
- crates/edge/ess-xtask/src/docs.rs  cited — `FORMAT_RELEASES` (137-159) loses its `ess` rows; the lane reads them from the catalogue
- crates/edge/ess-xtask/src/format_history.rs  inferred — renders and checks the page's `ess/` table between markers
- crates/edge/ess-xtask/src/main.rs  inferred — registers the new xtask command
- Taskfile.yml  cited — `projection-check` (265-290) gains the `--check` step
- website/docs/reference/spec-versions.md  cited — `ess/` table (38-61) becomes a generated block
- website/docs/reference/cli.md  cited — generated command sections gain `ess specify formats`
- crates/specify/ess-domain/tests/format_history.rs  inferred — majors equal `SUPPORTED_FORMATS`
