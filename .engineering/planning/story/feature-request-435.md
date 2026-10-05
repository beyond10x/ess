---
format: aep.planning-md/3
id: story:feature-request-435
kind: story
status: active
title: 'Guidance: which generated files a brownfield repository commits, and why the diff is large'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#435
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:ess-generate-check
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/generate_check.rs
- confidence: cited
  path: website/docs/concepts/test-pyramid.md
- confidence: inferred
  path: website/docs/guides/commit-generated-files.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/sidebars.ts
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:27Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:27Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome
Resolve beyond10x/ess#435: Guidance: which generated files a brownfield repository commits, and why the diff is large.

## Origin
beyond10x/ess#435, filed 2026-10-05; reported 2026-10-02 by an adopter who applied ESS and the AEP planning store to a small change in an existing repository. The change produced a large diff of JSON, Markdown and YAML files, and the adopter asked which of them belong in version control.

## Fit review
1. Need: a list, per artifact, of whether it is committed, whether it is regenerated, and how to keep the diff small. The requester's list is: specification sources, compiled IR, suites, generated projections, planning-store records, evidence. The diff is large because generation without `--kind` writes all five projections. For `examples/billing` that is 48 artifacts, including `site/assets/mermaid.min.js` at 3,572,909 bytes (`ess generate --path examples/billing`, ess 0.52.0).
2. Class: gap in documentation, plus one documentation defect. The defect: "Drift-checking in CI" tells adopters to run `cargo xtask generate --check` (`website/docs/guides/generate-artifacts.md:786-797`). The CLI reference calls that command "repository-only" (`website/docs/reference/cli.md:235`), and `ess generate` has no `--check` (`website/docs/reference/cli.md:673` option table). An adopter has no such command.
3. Existing idiom: the answers exist but are scattered across pages.
   - Generated output is deterministic, so it "can be committed, reviewed and drift-checked like source" (`generate-artifacts.md:9-11`).
   - `.ess-output` carries ownership, and a regeneration that changes nothing writes nothing (`generate-artifacts.md:52-57`, `:92-96`).
   - Commit the synthesized Go/TypeScript runner (`website/docs/concepts/test-pyramid.md:108-119`).
   - A committed suite is run with `--suite` (`cli.md:72`).
   - The toolchain pin lives in `ess-inputs.yaml` (`website/docs/start/install.md:115-121`).
   - Compiling twice is byte-identical (`website/docs/concepts/ess.md:136-138`), so compiled IR can always be rebuilt.
   - Planning-store records and evidence are "all plain files you commit" (AEP `website/docs/concepts/planning-store.md:12-18`). AEP owns that answer, and ESS has no AEP dependency (ESS `AGENTS.md`, Boundaries).
   - The drift check an adopter can run today: regenerate into the committed root, then fail CI on a dirty tree in the adopter's own CI. Whether a no-change run leaves the tree clean is inferred from `generate-artifacts.md:95`, not run.
4. Fit: the page is hand-written guide prose, which the organization docs rule allows. It must link to the AEP page for the planning-store rows and not restate them. It must not duplicate `test-pyramid.md`'s runner section; it links to it. Fixing `generate-artifacts.md:786-797` keeps one statement of the drift check.
5. Second adopter: any team adding ESS to an existing service with a published OpenAPI contract. It commits `openapi/` because clients read it, and it does not want `site/` assets in its history. The question is general.
6. Cost: one new guide page and one sidebar entry. The `generate-artifacts.md` drift section is rewritten. No format, CLI or diagnostic change.
7. Alternatives: (a) change nothing; refused, the page tells adopters to run a command they lack. (b) add `--check` to `ess generate`, as its siblings already have (`ess generate cli`, `schema typescript`, `realization generate`, `ui docs`; `cli.md:124`, `:566`, `:710`, `:994`, `:1024`, `:1699`). This is the better long-term drift check, and it is new CLI surface. Coordinator decision 2026-10-05: it is built, as its own story `ess-generate-check`, which this guide depends on. (c) a new guide that documents today's behaviour; chosen.

## Decisions
Accept, redesigned. A new guide, `website/docs/guides/commit-generated-files.md` ("Commit what a reviewer reads, regenerate the rest"), listed in `website/sidebars.ts` under Guides after `guides/generate-artifacts`. It has one table: artifact | commit? | regenerated by | drift check.
- Sources, `ess-inputs.yaml` with its pin, authored scenarios and `ess-known-failures/1`: committed.
- Compiled IR, run reports, histories and mutation output: not committed; regenerated or rerun.
- Projections: commit only the `--kind` that consumers read, with `.ess-output`. `site/` and `docs/` are built in CI for publication.
- Synthesized runner and synthesized code: committed when linked (link to `test-pyramid.md`).
- Planning store and evidence: link to AEP.

A section "Why the first diff is large" gives the measured default inventory and the `--kind` remedy. Rewrite `generate-artifacts.md` "Drift-checking in CI" so it shows `ess generate --path <spec> --out <root> --check`, and say that `cargo xtask generate --check` is this repository's own. That flag is built by story `ess-generate-check`; this story depends on it (edge recorded), so the page never shows a command the released binary lacks. Changed from the request: the planning-store rows point to AEP rather than being answered here. No format bump.

## Acceptance
- site_navigation: `crates/edge/ess-xtask/tests/site_navigation.rs` passes with the new page in the sidebar and every link landing on a page and heading. The heading "Drift-checking in CI" keeps its text, because its fragment `#drift-checking-in-ci` is published. No page in the tree links to it today (grep of `website`, `docs`, `crates`).
- drift_section_shows_ess_generate_check: the "Drift-checking in CI" section of `website/docs/guides/generate-artifacts.md` contains `ess generate` with `--check`, and contains `cargo xtask` only in a sentence that says it is this repository's own. Checked by a case in `crates/edge/ess-cli/tests/generate_check.rs` (created by `ess-generate-check`; this story adds the case), which reads the section.
- commit_generated_files_page_states_the_table: `website/docs/guides/commit-generated-files.md` exists with the title "Commit what a reviewer reads, regenerate the rest" and one table whose header is `artifact | commit? | regenerated by | drift check`. Its rows name "`ess-inputs.yaml`", "authored scenarios", "`ess-known-failures/1`", "compiled IR", "`.ess-output`", "`--kind`" and "synthesized runner", and the page links `concepts/test-pyramid.md`. It has the heading `## Why the first diff is large`, containing "`--kind`". A case of that name in `crates/edge/ess-cli/tests/generate_check.rs` reads the page and fails naming the missing page, heading, column or phrase.
- default_inventory_is_recorded_from_the_binary: the `## Why the first diff is large` section states the number of files `ess generate --path examples/billing` writes by default, as "N files". A case of that name in `crates/edge/ess-cli/tests/generate_check.rs` runs the test binary's `ess generate --path examples/billing --out <temporary dir>`, counts the files written and fails when the count differs from the page's N.
- No adopter-facing page tells an adopter to run `cargo xtask`. Grep `website/docs` for `cargo xtask` outside sentences that say it is this repository's own.
- `task site-build` succeeds with no broken-link or broken-anchor warning.

## Scope
- website/docs/guides/commit-generated-files.md  inferred — new guide page
- website/sidebars.ts  cited — Guides category, :33-78
- website/docs/guides/generate-artifacts.md  cited — :786-797 drift section names a repository-only command; rewritten to show `ess generate --check`
- website/docs/concepts/test-pyramid.md  cited — :108 "Commit the generated runner" gains a link to the new page
- crates/edge/ess-cli/tests/generate_check.rs  inferred — adds `drift_section_shows_ess_generate_check`, `commit_generated_files_page_states_the_table` and `default_inventory_is_recorded_from_the_binary` to the file `ess-generate-check` creates
