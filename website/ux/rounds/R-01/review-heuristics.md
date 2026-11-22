# R-01 heuristic review

6 pass, 7 partial, 3 fail; 0 not observed (3 judged in part only: 11, 13, 16).

Evidence used: `report.md`, `round.json`, the 15 files under `shots/`, the 9 scenario cards and
the site source under `website/` at `df384afaa7`. No recordings and no operator notes exist for
this round, so no verdict cites a recording second; the theme's CSS (the docs-system package) is
not in the tree, so sizes are read from screenshot pixels. No new shot was taken.

| # | heuristic | verdict | evidence |
| --- | --- | --- | --- |
| 1 | One next thing | partial | Doc pages end with one "Next:" line and the next-page card (`uxf-002-desktop-1280-light.jpg`). At desktop-1280 the landing page's "Get started" is below the first screen, at mobile-390 it is in it (`contact-light.png`, landing frames; UXF-019). The docs introduction offers a six-row "Where to go next" table, below its first screen (`website/docs/index.md`, docs-intro frame). On Adopt ESS the "next step" column is out of view (UXF-020). |
| 2 | State in words | partial | Words: breadcrumbs, the "CONCEPTS · 3 OF 5" eyebrow and the "SHIPPED" badge (`uxf-004-desktop-1280-light.jpg`). Shape only: the current sidebar entry (tint and left bar), the navbar's current item (underline), a category's open state (chevron) and the theme mode (an unlabelled icon, every shot); the hidden mode is why the toggle's first press seems to do nothing (UXF-012). |
| 3 | Nothing required hidden | fail | No search on any page (UXF-001); "Verify conformance" inside a closed category (UXF-011); navbar links behind the menu at 390 px (UXF-013); cut terminal titles (UXF-009, `uxf-009-mobile-390-light.jpg`); a table column out of view (UXF-020, `uxf-004-desktop-1280-light.jpg`). |
| 4 | Stable layout | pass | `round.json` `site/load` `layout_shift.max` 0.0183; the eight contact-sheet frames keep the same positions in both themes (`contact-light.png`, `contact-dark.png`). The sidebar pushed up at the end of a page is counted under 16. |
| 5 | Provenance of every value | partial | Generated values name their source: "Source: examples/billing, compiled by `ess specify compile`" (`uxf-005-desktop-1280-light.jpg`), "generated from the change records" (`worst-1-desktop-1280-dark.png`), "Edit this page" (`uxf-002-desktop-1280-light.jpg`). The Status page's "latest release" is 0.51.0 against 0.97.1 on the install page (UXF-016). The footer's "THIS BUILD docs-system 5fd348a" names the theme's revision, not the ESS release or commit the pages describe (`uxf-002-desktop-1280-light.jpg`). |
| 6 | Errors cheap | pass | A wrong page is one Back or one breadcrumb; a wrong category is one click; the theme reaches either mode within 2 presses (UXF-012); a wrong mobile menu panel is one "Back to main menu" (UXF-013). All 22 task runs completed (`report.md`, Screen-only user table). |
| 7 | Words the user can say | partial | Navbar labels differ from the heading they open: "Example" opens "A specification and its contracts", "Status" opens "Where this stands", "Visualise" opens "Visualise a specification", which the sidebar calls "Visualise the billing model" (`website/docusaurus.config.ts`, `website/docs/visualise.md`, `uxf-007-desktop-1280-light.jpg`). 3 of 68 sidebar labels differ from their page's heading (binding-conditions, selection-effects, visualise). The theme toggle has no visible word. Graph nodes' names do not contain their label (UXF-005). No measured time cost, so no new record. |
| 8 | The reader's time respected | fail | `ess-31-release` takes 26 and 35 actions against 6 (UXF-010); 4 of 9 pointer tasks at desktop-1280 and 7 of 9 at mobile-390 exceed their card's expectation (`report.md`, Screen-only user table; UXF-011, UXF-012, UXF-013). |
| 9 | Keyboard and pointer parity | partial | Every keyboard run completes (4 of 4, `report.md` gate "5 keyboard only"), at about ten times the pointer's actions: 21 to 26 against 2 to 4 (UXF-015). On the visualise page a nested control and an unfocusable scroll region (UXF-007, UXF-008). |
| 10 | Recognition over recall | partial | Recognition helps: sidebar categories, breadcrumbs and the table of contents are in view on every desktop doc page (`worst-2-desktop-1280-dark.png`). Recall needed: without search the reader must know which category holds a page (UXF-001, UXF-011); at the `ess/31` row the column header "Introduced in" is 19,000 px up (UXF-018, `uxf-010-desktop-1280-light.jpg`). |
| 11 | Calm by default | pass | 0 pages animate or scroll smoothly under reduced motion (`report.md`, gate "reduced motion"); no banner, toast or alert in any frame of either contact sheet; the visualise demo waits at "step 0/20" until Play (`uxf-007-desktop-1280-light.jpg`). In part only: motion without the reduce preference is not measured. |
| 12 | One state vocabulary | pass | One set of status words: front matter `status:` holds only `shipped` (5) and `planned` (2), the Status page lists capabilities as `shipped` and `planned`, the product as "experimental" (`website/docs/status/where-this-stands.md`, `website/product.json`); the badge reads "SHIPPED" (`uxf-004-desktop-1280-light.jpg`). |
| 13 | A wrong cue recovered in one action | pass | Navigation: a wrong category, page or menu panel is left with one action (see 6), and the toggle's no-change first press is recovered by one more press (UXF-012). In part only: the search half cannot be judged, the site has no search (UXF-001, counted under 3). |
| 14 | Unmistakable end state | pass | Gate 3: `completion_miss` 0, `false_complete` 0; 18 of 18 gate task runs complete (`round.json` `gate3_unmissable_completion`, `gate4_screen_only`); every task ends on a page whose heading names the target (scenario cards). |
| 15 | Readable in every theme; key data at 14 px | fail | Links told apart by colour only on 102 pages, both themes (UXF-004, `worst-1-desktop-1280-dark.png`); dark heading on the dark green banner of the release archive (UXF-006, `uxf-006-desktop-1280-light.jpg`); sidebar category names and column headers at about 11 px (UXF-017, `uxf-004-desktop-1280-light.jpg`). |
| 16 | Focus follows the work | partial | The sidebar opens on the current page and marks it, also in the mobile menu (`contact-light.png`, mobile-menu frame); the table of contents marks the section in view (`uxf-002-desktop-1280-light.jpg`). At the end of a doc page or release post the sidebar is pushed under the navbar or out of view (UXF-002, UXF-003). In part only: where keyboard focus lands after a sidebar click is not recorded; a keyboard-run recording or an L2 focus probe would show it. |

## Findings filed

| id | title | severity | class | sessions | cost |
| --- | --- | --- | --- | --- | --- |
| UXF-015 | With the keyboard a sidebar link takes about 20 Tab presses, ten times the pointer's actions | S3 | input | the 4 keyboard runs (first-conformance-run, install-linux-x86_64, both viewports) | M |
| UXF-016 | The Status page names 0.51.0 as the latest release; the install page installs 0.97.1 | S2 | copy | none: no card visits the Status page; found in the source | S |
| UXF-017 | Sidebar category names and table column headers are set at about 11 px | S3 | layout | every desktop doc page; no task metric | S |
| UXF-018 | The column headers of a long reference table scroll away; at the ess/31 row nothing says what 0.69.0 is | S3 | layout | ess-31-release at both viewports | S |
| UXF-019 | At desktop-1280 the landing page's "Get started" is below the first screen | S3 | affordance | none: the landing-page cards go through the navbar | S |
| UXF-020 | A six-column table hides its last column, "next step", at desktop-1280 | S3 | layout | none: no card visits the page; seen in a gate-5 shot | S |

Existing records cited instead of new ones: UXF-001 to UXF-013. UXF-014 (the gate's run time) is
a defect of the loop, not of the site, and enters no verdict.
