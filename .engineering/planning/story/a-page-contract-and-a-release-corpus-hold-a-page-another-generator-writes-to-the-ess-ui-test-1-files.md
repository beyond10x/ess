---
format: aep.planning-md/3
id: story:a-page-contract-and-a-release-corpus-hold-a-page-another-generator-writes-to-the-ess-ui-test-1-files
kind: story
status: draft
title: A page another generator writes is held to the ess-ui-test/1 files by a published page contract and a conformance corpus each release ships
tags:
- ess-ui
- external-request
- gap-reports-2026-11-25
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:the-reference-states-what-a-refetching-resume-keeps-in-fixture-mode-and-what-a-by-id-from-view-answers-without-a-match
revision: 1
---
## Outcome

An application that writes its own generator from `ess-ui/1` runs the same `ess-ui-test/1` files
against its served pages that ESS runs against the generated React project, and gets the same
verdict wherever the format promises one. What a page must carry for a test to read it is written
down once and versioned. Each release ships a corpus of documents, fixtures and tests that pins the
behaviour of every construct, and any generator can be held to it.

## Context

A downstream page generator wrote its own generator for one page of its application from
`ess-ui/1` (0.98.0): static HTML and plain JavaScript, and it re-implements the behaviour, not the
form: live effects, `coalesce`, the channel lifecycle and `resume`, fixture scripts and their loop,
derived fixture views, expressions. The form it takes from the resolved document
(`ess-ui-resolved/1`, the active stories below). Only a suite holds a generator to the behaviour.

What ESS does today:

- `ess ui test --playwright` writes a spec "for the project `ess generate ui --target react`
  generates" and nothing else (`website/docs/reference/ess-ui-test.md:448-480`, "In the browser").
- The spec selects `[data-ui-path="<path>"]`, and also `.ui-row` (`crates/ui/ess-ui-test/src/playwright.rs:812`),
  `data-status` (`:815`), `.ui-stale-badge` (`:864`), the focused-row class `ui-focused` (`:967`),
  `input[aria-label="Select row"]` (`:612`), the pager's `Previous` and `Next`, and the requests
  `/views/<view>` (`:138`) and `/commands/<command>` (`:216`). The reference lists some of them in
  its table of steps. None is a named, versioned contract, so a page from another generator meets
  them only by copying the React runtime's names.
- One verdict in both renderers is held by the parity rules (`ess-ui-test.md:412-446`), read from
  `ess-ui-test`'s own source, for the terminal and the generated React project only.
- ESS pins behaviour with its own tests under `crates/ui/ess-ui-test/tests/` and
  `crates/ui/ess-ui-react/tests/`. No tree path holds them as a corpus a second implementation can
  run.

The requester's proposal, restated and labelled as theirs: a page contract published with
`ess-ui-test/1`, `ess ui test --playwright` reading only that contract and taking the base URL and
the form of a page route from configuration, and a conformance corpus at a fixed path of each tagged
tree that the terminal and the React project both pass.

Beside (existing stories, not repeated here):

- `story:the-terminal-addresses-every-node-the-browser-addresses` (active): the paths.
- `story:with-model-ess-ui-resolved-1-carries-the-route-table-the-stock-generators-bind-the-document-s-views-and`
  (active): the route table another generator reads, for requests.
- `story:the-generated-react-project-draws-one-section-overlay-or-widget-alone-at-a-given-width-and-the-playwright-spec`:
  what the spec does for a part of a page.

## Acceptance

1. Red first: a test in `crates/ui/ess-ui-test/tests/` extracts every mark the generated spec
   selects (the constants of `playwright.rs`) and fails while the `ess-ui-test/1` reference lists
   a mark for none of them or lists one the spec does not select. Today the reference lists no
   such table, so the test fails.
2. The `ess-ui-test/1` reference has a page contract: for each node kind a step reads or acts on,
   the mark a page carries (`data-ui-path` on every addressed node, rows and cells included, in the
   canonical paths; the row mark; the focused row; a section's status; the stale mark; the
   pager's controls; a row's selection control) and the requests through which a page reads fixtures
   and sends commands. The contract has a version, and a generated spec names the version it needs.
3. `ess ui test --playwright` selects only marks of that contract and takes the base URL and the
   form of a page route from its configuration. The generated React project meets the contract;
   its existing spec cases keep their verdicts.
4. Each release carries a UI conformance corpus at a fixed path of the tagged tree: documents,
   fixtures and `ess-ui-test/1` tests that pin every live effect, `coalesce`, the channel
   lifecycle and `resume`, fixture scripts and their loop, derived fixture views, expressions and
   the filled defaults. The terminal and the generated React project pass all of it, and the
   reference says how another generator runs it.
5. Where the renderers disagree today, the corpus pins the answer that its own story decides, and
   leaves the case out until then (`story:the-reference-states-what-a-refetching-resume-keeps-in-fixture-mode-and-what-a-by-id-from-view-answers-without-a-match`
   names three such places).
6. The corpus's terminal run stays inside the repository's test budget (the test step ends within
   2 minutes, no test runs over 60 s). Its browser run is a lane of its own, outside the CI shards
   as `test-browser-product` is today. Each test is timed before it is added, and one over 5 s is
   named with its time.

## Fit review

Tree `2e055bcd0c` (branch `plan/gap-reports-2026-11-24`), read-only. This story covers requirement 1
of a downstream page generator's report of 2026-11-25.

1. **Need.** A generator other than ESS's own cannot be held to the behaviour `ess-ui/1` promises:
   nothing names what a page must carry for a test to read it, and nothing ships a set of tests
   that a second implementation runs. Minimal reproduction (fresh, not run here): serve a page
   that draws every node with its `data-ui-path` and nothing else of the React runtime's markup,
   and run `ess ui test --playwright` over any test that selects a row; the row, focus and pager
   steps need `.ui-row`, `ui-focused` and the pager's button names. The requester's proposed
   spelling is stated under Context.
2. **Class.** *Gap*: a real fact (the contract between a test and a page) cannot be read anywhere.
   Not a defect: the reference promises the React project only.
3. **Already expressible?** Partly. A generator may copy the React runtime's marks, which is the
   workaround, and the reference does not promise them to stay (`ess-ui-test.md:448-480`).
   An idiom that leaves the checked behaviour unpinned is no answer (skill question 3).
4. **Fit.** The language grids cover the specification language and not `ess-ui/1`
   (`docs/design/language-grids.md:15`), so no cell applies. It reuses `ess-ui-test/1` and the node
   paths, adds no `ess-ui/1` key and no step. Both renderers already read the parity rules from one
   place, so a published contract extends that rule to a third implementation. Every target is
   named: the terminal and the generated React project meet it, a third implementation is held to it.
5. **Second adopter.** A team that generates Vue or Svelte pages from the same document and wants
   the tests ESS writes to run against them.
6. **Cost.** A reference section and a version for the contract; a changed Playwright emitter that
   stops reading names outside it; a corpus tree that every release keeps and runs; the browser lane
   of that corpus. A contract version is a new versioned surface, and a corpus a new
   release deliverable: both decisions below are the owner's.
7. **Considered.** (a) Change nothing: another generator copies the React names. (b) A contract
   section in `ess-ui-test/1`'s reference plus a corpus in the tagged tree (chosen). (c) A separate
   format for the contract. (d) ESS runs a browser against another generator's URL itself.

**Defer.** The need is real; the form of the contract and the corpus's delivery touch the
architecture, so they are open decisions and no key is accepted.

## Decisions

Would add to `ess-ui-test/1`: nothing under PC-D1 A (a reference section and a version constant in
the generated spec). Would add under PC-D1 B: a format id for the contract (waits for the owner's
read before a release). Would add to `ess-ui/1`: nothing.

- **PC-D1. What form does the page contract take?** (A) A section of the `ess-ui-test/1`
  reference with a version the generated spec names; no new format. (B) A format of its own with
  its own schema and version. (C) Change nothing; the marks stay the React runtime's.
  **Recommended: A.** Open, the coordinator's. B is a new format: architecture: the owner's.
- **PC-D2. How does the corpus reach a generator's author?** (A) A directory of the tagged tree
  plus a reference page that says how to run it with `ess ui test --playwright`. (B) A release
  asset beside the archives. (C) A published package. **Recommended: A.** Open, the coordinator's.
  B and C add a release deliverable: architecture: the owner's.
- **PC-D3. Does ESS run the corpus against a page it did not generate?** (A) No: ESS ships the
  files and the spec, and runs the corpus on the terminal and on the generated React project; the
  generator's author runs the spec in their own browser. (B) ESS adds a command that starts a
  browser against a URL. **Recommended: A.** Open, the coordinator's. B is a new execution surface:
  architecture: the owner's.

## Source

A downstream page generator, report of 2026-11-25, requirement 1.
