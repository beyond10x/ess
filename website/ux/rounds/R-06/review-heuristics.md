# R-06 heuristic review

7 pass, 7 partial, 2 fail, 0 not judged (judged in part only: 1, 6, 11, 12, 13, 16).

R-05 was 7 pass, 7 partial, 2 fail. No verdict moved. Three partials gained without crossing a line:
1 (the landing's primary action is now in the first screen at 1280), 8 (task runs over their card's
limit 5 to 3) and 15 (axe nodes 27 to 23, the landing's graph clean). Nothing got worse.

Evidence read: `website/ux/rounds/R-06/report.md`, `round.json` (header, runs), `triage.md`,
`decision.md`, `notes.md`, `probe.json` (size only), `shots/contact-light.png`,
`shots/contact-dark.png`, `shots/worst-1-mobile-390-light.png` (all three read as images),
`website/ux/rounds/R-05/review-heuristics.md`, `website/ux/trend.md`, scenario card
`scenarios/page-a-view/`, findings UXF-001 to UXF-029 (titles and status; full text of 006, 012, 013,
019, 027, 029), and the sources `website/product.json`, `website/docs/index.md`,
`website/docs/status/where-this-stands.md`, `website/docs/start/install.md`,
`website/docs/visualise.md`. I ran no build, test, server or browser and took no shot.

No recording, person or questionnaire exists for the round (`report.md`, "Missing inputs"), so every
verdict rests on the report's numbers, the three shots and page source. Where a claim is about a page
no shot shows, it says "not observed".

What the shots show: both contact sheets (landing, docs-intro, install, cli-middle, release-post-index,
mobile-landing, mobile-doc, mobile-menu) in light and dark; the worst moment is the top of
`/ess/docs/visualise` at 390 px in light. The worst-moment shot carries no magenta outline: the
report says the outline marks the first axe nodes, but the frame shown is the page's opening text and
none of the 8 contrast nodes can be told from it (they lie below the frame, in the presentation).
The shot does not show the failure it is filed under.

## Verdicts

| # | heuristic | verdict | evidence |
| --- | --- | --- | --- |
| 1 | One next thing | partial | Landing at 1280: the filled primary "Get started" and the secondary "Run your first conformance suite" now sit at the bottom edge of the first frame (`shots/contact-light.png`, `landing (desktop-1280)`), where R-05 had none; the other two actions are below. At 390 the primary and three text links fill the first screen (`mobile-landing`). Docs intro still offers four equal "Start here" links with no first among them (`contact-light.png`, `docs-intro`; `website/docs/index.md:14-17`). UXF-019 stays open with no test: its R-06 note is a scratch probe, "not a measurement of the round". In part only: no recording, so the frame's fold at 1280 x 800 is read from a thumbnail. |
| 2 | State in words | partial | Unchanged. Theme mode is an unlabelled half-disc in every frame (`contact-dark.png`, top right of all 8 desktop frames, and `mobile-menu`); the current sidebar item is shown by tint (`install`, `cli-middle`). `theme-dark-and-back` takes 3 of 2 (1280) and 4 of 2 (390) actions (`report.md`, Screen-only user; UXF-012). |
| 3 | Nothing required hidden | fail | No search on any page (UXF-001, open by default). On a phone the docs categories are collapsed behind the menu (`contact-light.png`, `mobile-menu`: CONCEPTS, GUIDES, REFERENCE, EXAMPLES, RELEASES as chevron rows). `page-a-view/mobile-390` 11 actions against 5 with 8 steers (`report.md`, L3 table). Gain: `first-conformance-run` and `limitations` at 390 now 3 of 3, `install-linux-x86_64` 3 of 4 (was 4, 4, 4 in R-05) through an in-page link, which is the site hiding nothing for those three routes. |
| 4 | Stable layout | pass | Gate 1: 0 spills, 0 page scroll, 139 of 139; truncation 0 of 139. `layout_shift.load` 0.013 (1280) and 0.015 (390) in light (`report.md`, Metrics). The fall from 0.028 at 390 is not credited to the site: UXF-026 says the value moves with the host's load. |
| 5 | Provenance of every value | pass | The newest release is written once by `cargo xtask docs --write` from the changelog; install and status pages both say 0.98.0 (`website/docs/start/install.md:35`, `website/docs/status/where-this-stands.md:19`) and the status page names its generator in its front matter (`source:`) and in the page ("Written from `CHANGELOG.md` and `release-notes/`"). The hand-typed copies the R-05 review counted (13 and 3) are now generator output; the check is `cargo xtask docs`, which refuses a page not as `--write` writes it. Not observed: the L1 test the commit message names (not in the round dir). |
| 6 | Errors cheap | pass | A wrong menu panel is one "Back to main menu" (`contact-light.png`, `mobile-menu`); 18 of 18 pointer and 4 of 4 keyboard runs complete (`report.md`, Gates 3 and 5). In part only: no run injects a wrong turn, and the card twists ("looks under Guides first") are played as scripted steps. |
| 7 | Words the user can say | partial | Unchanged from R-05: navbar "Example" and "Visualise" against the breadcrumb "Examples" and page title "Visualise a specification" (`shots/worst-1-mobile-390-light.png`: breadcrumb "Examples > Visualise the billing model", h1 "Visualise a specification"); graph nodes lack their label in their accessible name (`label-content-name-mismatch` 2 nodes, `report.md` Gate 5; UXF-005). Small drift the round added: the status page's heading is "Latest release" and the generated text under it says "The newest release is" (`where-this-stands.md:13,19`); both are sayable, but one word would do. |
| 8 | The reader's time respected | partial | Gate 4 still fails: 3 runs over their card's limit (`page-a-view/mobile-390` 11 of 5, `theme-dark-and-back` 3 of 2 and 4 of 2), down from 5 (R-05). Reading load fell on most task runs (`report.md`, Metrics, "this round" against "last round": `first-conformance-run/mobile-390` max 108 to 92, `limitations/mobile-390` 108 to 87, `ess-31-release/desktop-1280` median 217 to 194). Not a reader cost, but a loop one: `slow-l2-6` timed a page test out at 60 s at load 56.81 (UXF-029). |
| 9 | Keyboard and pointer parity | fail | Keyboard 23 and 21 presses at 1280 against 2 and 2 pointer actions; 81 and 80 at 390 against 3 and 3 (`report.md`, L3 table). Down from 131 and 129 on the phone (the in-page link spares the menu), still 27 times the pointer. `nested-interactive` 2 nodes and `scrollable-region-focusable` 1 node on the visualise page (UXF-007, UXF-008; gate 5). The 4 of 4 keyboard runs complete, so this is cost, not blockage. |
| 10 | Recognition over recall | partial | Index lists and the "Last five releases" table give recognisable rows (`where-this-stands.md:21-25`; UXF-027's `nav` lists, R-05). Still recall: no search (UXF-001), 37 versions and 77 diagnostic codes behind two indexes, `page-a-view` needs the reader to guess "Guides" then "Write a specification" then "Views" (`scenarios/page-a-view/`: three clicks after expanding a category). |
| 11 | Calm by default | pass | `reduced_motion` 0 of 139 pages animate or scroll smoothly; `motion.running` 0 in all 278 light configurations (`report.md`, Gates and Metrics). No banner or alert in the 16 contact-sheet frames. In part only: motion without the reduce preference is not measured. |
| 12 | One state vocabulary | pass | Landing eyebrow "EXECUTABLE SYSTEM SPECIFICATIONS · EXPERIMENTAL" in both themes (`contact-light.png`, `contact-dark.png`, `landing`); one "experimental" word on the status page front matter. In part only: no frame shows a status pill, the status page or the capability states ("shipped", "planned" in `website/product.json`). |
| 13 | A wrong cue recovered in one action | pass | `theme-dark-and-back` 3 for 2: the no-change first press is recovered by the next press. A wrong menu panel is one "Back to main menu". In part only: the search half cannot be judged (no search). |
| 14 | Unmistakable end state | pass | `completion_miss` 0 and `false_complete` 0 in all 22 runs (`report.md`, Gate 3 and Metrics). In part only: the end state is "heading in view" by the card, not a sentence on the page. |
| 15 | Readable in every theme; key data at 14 px | partial | `color-contrast` 18 nodes, all on `/ess/docs/visualise` in light (`report.md`, Gate 5: 23 nodes on 1 page, from 27 on 2). The landing is clean in both themes after the theme release 0.15.0 (`notes.md`; `contact-light.png` and `contact-dark.png` `landing` read well). Dark is legible in all 8 dark frames (`contact-dark.png`). The lede in the worst-moment shot is a mid grey on pale green (`worst-1-mobile-390-light.png`); whether it is among the 8 nodes is not observed. Sidebar category names and table headers at about 11 px remain visible in every desktop docs frame and are still unmeasured (`contact-light.png`, `cli-middle`; UXF-017). |
| 16 | Focus follows the work | partial | Gate 2: 0 failures at the gated states; the `end` state of the docs and blog sidebars is out of view on 136 and 136 configurations, measured only (accepted, UXF-002, UXF-003); 1 desktop-1440 page has an element out of view or missing, unnamed (`report.md`, Measured-only viewports; UXF-028). The wrapped-link miss that the screen-only user hit (`report.md`, notes, last item) is a tool fault, not a reader's, and does not count here. |

## Totals

- pass: 4, 5, 6, 11, 12, 13, 14 (7)
- partial: 1, 2, 7, 8, 10, 15, 16 (7)
- fail: 3, 9 (2)
- not judged: none; judged in part only: 1, 6, 11, 12, 13, 16

## Findings filed

None filed (read-only brief). Every fail and every costly partial has an existing record:
UXF-001 (3, 10), UXF-005 to UXF-008 (7, 9, 15), UXF-012 (2, 13), UXF-013 (3, 8), UXF-015 (9),
UXF-017 (15), UXF-019 (1), UXF-026 (4), UXF-027 (10), UXF-028 (16). Two proposals have no record and
are for the next triage:

### P1. A reader who cannot use the visualise frame has the model as text only through the frame

- title: The visualise page gives the model only through the frame; a reader who cannot use the frame has no text version on the page
- class: affordance
- severity: S2
- metric: new L1 count, `visualise.text_model_entities` (entities written as page text) against the presentation file's entity count; today 0 of N. Does not move `gates.gate5_accessibility.nodes` (the frame keeps its 23 nodes).
- found_by: L7 heuristic reviewer (heuristics 3, 9, 15), R-05 as "new", carried and still without a record in R-06
- evidence: `website/docs/visualise.md` has no table, list or link to the model as text (the only text is the lead paragraph and the frame); `shots/worst-1-mobile-390-light.png` shows the page's text above the frame and nothing that names a text version; `report.md` Gate 5 (23 serious nodes on this one page, among them `scrollable-region-focusable` and `nested-interactive`); `triage.md` caveats, "the visualise model reachable only through the frame ... the second waits". Not observed: the frame's own "Domain as text" disclosure at 390 px (no shot of it; R-05 placed it inside the frame, `worst-2-desktop-1280-light.png` of R-05).
- regression asset proposed: an L1 test on the built `/ess/docs/visualise` that the page holds, outside the frame, a table of entities, states and commands whose entity count equals the presentation's, written by `cargo xtask site-data` and held by its `--check`
- sessions affected: every visit to `/ess/docs/visualise` and to the landing's graph; 2 of 139 pages; none of the 22 task runs reaches it, so no round metric sees it
- cost: M
- note: UXF-005 to UXF-008 are the templates package's and wait on its fix; this is the part the site owns. A decision that names the pin as owner of the 23 nodes belongs with it, not a hidden page.

### P2. `page-a-view/mobile-390` is over its limit for a reason the site owns, and it sits under a theme-owned finding

- title: A phone reader needs 11 actions and 8 steers to reach a guide from `/ess/docs` because the guides are behind a closed category and the first screen names none
- class: input
- severity: S2
- metric: `page-a-view/mobile-390` `screen_only.actions` 11 against its limit 5, `screen_only.steers` 8, `screen_only.time_to_act_steps.max` 6; `page-a-view/desktop-1280` is 4 of 5 (`report.md`, L3 table)
- found_by: L3 screen-only user (R-01 on), L7 heuristic reviewer (heuristics 3, 8, 10)
- evidence: `report.md` L3 table, rows `page-a-view` (11 (5), steers 8, looks 4, time to act 6; the largest of 22 runs on every column); `scenarios/page-a-view/` (start `/ess/docs`, route: expand "Guides", "Write a specification", "Views", heading); `shots/contact-light.png` `docs-intro` (four "Start here" links, none a guide) and `mobile-menu` (GUIDES as a collapsed chevron row among five); `findings/UXF-013.md` (R-04: the menu's first panel is the theme's, so the finding waits on the theme; R-06: `page-a-view` "keeps the asset: its route goes through a closed sidebar category").
- regression asset proposed: none new; the existing `L3 page-a-view mobile-390 pointer` test stays the asset, and the new record owns it so that UXF-013 can close on the two routes it fixed
- sessions affected: 1 of 22 task runs (`page-a-view/mobile-390`); the same route shape applies to any guide task on a phone
- cost: M
- note: UXF-013's cause for the other three phone routes was the theme's menu, and R-06 fixed them with an in-page link. This route's cause is different and the site's: a link from `/ess/docs` (or the "Where to go next" table, `website/docs/index.md:51`, below the first screen) to the guides' index would take it to about 4 actions. Filed apart so it is not parked behind an owner who cannot fix it. Caution carried from R-05: the link must be what a reader of this site wants, not the card's route.

## Observations without a record

- The R-06 change that made the status page's link start its line was made because the screen-only
  user clicks the centre of a link's box (`report.md`, notes, last item). A page edit made to suit a
  tool defect is fitting the site to the bot; the tool should click a point inside the link's first
  line box. For the report reviewer, not a site finding.
- The `worst-1` shot does not show what its caption says (no magenta outline in the frame). A
  scroll-to-first-node shot would make the "worst moment" read as a moment.
- UXF-019 closes on a test, not on a probe; the contact sheet at 1280 is consistent with the probe
  (primary and secondary actions at the bottom edge of the first frame).

## Changes I would put first in the next round

| # | change | serves | metric that should move | size |
| --- | --- | --- | --- | --- |
| 1 | P2: a link from `/ess/docs`'s first screen to the guides' index, and the "Where to go next" table moved above the code block | P2, UXF-013 (heuristics 3, 8) | `page-a-view/mobile-390` actions 11 to 5 or under; `reading_load.max` of the docs-intro scenarios must not rise | M |
| 2 | P1: the model as text on the visualise page, written by `cargo xtask site-data` | P1 (heuristics 3, 9, 15) | new L1 entity count; gate 5 nodes stay at 23 | M |
| 3 | The L2 assertion that "Get started" is wholly in view before any scroll at both gating viewports | UXF-019 (heuristic 1) | UXF-019 closes on a test | S |
| 4 | Sidebar category names and table headers at 14 px or more, after an L2 probe of their computed size | UXF-017 (heuristic 15) | new small-text count from about 70 pages to 0 | S |
| 5 | A keyboard path to a sidebar link, after a per-press focus log (the phone runs fell 131 to 81 with the in-page link; the desktop runs did not move) | UXF-015 (heuristic 9) | keyboard `screen_only.actions` of `first-conformance-run/desktop-1280` 23 down | M |
