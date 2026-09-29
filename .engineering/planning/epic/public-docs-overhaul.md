---
format: aep.planning-md/3
id: epic:public-docs-overhaul
kind: epic
status: active
title: Public documentation takes a first-time adopter from zero to a green conformance run
summary: 'Audit and overhaul of the website docs and README against 0.38.0: stale claims, missing adopter path, navigation, jargon.'
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T14:10:56Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-29T14:10:56Z", actor: "human:timo", revision: 4}
---
## Outcome

A first-time adopter can go from nothing to a validated specification of their own, a generated
artifact and a green conformance run against their own implementation, using only the published
documentation. Every reference page is generated from the code or checked against it, and the
tutorials run in CI, so the site cannot fall behind the release it documents again.

## Audit (2026-09-29, against 0.43.0 at `origin/main` 24974b10c)

| Problem | Evidence |
|---|---|
| Stale content | `concepts/overview.md`, `concepts/component-delivery.md`, `status/limitations.md` last changed 2026-09-20; the tutorial pins `format: ess/1` (current ess/18) with no `ess-inputs.yaml`; no 0.43.0 release post |
| Structure | `guides/write-a-specification.md` 1636 lines, `guides/verify-conformance.md` 755; one sidebar, six categories all expanded; the example page is excluded from the unified bundle while other pages link to it |
| Onboarding | install is macOS-only (`shasum`); the agent path (SETUP.md, `/ess:init`) is only in `README.md`; no test executes the tutorial (`cargo xtask docs` checks only its versions) |
| Reference gaps | no diagnostics page (ESS-AUTHOR 3 of 37, ESS-SYNTH 7 of 18, no compiler class table, 0 of 15 dotted refusals documented); `reference/cli.md` hand-written and missing `project buildkit/helm`, `conform author/select` and flags such as `--catalog`, `--chart`, `--stack-lock`, `--timeout`, `--retry-of`; about 33 format families named but not tracked by `FORMAT_RELEASES` |

The 2026-09-28 audit against 0.38.0 (stale versions, removed `ess skill`, `site` described as
Markdown, internal wave vocabulary) is carried by the stories below where it still holds.

## Plan

Wave A builds generators and checks in `ess-xtask` and `ess-cli` tests (Rust): CLI reference from
clap, diagnostics reference from the code catalogues, every format family tracked, an executable
tutorial. Wave B restructures and rewrites on top of them: information architecture with
redirects, onboarding, guides and status, releases, the example page in both renderings. Units
merge into `integrate/ess-docs`; one PR to `main`.

## Out of scope

The beyond10x organization website, Atlas and docs-system changes, an ESS version release. Docs
publish from `main` through `pages.yml` and the `b10x-docs-*` workflows and are reported pending
until publication is verified.
