# R-03 heuristic review

6 pass, 7 partial, 3 fail, 0 not judged (4 judged in part only: 6, 11, 13, 16).

R-02 was 6 pass, 6 partial, 4 fail. One verdict moved: heuristic 15, fail to partial, because the
colour-only links are gone (below). No other verdict moved.

Evidence used: `rounds/R-03/report.md`, `round.json`, `triage.md`, `decision.md`, the 11 files under
`rounds/R-03/shots/`, the 9 scenario cards (by name only), `adapter.ts`, findings UXF-001 to UXF-024,
`rounds/R-02/review-heuristics.md`, and the site source under `website/` (`src/css/custom.css`,
`docusaurus.config.ts`, `product.json`, `docs/`). No recording, operator note or questionnaire exists
for this round (`report.md`, "Missing inputs"), so no verdict cites a recording second. No command
was run and no shot was taken. `website/node_modules` is absent, so the theme's own DOM and CSS were
not read; claims about where the theme puts a component rest on the shots.

## What changed since R-02

The site changed in one place: commit `a1c9791b2c` adds `website/src/css/custom.css` (5 lines, one
rule) and registers it as `customCss` (`website/docusaurus.config.ts:56`). Everything else in R-03
changed the tools (`triage.md`, "Change set").

- Gate 5: `gates.gate5_accessibility.by_rule` holds 4 rules, none of them `link-in-text-block`
  (R-02: 2,724 nodes on 103 pages, `rounds/R-02/review-heuristics.md`, heuristic 15). Nodes 31 on 3
  of 139 pages (R-02: 2,732). I recomputed the 31 from the per-configuration values: landing, light,
  2 + 2 (desktop, mobile); `/ess/docs/visualise` 1 + 10 + 1 + 11 (dark and light, both viewports);
  `/ess/releases/archive` 1 + 1 + 1 + 1. Sum 4 + 23 + 4 = 31.
- The restored frame measurement (UXF-021, closed) shows `nested-interactive` 2 nodes and
  `scrollable-region-focusable` 1 node again, all on `/ess/docs/visualise`. They are R-01's
  UXF-007 and UXF-008, not new. `decision.md` takes 4 worse values on that page under `R-03-axe`.
- Gate 2: unchanged, 2 pages, 4 failures (`gates.gate2_always_visible`). Gate 4: unchanged, 10 of 18
  pointer runs over their limit, with the same counts as R-02 in every row of the Screen-only user
  table. Truncation: unchanged, 6 texts on 2 pages. Gate 1, gate 3, keyboard and reduced motion:
  pass, as in R-02.
- Layout shift: largest single value 0.0144
  (`/ess/releases/aggregates-over-related-guards@mobile-390/dark`), the same as R-02, but the
  per-viewport sums rose (new finding UXF-026).

## The link change

Rule, `website/src/css/custom.css:3-5`:

```
.markdown :is(p, li, td, th, dd, blockquote) a:not(.hash-link) { text-decoration: underline; }
```

Reads well. In `link-underline-before-*` the first-paragraph link "Start here" differs from its text
by hue only, in light (teal on grey) and dark (mint on grey). In `link-underline-after-*` the same
link carries a one-pixel underline in the link colour in both themes, at a contrast clearly above
the text it sits in. The underline does not touch the text beside it. `worst-2-desktop-1280-dark.png`
(`/ess/docs/getting-started`) shows the rule at density: a numbered list of 4 links, two paragraphs
with 3 and 1 links, every one underlined and every wrapped link underlined on both lines ("Explore
the repository" / "at the depth you need"). `worst-1-mobile-390-light.png` shows it at 390 px:
"ess-ui-templates" underlined beside inline code. Axe agrees: `link-in-text-block` is absent from
`by_rule` in both themes at both viewports.

What the selector matches, and what it does not:

- It matches only descendants of `.markdown` (the doc and blog article body) that sit in a
  paragraph, list item, table cell or header cell, definition, or quote. The navbar, the docs and
  blog sidebars, the breadcrumbs, the table of contents, the footer, the Previous/Next cards, "Edit
  this page" and the tag chips are not inside `.markdown`, and none of them is underlined in
  `link-underline-after-*`, `worst-2-desktop-1280-dark.png` ("Edit this page", breadcrumb, sidebar
  entries, navbar) or `worst-3-desktop-1280-dark.png` (tag chips, blog sidebar, navbar). Not judged:
  the mobile "On this page" control; it is too small in `contact-light.png` to read.
- Buttons and cards: the docs hold none inside `.markdown` (`grep` for `className`, `<div` and
  `<Link` in `website/docs/` finds only `<a id=...>` anchors). The landing page (`product.json`,
  rendered by the template) is outside `.markdown`: "Get started" and "See the model" keep their
  look in `contact-light.png`, mobile-landing.
- Heading anchors: `.hash-link` sits in `h1`-`h6`, which the `:is()` list does not name, so
  `:not(.hash-link)` is a second guard and changes nothing today.
- A link outside a `p`, `li`, `td`, `th`, `dd` or `blockquote` inside `.markdown` is not underlined
  (a link placed straight in a `div`, a `dt` or a `figcaption`). The docs hold no such link today
  (same `grep`), so no node shows it. The frame of `/ess/docs/visualise` is a separate document and
  the rule cannot reach it.
- It matches 85 empty `<a id="...">` anchors in 5 docs files (`website/docs/guides/verify-conformance.md`
  has 22 of them, each in front of a link). They have no text, so nothing is drawn. `a[href]` would
  be tighter. No cost to a reader.
- Cost to read: link-only lists, such as the 22 entries of the index in
  `website/docs/guides/verify-conformance.md`, are now 22 underlined lines. That reads as a list of
  links, which it is.
- Not judged: hover and focus states. No shot holds one. Infima changes the colour on hover; the
  underline stays, so a hover cue is colour only.

Verdict on the change: it does what UXF-004 asked and nothing outside it. One asset watches it:
the adoption-modes page in the gate (UXF-004's asset); the other 102 pages are held by the baseline,
where a page that returns to 1 node is a rise.

## Verdicts

| # | heuristic | verdict | evidence |
| --- | --- | --- | --- |
| 1 | One next thing | partial | Landing at desktop-1280 shows headline, paragraph and a code panel, no button (`contact-light.png`, landing; UXF-019); at mobile-390 "Get started" and "See the model" are in the first screen (mobile-landing). The docs introduction has no single next action in its first screen (docs-intro). The numbered list of `/ess/docs/getting-started` names the order, and its 4 links are now underlined (`worst-2-desktop-1280-dark.png`). |
| 2 | State in words | partial | Words: breadcrumbs, the "SHIPPED" pill and "CONCEPTS . 3 OF 5" eyebrow (`link-underline-after-desktop-1280-light.png`). Tint or shape only: the current sidebar entry, the current navbar item (underline), a category's open state (chevron), the theme mode, an unlabelled half-disc icon in every frame. `theme-dark-and-back/desktop-1280` `screen_only.actions` 3 against 2 (UXF-012). A link is now told by shape, not colour. |
| 3 | Nothing required hidden | fail | No search control on any template page (UXF-001; `website/docusaurus.config.ts` has no search entry; `decision.md` default no). "Verify conformance" is found by scrolling the sidebar: `target-process/desktop-1280` 5 of 4 and `/mobile-390` 12 of 4 (UXF-011). At 390 px the navbar is behind the menu: `first-conformance-run/mobile-390` 4 of 3, `limitations/mobile-390` 4 of 3, `page-a-view/mobile-390` 11 of 5 (UXF-013). `gates.truncation` 6 texts on 2 pages (UXF-009). |
| 4 | Stable layout | pass | `gates.gate1_no_horizontal_overflow` page_scroll 0, spills 0 of 139. `layout_shift.load` on 278 configurations (dark): largest 0.0144, next 0.0133, next 0.007. Contact sheets keep their frame positions in both themes. In part only: the light theme has no shift value, and the sums rose from R-02 (UXF-026). |
| 5 | Provenance of every value | partial | Generated pages say so and link "Edit this page" (`worst-2-desktop-1280-dark.png`, `decision-yes.png`). The Status page still names 0.51.0 as the latest release (`website/docs/status/where-this-stands.md:14`, table row at `:23`) while the install page installs 0.97.3 (`website/docs/start/install.md:34`): UXF-016, open. |
| 6 | Errors cheap | pass | A wrong page is one breadcrumb or Back (`worst-2-desktop-1280-dark.png`); a wrong mobile menu panel is one "Back to main menu" (`contact-light.png`, mobile-menu). `gates.gate4_screen_only` completed 18 of 18, `gate5_keyboard` 4 of 4. In part only: no L3 run injects a wrong turn. |
| 7 | Words the user can say | partial | Navbar "Example" and "Visualise" against footer "Specification to contracts" and "Visualise the billing model" (`website/docusaurus.config.ts:71-72`, `:93-94`). The theme toggle has no visible word. Graph node names lack their label (UXF-005; `label-content-name-mismatch` 6 nodes on 2 pages, `by_rule`). |
| 8 | The reader's time respected | fail | `gates.gate4_screen_only.over_limit` has 10 entries of 18 runs: `ess-31-release` 25 and 33 of 6, `target-process/mobile-390` 12 of 4, `page-a-view/mobile-390` 11 of 5, `diagnostic-type-009/mobile-390` 7 of 5 (UXF-010, UXF-011, UXF-013). `ess-31-release/desktop-1280` `reading_load.max` 291 words in one screen, `screen_only.steers` 23. |
| 9 | Keyboard and pointer parity | fail | Keyboard runs complete (4 of 4) at 23 and 21 actions at desktop-1280 against 2 for the pointer, and at 130 and 128 at mobile-390 against 4 (`first-conformance-run`, `install-linux-x86_64`, Screen-only user table). UXF-015. Visualise: `nested-interactive` 2 nodes and `scrollable-region-focusable` 1 node, again reported (UXF-007, UXF-008). |
| 10 | Recognition over recall | partial | Sidebar categories, breadcrumbs and the contents column are in view on desktop doc pages (`worst-2-desktop-1280-dark.png`). Underlined links make a link recognisable inside a sentence in both themes (`link-underline-after-*`). Recall still needed: no search (UXF-001); the `ess/31` row sits under 23 scroll steps (UXF-010). |
| 11 | Calm by default | pass | `gates.reduced_motion` 0 of 139 pages animate; `motion.running` 0 in all 278 dark configurations. No banner, toast or alert in any frame of `contact-light.png` or `contact-dark.png`. In part only: motion without the reduce preference is not measured; no light-theme value. |
| 12 | One state vocabulary | pass | "SHIPPED" pill and "experimental" eyebrow (`link-underline-after-desktop-1280-light.png`, `contact-light.png` landing); `website/docs/status/where-this-stands.md:60`. No second vocabulary in any frame. |
| 13 | A wrong cue recovered in one action | pass | `theme-dark-and-back/desktop-1280` 3 actions: the no-change first press is recovered by the next. A wrong menu panel is one "Back to main menu". In part only: the search half cannot be judged, the site has none. |
| 14 | Unmistakable end state | pass | `gates.gate3_unmissable_completion` `completion_miss` 0, `false_complete` 0; 18 of 18 pointer and 4 of 4 keyboard runs completed (`report.md`, Screen-only user table). |
| 15 | Readable in every theme; key data at 14 px | partial (was fail) | Fixed: `link-in-text-block` 2,724 nodes to 0 in both themes, shown in `link-underline-after-desktop-1280-{light,dark}.png`. Still open: `color-contrast` 22 nodes on 2 pages, which I resolve as `/ess/docs/visualise` 18 (light 8 + 8, dark 1 + 1) and `/ess/releases/archive` 4 (UXF-006); uppercase sidebar and table headers at about 11 px (`link-underline-after-desktop-1280-light.png`: "START HERE", "MODE"; UXF-017, not measured). Dark theme is legible in all 8 dark frames (`contact-dark.png`). |
| 16 | Focus follows the work | partial | The sidebar opens on the current page and marks it (`worst-2-desktop-1280-dark.png`, `contact-light.png` cli-middle, mobile-menu). `gates.gate2_always_visible` fails on 2 pages at the middle state (`/ess/docs/getting-started`, `/ess/releases/tags` at desktop-1280, 4 configurations; UXF-022), and measured-only at the end state on 136 docs and 136 blog configurations (UXF-002, UXF-003). `worst-2` and `worst-3` show the top of the page, where the sidebar is in view, so the shot does not show the failing state. Not recorded: where keyboard focus lands after a click. |

## Totals

- pass: 4, 6, 11, 12, 13, 14 (6)
- partial: 1, 2, 5, 7, 10, 15, 16 (7)
- fail: 3, 8, 9 (3)
- not judged: none; judged in part only: 6, 11, 13, 16

## New findings

Existing records cover UXF-001 to UXF-024. Not re-filed: UXF-004 and UXF-021 are closed and hold up
in R-03's numbers. UXF-023 (closed as not reproduced) holds: the landing page's 2 nodes per light
configuration are `label-content-name-mismatch` (UXF-005); the 22 color-contrast nodes sit on
`/ess/docs/visualise` (18) and `/ess/releases/archive` (4), by the arithmetic above. UXF-024 still stands: R-03 still prints slices as
failed (`report.md`, Runs: `slow-l2-1` passed with 3 known failures, `unmeasured` 1).

| id | title | class | severity | affected | cost | regression asset |
| --- | --- | --- | --- | --- | --- | --- |
| UXF-025 | Heavy reference pages run at or over the 60 s test bound, and the gate step ran to its 120 s cut | engine | S2 | 4 of 278 L2 page tests timed out at 60 s (`/ess/docs/reference/cli` both viewports, `/ess/docs/reference/ess-ui` both viewports, `report.md` slowest tests: 8 of the 10 slowest are `timedOut` at 60.0 to 60.2 s) and 1 of 22 L3 runs (`slow-l3`, `ess-31-release mobile-390 pointer`, 60 s); 1 of 3 gate runs cut (`gate-4`, status `timedout`, 119.6 s; `gate-5` 100.7 s, `gate-6` 93.6 s) | M | An L1 reporter test on a canned timing record: any test over 45 s, or a gate step over 90 s, fails the round with the page named. Split the light and dark axe pass of a page into two tests (axe light 19.6 to 23.2 s per page in `report.md`, "Where the slowest pages spent their time"). Related: UXF-014 (closed in R-02 at 28.7 to 41.6 s per gate step; its fix for `ess-ui` was the axe mode, and R-03's frame repair, UXF-021, is the likely cause of the rise); reopen UXF-014 instead if the coordinator prefers one record. |
| UXF-026 | `layout_shift.load` is load-dependent, so the ratchet's lower-is-better value moves with the host, not the site | engine | S3 | 278 of 556 configurations carry the value; the sums at mobile-390/dark rose 0.028 to 0.039 and at desktop-1280/dark 0.013 to 0.015 on a site unchanged in layout (`report.md`, Metrics table); one page read 0.0253 in `slow-l2-4` and 0.00004 in `rerun-shift` (`report.md`, Notes; host load 40 to 165 on 20 cores) | S | An L1 test on a canned page whose shift is recorded three times: the metric is the median of 3 loads, or a value under 0.01 is written as 0, with the rule named in the ratchet's `about`. Run the probe with the machine's load named in the round. |

## Changes I would put first in the next round

All five are changes to the site. Gate 4 holds 10 entries; 8 of them are covered by changes 1 and 2.

| # | change | serves | metric in `round.json` that should move |
| --- | --- | --- | --- |
| 1 | Open the phone's navbar menu on the main panel, and put "Verify conformance" and the Guides entries in the docs introduction's first screen | UXF-013, UXF-011 (heuristics 3, 8) | `gates.gate4_screen_only.over_limit` 10 entries to 5 or fewer; `page-a-view/mobile-390` `screen_only.actions` 11 to 5; `target-process/mobile-390` 12 to 4; `target-process/desktop-1280` 5 to 4; `first-conformance-run/mobile-390` and `limitations/mobile-390` 4 to 3 |
| 2 | Let a reader reach one format version or one diagnostic code without scrolling the table: an in-page list of versions and codes at the top of `reference/spec-versions` and `reference/diagnostics`, each a link to its row | UXF-010 (heuristics 8, 10) | `ess-31-release/desktop-1280` `screen_only.actions` 25 to 6 or fewer, `/mobile-390` 33 to 6 or fewer, `diagnostic-type-009/mobile-390` 7 to 5 |
| 3 | Fix the controls the visualise page's frame ships: bump the pinned `ess-ui-templates` commit to one without the nested control, the unreachable scroll region and the low-contrast text, or hold the page back from the sweep with a named decision | UXF-006, UXF-007, UXF-008, UXF-005 (heuristics 9, 15) | `gates.gate5_accessibility.nodes` 31 to 8 or fewer (23 are this page); `/ess/docs/visualise@desktop-1280/light` and `@mobile-390/light` `axe.serious_critical_nodes` 10 and 11 to 0; `by_rule` `nested-interactive` and `scrollable-region-focusable` absent |
| 4 | Keep the docs and blog sidebar in view on short pages: a sticky sidebar with its own height limit and scroll in `website/src/css/custom.css` | UXF-022, UXF-002, UXF-003 (heuristic 16) | `gates.gate2_always_visible.failures` 4 to 0 and `pages` 2 to 0; `.measured_only` "docs sidebar: out of viewport at end" 136 to 0 |
| 5 | Two one-rule fixes in `custom.css`: wrap the terminal titles at 390 px, and give the archive page's `h1` a colour that passes 4.5:1 | UXF-009, UXF-006 (heuristics 3, 15) | `gates.truncation.texts` 6 to 0 and `pages` 2 to 0; `/ess/releases/archive@*` `axe.serious_critical_nodes` 1 each (4 configurations) to 0 |

Next in line, not in the five: the theme toggle that shows no change on its first press (UXF-012),
which holds the last 2 over-limit entries (`theme-dark-and-back/desktop-1280` 3 of 2, `/mobile-390` 4
of 2), and the Status page's 0.51.0 (UXF-016), which no `round.json` metric measures yet.
