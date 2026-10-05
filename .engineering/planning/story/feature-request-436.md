---
format: aep.planning-md/3
id: story:feature-request-436
kind: story
status: active
title: 'Guidance: ESS does not require generated code in every repository'
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#436
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-435
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/adoption_modes_page.rs
- confidence: inferred
  path: website/docs/concepts/adoption-modes.md
- confidence: cited
  path: website/docs/getting-started.md
- confidence: cited
  path: website/docs/index.md
- confidence: cited
  path: website/sidebars.ts
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:28Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:28Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#436: Guidance: ESS does not require generated code in every repository.

## Origin
beyond10x/ess#436, filed 2026-10-05; an adopter asked on 2026-09-28 whether adopting ESS means every repository must switch to generated code.

## Fit review
1. Need: a page that names the ways to adopt ESS, says what each one costs and what it buys, and says a repository can stop at any of them. The requester lists four modes: specification only; conformance against a hand-written implementation; generated contracts beside hand-written code; generated behaviour. The list is theirs, and every mode in it already exists in the CLI.
2. Class: gap in documentation. No authored surface is involved.
3. Existing idiom: each mode exists and has a guide, but no page puts them side by side.
   - Specification only: `ess specify validate`, `ess verify diff` / `impact` (`website/docs/concepts/ess.md:128-138`, `:183-190`).
   - Conformance against hand-written code: a target the adopter writes (`website/docs/concepts/test-pyramid.md:17-27`, `:99-106`; `website/docs/guides/verify/runners.md`).
   - Generated contracts beside hand-written code: `ess generate --kind openapi|asyncapi|schema` and `ess generate types` (`website/docs/guides/generate-artifacts.md:14-30`, `:337-339`).
   - Generated behaviour: `ess generate synthesize`, where every capability is generated, an obligation, or refused (`website/docs/concepts/ess.md:192-210`).
   - Brownfield entry: `ess:retrofitting` (`website/docs/start/use-with-an-agent.md:33`).
   The Start path shows one route only, spec → OpenAPI → TypeScript suite (`website/docs/getting-started.md:9-22`; `website/docs/index.md:17-22`). That route reads as the way to adopt ESS. No concepts page says synthesis is optional. The `Concepts` sidebar has overview, model, component delivery and test pyramid (`website/sidebars.ts:23-32`).
4. Fit: a concepts page is hand-written prose with no derivable content, which the organization docs rule allows. It must not restate the CLI reference; it links to it. It must not overclaim. Its "what each costs" column cites the refusals each mode carries: synthesis refuses modeled Binary64 on every target (`concepts/ess.md:200-202`); the conformance runner cannot be imported as a library, and no wire-protocol target exists (`test-pyramid.md:121-127`).
5. Second adopter: a team with an existing Go service and an OpenAPI contract wants review and diff of changes plus a suite against the service it has, and no generated code. The same page answers it. The question is general.
6. Cost: one new page and one sidebar entry. Two one-line links: from `getting-started.md`, and from `index.md` "Where to go next". No format, CLI or diagnostic change.
7. Alternatives: (a) change nothing; refused, the question is not answered anywhere today. (b) a section in `concepts/ess.md`; refused, because that page is the model reference and is already 231 lines. (c) a new concepts page; chosen. (d) a guide; refused, because the question is "which mode", not "how to do one", and each mode already has its guide.

## Decisions
Accept as proposed, as documentation. New page `website/docs/concepts/adoption-modes.md` ("Adopt ESS at the depth you need"), listed in `website/sidebars.ts` under Concepts after `concepts/ess`. It has one table: mode | commands | what you commit | what it checks | what it does not | next step. It covers the four modes plus retrofitting as the entry point for an existing repository, and states that the modes are independent and a repository may stop at any of them. "What you commit" links to the #435 page instead of restating it. Depends on #435 (edge recorded), which creates that page; `site_navigation` fails while the link target is absent. Links from `getting-started.md` and from `index.md` "Where to go next". No format bump.

## Acceptance
- adoption_modes_page_states_the_modes: `website/docs/concepts/adoption-modes.md` exists with the title "Adopt ESS at the depth you need" and one table whose header is `mode | commands | what you commit | what it checks | what it does not | next step`. Its rows name "specification only", "conformance against a hand-written implementation", "generated contracts beside hand-written code", "generated behaviour" and "retrofitting". The page says "the modes are independent" and "stop at any of them", and links `guides/commit-generated-files.md`. `website/docs/getting-started.md` and the "Where to go next" table of `website/docs/index.md` each link `concepts/adoption-modes.md`. A case of that name in `crates/edge/ess-cli/tests/adoption_modes_page.rs` reads the three pages and fails naming the missing page, column, mode, phrase or link.
- site_navigation: `crates/edge/ess-xtask/tests/site_navigation.rs` passes. The new page is in the sidebar (rule 2), and every link on it lands on a page and heading (rule 4).
- Page claims: every refusal or limit the page names is cited to an existing guide or reference anchor. No shell session is recorded on the page, so there is no `tutorial_page.rs` obligation (that test covers `start/` pages only, `crates/edge/ess-cli/tests/tutorial_page.rs:76-82`).
- `task site-build` succeeds with no broken-link or broken-anchor warning (AGENTS.md: documentation changes pass `task site-build`).

## Scope
- website/docs/concepts/adoption-modes.md  inferred — new concepts page
- website/sidebars.ts  cited — Concepts category, :23-32
- website/docs/getting-started.md  cited — one link after :22
- website/docs/index.md  cited — "Where to go next" table, :45-53
- crates/edge/ess-cli/tests/adoption_modes_page.rs  inferred — new page-content case
