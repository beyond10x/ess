# R-02 heuristic review

6 pass, 6 partial, 4 fail, 0 not judged (4 judged in part only: 6, 11, 13, 16).

R-01 was 6 pass, 7 partial, 3 fail. One verdict moved: heuristic 9, partial to fail, on a changed
measurement (below).

Evidence used: `rounds/R-02/report.md`, `round.json`, `triage.md`, `decision.md`, the 7 files under
`shots/`, the 9 scenario cards, the adapter, findings UXF-001 to UXF-020, `trend.md` and the site
source under `website/`. No recording, no operator note and no questionnaire exist for this round
(`report.md`, "Missing inputs"), so no verdict cites a recording second. The theme's CSS is not in
the tree, so type sizes are read from screenshot pixels. No new shot was taken; no command was
run.

## What changed since R-01

The site is not changed in R-02 (`triage.md`, first paragraph; the 3 newest commits touching
`website/` are all `site-ux` commits). Every difference below comes from the tools:

- Coverage: 139 of 139 pages, 556 of 556 configurations, 22 of 22 task runs (`round.json`
  `coverage`); R-01 measured 138 of 139.
- Gate 2: 2 of 139 pages fail (R-01: 138), because search is no longer an always-visible element,
  the archive has no sidebar by design, and the end-of-page state is measured only (D15). The
  end-of-page state still fails on 136 docs and 136 blog configurations
  (`gates.gate2_always_visible.measured_only`).
- Gate 4 now fails: 10 of 18 pointer runs are over their card's limit
  (`gates.gate4_screen_only.over_limit`). R-01 had none gated.
- Axe: 2,732 nodes on 105 pages (R-01: 2,733 on 104), but the rule set changed: R-01 reported
  `nested-interactive` (2 nodes) and `scrollable-region-focusable` (1 node), R-02 reports neither
  (`gates.gate5_accessibility.by_rule` holds 3 rules; R-01 `report.md` line 15 held 5), and
  `color-contrast` fell from 22 nodes on 2 pages to 4 on 1. See UXF-021.
- Keyboard runs at mobile-390: 130 and 128 actions (R-01: 26 and 24). Desktop is unchanged: 23
  and 21.

## Verdicts

| # | heuristic | verdict | evidence |
| --- | --- | --- | --- |
| 1 | One next thing | partial | At desktop-1280 the landing frame shows a headline, a paragraph and a code panel, no button (`contact-light.png`, landing); at mobile-390 "Get started" and "See the model" are in the first screen (`contact-light.png`, mobile-landing). UXF-019 stands. The docs introduction shows the sidebar, a code block and "What ESS owns" in its first screen, no single next action (`contact-light.png`, docs-intro). Doc pages end with one "Next:" line and a Previous/Next card (`decision-yes.png`). |
| 2 | State in words | partial | Words: breadcrumbs (`worst-1-desktop-1280-dark.png`), the "EXPERIMENTAL" eyebrow on the landing page. Shape or tint only: the current sidebar entry, the navbar's current item (underline), a category's open state (chevron), and the theme mode, an unlabelled half-disc icon in every frame. `round.json` `theme-dark-and-back/desktop-1280` `screen_only.actions` 3 against a limit of 2 comes from the hidden mode (UXF-012). |
| 3 | Nothing required hidden | fail | No search control on any template page (UXF-001; `website/docusaurus.config.ts` has no search plugin). "Verify conformance" is found only by scrolling the sidebar: `target-process/desktop-1280` 5 actions of 4, `target-process/mobile-390` 12 of 4 (UXF-011). At 390 px every navbar link is behind the menu: `first-conformance-run/mobile-390` 4 of 3, `limitations/mobile-390` 4 of 3, `page-a-view/mobile-390` 11 of 5 (UXF-013). Terminal titles cut: 6 texts on 2 pages, `gates.truncation` (UXF-009). The "next step" column is out of view (`website/docs/concepts/adoption-modes.md:19`; UXF-020). |
| 4 | Stable layout | pass | `round.json` `scenarios` holds `layout_shift.load` for 278 configurations (dark theme, both viewports); the largest is 0.0144 (`/ess/releases/aggregates-over-related-guards@mobile-390/dark`), next 0.0133 (`/ess/releases/structural-synthesis@desktop-1280/dark`), every other value 0.0006 or less. `gates.gate1_no_horizontal_overflow` 0 page scroll, 0 spills of 139. The frames of both contact sheets keep the same positions in both themes. In part: the light theme has no shift value. |
| 5 | Provenance of every value | partial | Generated pages say so: "This page is generated from the change records kept in the repository" (`worst-1-desktop-1280-dark.png`); "Edit this page" (`decision-yes.png`). The Status page still names 0.51.0 as the latest release, observed 1 October 2026 (`website/docs/status/where-this-stands.md:13`), while the install page installs 0.97.3 (`website/docs/start/install.md:34`): a gap of 46 minor versions (UXF-016). The footer's "THIS BUILD docs-system 5fd348a" names the theme's revision, not the ESS release the pages describe (`decision-yes.png`). |
| 6 | Errors cheap | pass | A wrong page is one breadcrumb or one Back (`worst-3-desktop-1280-dark.png`, breadcrumb "Reference"); a wrong mobile menu panel is one "Back to main menu" (`contact-light.png`, mobile-menu). `gates.gate4_screen_only` 18 of 18 runs completed, `gates.gate5_keyboard` 4 of 4. In part only: no L3 run injects a wrong turn, so recovery is read from structure, not from a run. |
| 7 | Words the user can say | partial | The navbar says "Example" and "Visualise" (`website/docusaurus.config.ts:71-72`); the footer names the same two destinations "Specification to contracts" and "Visualise the billing model" (`:93-94`). The theme toggle has no visible word; its accessible name "dark and light mode" is what the card clicks (`scenarios/theme-dark-and-back/scenario.json`). The anchor cards' link names match their page headings (`first-conformance-run`, `install-linux-x86_64`, both 2 actions at desktop-1280). Graph nodes' names do not contain their label (UXF-005). No new record: no metric moves on it. |
| 8 | The reader's time respected | fail | 10 of 18 pointer runs over their card's limit (`gates.gate4_screen_only.over_limit`): `ess-31-release` 25 and 33 actions of 6, `target-process/mobile-390` 12 of 4, `page-a-view/mobile-390` 11 of 5, `diagnostic-type-009/mobile-390` 7 of 5 (UXF-010, UXF-011, UXF-013). `ess-31-release/desktop-1280` `time_to_act_steps.max` 24. Reading load at desktop-1280 reaches 291 words in one screen (`ess-31-release/desktop-1280` `reading_load.max`), against 142 at mobile-390. |
| 9 | Keyboard and pointer parity | fail | All 4 keyboard runs complete, at 21 and 23 actions on desktop against 2 for the pointer, and at 128 and 130 on a phone against 4 (`first-conformance-run`, `install-linux-x86_64`, `report.md` Screen-only user table). The phone ratio is 32 to 1; R-01 recorded 26 and 24 with the old tool, and the run that changed the count (`21e8c21e03`, the screen-only user reads a state after render) leaves the site-versus-tool split unproven. UXF-015 stands and its phone numbers are 5 times higher. Visualise: nested control and unreachable scroll region (UXF-007, UXF-008), not reported by axe this round (UXF-021). |
| 10 | Recognition over recall | partial | Recognition helps: sidebar categories, breadcrumbs and the table of contents sit in view on every desktop doc page (`worst-1-desktop-1280-dark.png`, `worst-3-desktop-1280-dark.png`). Recall needed: without search a reader must know which category holds a page (UXF-001, UXF-011); the `ess/31` row is under 24 scroll steps and the column header is out of view (`ess-31-release/desktop-1280` `steers` 23; UXF-018). |
| 11 | Calm by default | pass | `gates.reduced_motion` 0 of 139 pages animate or scroll smoothly; `motion.running` 0 in all 278 configurations that hold it (dark theme). No banner, toast or alert in any frame of either contact sheet. In part only: motion without the reduce preference is not measured, and `motion.running` has no light-theme value. |
| 12 | One state vocabulary | pass | One set of words: `shipped` on the Status page (`website/docs/status/where-this-stands.md:60`), "experimental" for the product (`:5` and the landing eyebrow in `contact-light.png`). No second vocabulary appears in any frame. |
| 13 | A wrong cue recovered in one action | pass | The toggle's no-change first press is recovered by the next press (`theme-dark-and-back/desktop-1280` 3 actions: two presses to dark, one back); a wrong menu panel is one "Back to main menu"; a wrong category is one click. In part only: the search half cannot be judged, the site has none (UXF-001, counted under 3). |
| 14 | Unmistakable end state | pass | `gates.gate3_unmissable_completion` `completion_miss` 0, `false_complete` 0; 18 of 18 pointer and 4 of 4 keyboard runs end on a page whose target (heading, or the `ess/31` cell beside a release number) is wholly in view and uncovered (scenario cards; `ux/adapter.ts`, `COMPLETION`). |
| 15 | Readable in every theme; key data at 14 px | fail | Links told apart by colour only: `link-in-text-block` 2,724 nodes on 103 pages, both themes, both viewports (`gates.gate5_accessibility.by_rule`; `worst-1-desktop-1280-dark.png` and `worst-3-desktop-1280-dark.png` outline them; UXF-004). Landing page, light theme: `color-contrast` 2 nodes at each viewport (`round.json` `/ess/@desktop-1280/light` and `/ess/@mobile-390/light`, `axe.serious_critical_nodes` 2; UXF-023). Sidebar category names and table column headers are uppercase and about 11 px (`worst-3-desktop-1280-dark.png`: "START HERE", "IDENTITY"; UXF-017). Dark theme is otherwise legible in all 8 dark frames (`contact-dark.png`). |
| 16 | Focus follows the work | partial | The sidebar opens on the current page and marks it, also in the mobile menu (`contact-light.png`, cli-middle and mobile-menu); the table of contents marks the section in view (`decision-yes.png`). At the end of a doc page the sidebar is pushed under the navbar (`decision-yes.png`; 136 docs and 136 blog configurations, measured only after D15; UXF-002, UXF-003). At desktop-1280 `/ess/docs/getting-started` and `/ess/releases/tags` fail gate 2 at the top or the middle (UXF-022). In part only: where keyboard focus lands after a click is not recorded. |

## Totals

- pass: 4, 6, 11, 12, 13, 14 (6)
- partial: 1, 2, 5, 7, 10, 16 (6)
- fail: 3, 8, 9, 15 (4)
- not judged: none; judged in part only: 6, 11, 13, 16

## New findings

Existing records cover UXF-001 to UXF-020. The next free id is UXF-021. Not re-filed: UXF-015 gets
new numbers (mobile keyboard 128 and 130 presses, 5 times R-01); update its table and raise it
from S3 to S2.

| id | title | class | severity | affected | cost | regression asset |
| --- | --- | --- | --- | --- | --- | --- |
| UXF-021 | Gate 5 stopped reporting three rules R-01 reported on an unchanged site, and the R-02 baseline holds that smaller set | engine | S2 | gate 5 on 2 of 139 pages (`/ess/docs/visualise`, `/ess/releases/archive`); the ratchet baseline of all 556 configurations | M | An L1 test on a doctored page holding a control nested in a control, a scrolling region without focus and a text under 4.5:1: gate 5 must report `nested-interactive`, `scrollable-region-focusable` and `color-contrast`, in the same axe mode the L2 test uses. |
| UXF-022 | Two short pages fail "docs sidebar" and "blog sidebar out of viewport" at the top or middle of the page at desktop-1280 | layout | S3 | 2 of 139 pages (`/ess/docs/getting-started`, `/ess/releases/tags`), 4 of 278 desktop configurations; the every-merge gate misses them (`gate-7`, `gate-8`, `gate-9` gate 2 pass on 9 of 9 template pages) | S | Add a short doc page and a release tag page to `TEMPLATE_PAGES` in `adapter.ts`; make the reporter name the page and the state of each gate 2 failure (`round.json` holds a count per element only). |
| UXF-023 | The landing page in the light theme has text below the contrast minimum at both gating viewports | affordance | S2 | 1 of 139 pages, 4 of 556 configurations (light only) | S | An L2 assertion on `/ess/` light: `color-contrast` 0 nodes; the reporter prints the node selector so the record can name the element. UXF-006 names other pages (visualise, archive) that R-02 no longer reports. |
| UXF-024 | A slice of the slow lane prints gate 1 and reduced motion as "fail" with "unmeasured 1" while the merged round passes both | engine | S3 | 6 of 11 runs (`slow-l2-1` to `slow-l2-6`); the merged `gates.gate1_no_horizontal_overflow` and `gates.reduced_motion` say pass with 0 unmeasured | S | An L1 reporter test: a run holding a slice of the sitemap marks a gate "partial", never "fail", for pages held by other slices. |

## Changes I would put first in the next round

| # | change | serves | metric in `round.json` that should move |
| --- | --- | --- | --- |
| 1 | Find why `nested-interactive`, `scrollable-region-focusable` and most `color-contrast` nodes vanished (axe mode, theme wait) before the next baseline is trusted; restore them in the report or show why they were false | UXF-021, UXF-006, UXF-007, UXF-008 | `gates.gate5_accessibility.by_rule` lists `nested-interactive` and `scrollable-region-focusable` again, or an L1 test proves them absent |
| 2 | One site-wide rule that underlines links inside running text | UXF-004 (heuristic 15) | `gates.gate5_accessibility.by_rule.link-in-text-block.nodes` 2,724 to 0; `gates.gate5_accessibility.nodes` 2,732 to under 10 |
| 3 | Add search, and a skip link to the sidebar and to the page body | UXF-001, UXF-015, UXF-010 (heuristics 3, 8, 9) | `ess-31-release/desktop-1280` and `/mobile-390` `screen_only.actions` 25 and 33 to 6 or fewer; `first-conformance-run/mobile-390/keyboard` `screen_only.actions` 130 to under 30 |
| 4 | Open the mobile menu on the main panel, and put "Verify conformance" and the Guides entries in the docs introduction's first screen | UXF-013, UXF-011 (heuristics 3, 8) | `gates.gate4_screen_only.over_limit` 10 entries to 4 or fewer; `target-process/mobile-390` `screen_only.actions` 12 to 4 |
| 5 | Fix the three one-file items: landing contrast and "Get started" above the first screen at 1280 px, and the two short pages' sidebar | UXF-023, UXF-019, UXF-022 (heuristics 1, 15, 16) | `/ess/@desktop-1280/light` `axe.serious_critical_nodes` 2 to 0; `gates.gate2_always_visible.failures` 4 to 0 |

## Notes for the coordinator

- The R-02 baseline replaces R-01's under decision `R-02-baseline` (`decision.md`). If UXF-021 is
  real, that baseline is too clean in gate 5 and a later round that restores the rules will read
  as worse; take the baseline decision again when UXF-021 closes.
- The slow lane steps `slow-l2-1` (142.1 s), `slow-l2-2` (136.4 s) and `slow-measured` (153.3 s)
  are over the 120 s step budget; the slowest single test is 20.3 s, under 60 s. UXF-014's closing
  text sets the slow lane's step limit at 28 minutes, so no new record; check that this matches the
  rule in `~/beyond10x/AGENTS.md` for a `slow` lane.
- No person, recording or questionnaire exists for R-02: heuristics 2, 7, 10 and 16 are judged from
  the screenshots and source only.
