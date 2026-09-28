---
format: aep.planning-md/3
id: epic:public-docs-overhaul
kind: epic
status: draft
title: Public documentation takes a first-time adopter from zero to a green conformance run
summary: 'Audit and overhaul of the website docs and README against 0.38.0: stale claims, missing adopter path, navigation, jargon.'
revision: 1
---
## Outcome

A first-time adopter can go from nothing to a validated specification of their own, a generated
artifact and a green conformance run against their own implementation, using only the published
documentation. Every version, command and format the public pages name matches ESS 0.38.0 and the
current `main`.

## Audit (2026-09-28, against 0.38.0 and `ess --help` of this tree)

| Category | Count | Examples |
|---|---|---|
| Stale release or version claim | 6 | `README.md:62` installs 0.9.1; `website/docs/status/where-this-stands.md:11` reports 0.27.0 as the latest release; `website/docs/reference/cli.md:130` calls the CLI bindings unreleased (shipped 0.21.0) |
| Removed command still described | 1 | `website/docs/status/where-this-stands.md:82` lists `ess skill`, removed in 0.31.0 |
| Behaviour described wrongly | 5 | `site` emits HTML, while `website/docs/index.md:42`, `website/docs/getting-started.md:89-107`, `README.md:133-135` and `website/docs/examples/specification-to-contracts.md:263-293` say Markdown plus `sidebar.json` and "not HTML"; the example page cites files that do not exist |
| Broken or circular navigation | 4 | `guides/record-realization.md` is in no sidebar; three links to the example page render on the unified site as a GitHub source link because the page is not published there; `guides/synthesize.md:135` points at a roadmap section that does not exist; `concepts/ess.md:208` says "reused twice more" and names one |
| Missing documentation for a shipped feature | 3 | no page shows `ess verify conform synthesize --target go\|typescript` and the `Target` an adopter implements; the walkthrough never leaves the repository example; the `ess/` version table stops at `ess/5` |
| Internal jargon on a public page | 4 | `guides/synthesize.md:131-136` (W7.4, wave 7, `docs/plan/…`); `status/roadmap.md:28` (wave records); `reference/spec-versions.md` and `reference/formats.md` ("round-3 pair", "retrofit issues") |
| Hard to follow | 3 | `guides/write-a-specification.md:120-148` puts a Binary64 digression under "Validate early"; `reference/cli.md:48-72` and `guides/verify-conformance.md:43-75` explain input selection in suite-version shorthand |

## Out of scope

Pages or sections describing conformance exploration and concurrent history, which another line
of work is changing. Rust behaviour. The blog, which is a dated record.
