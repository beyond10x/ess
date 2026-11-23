# R-05 heuristic review

7 pass, 7 partial, 2 fail, 0 not judged (judged in part only: 6, 11, 12, 13, 16).

R-04 was 6 pass, 8 partial, 2 fail. One verdict moved: heuristic 5 (provenance), partial to pass,
because the one contradiction behind it (UXF-016: status page 0.51.0, install page 0.97.3) is gone
and a test holds it. Nothing else moved; no verdict got worse.

Evidence used: `website/ux/rounds/R-05/report.md`, `round.json`, `triage.md`, `decision.md`,
`notes.md`, the 4 files under `website/ux/rounds/R-05/shots/` (read as images), `website/ux/rounds/trend.md`
is absent and `website/ux/trend.md` was read instead, findings UXF-005, 006, 009, 012, 013, 015, 016,
017, 027, 028, `website/ux/rounds/R-04/review-heuristics.md`, the diff `4150f0a494..2b0c77ce84` of
`website/src/css/custom.css`, `website/docs/index.md`, `website/docs/reference/spec-versions.md`,
`website/docs/reference/diagnostics.md`, `website/docs/status/where-this-stands.md`,
`website/docs/start/install.md`, `website/docs/status/limitations.md`, `website/docusaurus.config.ts`,
`website/product.json`, `website/docs/visualise.md`. I ran no command that builds or serves and took no
shot. `website/node_modules` is absent, so the shared theme's own CSS was not read; claims about it
are marked as source reading.

What the 4 shots show: `contact-light.png` / `contact-dark.png` (landing, docs-intro, install,
cli-middle, release-post-index, mobile-landing, mobile-doc, mobile-menu), `worst-1-mobile-390-light.png`
(visualise, top of page) and `worst-2-desktop-1280-light.png` (a domain graph, the landing's). None
shows a terminal title at 390 px wrapped, the release archive page, or either reference index page.
Judgements on those three rest on `round.json` values and source, and say so. No recording, person or
questionnaire exists for the round (`report.md`, "Missing inputs").

## The four questions

### 1. Did the merge of `main`'s reworded pages make a page worse?

Verdict: no page is measurably worse by a heuristic; one added line does not earn all its words.

- What the metrics can see: gates 1 to 3 and truncation pass; the two reference pages the merge
  rewrote most (`spec-versions` 818 changed lines, `predicates` 791, `cli` 321) hold axe 0 nodes in all
  4 configurations each (`round.json`, scenarios `/ess/docs/reference/*`). Reading load is measured
  only on the pages 22 task runs pass through, so about 30 of the 38 reworded pages have no
  reading-load value at all. I read `index.md`, `limitations.md`, `where-this-stands.md` and
  `install.md` in the diff; the other 34 I did not read.
- The first screen of `/ess/docs` (`website/docs/index.md:18`): "Its first level is four areas:"
  became "Its first level is five areas (`specify`, `generate`, `verify`, `infra`, `ui`); the first
  four are shown:". 16 more words on a screen of 205 (8 %), above a code block that still shows four
  commands (`contact-light.png`, `docs-intro`, which is 3 lines of lead, then the code block). It
  earns the correction: the old line was false, and the five names are the vocabulary the CLI
  reference now uses (`website/docs/reference/cli.md:12`, "the five areas"). It does not earn the
  form: it names five, shows four, and gives the reader of `ui` nothing to click. A link to the
  reference's area list would say the same in fewer words and name the next step (heuristic 1).
- This line is the whole of the nine docs-introduction reading-load rises under `R-05-worse`
  (`decision.md`): +16 words on `first-conformance-run`, `install-linux-x86_64`, `go-runner`,
  `page-a-view`, `limitations` at desktop-1280 (205, 205, 214, 258, 176). The other two of the 11 are the
  index lists (`diagnostic-type-009/desktop-1280` and `ess-31-release/desktop-1280` medians 201 to
  217, and `ess-31-release/mobile-390` max 164 to 177), where the added text is the family
  scope sentence and the class names: that text is function, not prose.
- Better, by evidence in the diff: `limitations.md` corrected a wrong statement (`--target
  interpreted` listed views, bindings, time and redelivery as unsupported; the page now lists them as
  run), the Rust runner line says what to depend on instead of "a checkout" (`limitations.md:26`),
  semicolons became sentences, and `ess-31-release/desktop-1280 reading_load.max` fell 358 to 322 and
  `diagnostic-type-009/mobile-390` 165 to 139 (`decision.md`). Heading renamed once ("The four command
  areas" to "The five command areas"), no link in the tree names the old anchor (grep of
  `website/docs`, `website/src`).
- A cost the merge left: the three longest reference pages are still 1,368 (`spec-versions`), 1,931
  (`cli`) and 2,169 (`predicates`) lines of Markdown in one scrolling page each; the reading-load
  metric does not see this, and it is where a reader needs the index lists most.

### 2. Do the wrapped terminal titles and the archive heading read well in both themes?

Verdict: they measure clean in both themes; I cannot confirm they read well, because no shot shows
either.

- Terminal titles (`website/src/css/custom.css:28-35`): `.b10x-terminal__bar > .b10x-terminal__title`
  now wraps with `overflow-wrap: anywhere`. Measured: `gates.truncation` 6 texts on 2 pages to 0 on 0
  pages, in both themes at both gated viewports (`report.md`, Gates). The rule sets no colour, so it
  cannot differ by theme; the contact sheets show the landing terminal's title in one line in both
  themes at 1280 and a short title at 390 (`mobile-landing`, both sheets), so the wrapped case itself
  is not in a frame. Source reading: `anywhere` lets a long path such as `examples/billing/invoice.yaml`
  break at any character, not after a slash, so a wrapped title can split a word; that is legible and
  cheaper than a cut title, and the bar grows by a line, which gate 1 (0 spills) and `layout_shift`
  do not flag.
- Archive heading (`custom.css:37-40`): `.hero--primary` takes `--b10x-color-on-accent` for text and
  heading. Measured: `/ess/releases/archive` `axe.serious_critical_nodes` 0 in all 4 configurations
  (light and dark, 1280 and 390), 4 nodes in R-04; gate 5 total 31 to 27. In R-04 the dark theme drew
  white on a mint band (`R-04 shots/worst-3-desktop-1280-dark.png`), the worse of the two themes;
  both are now 0. Caveat: axe reports text over a gradient or image as "incomplete", not as a
  violation, so 0 nodes is evidence the band's colour is computable and passes, not proof of the look;
  one shot of the archive page in each theme would settle it. The "mint band under a dark page" seam
  that R-04 named is not touched by this rule.

### 3. The index lists as a labelled navigation: usable by keyboard and screen reader?

Verdict: better, and sound as a landmark; the list is still one long run of tab stops with no list
semantics. Source reading throughout (no shot shows either page).

- Landmark and name: `website/docs/reference/spec-versions.md:11` and `diagnostics.md:10` wrap the
  paragraph in `<nav aria-label="Versions of the ess/ format">` and
  `<nav aria-label="Classes of the specification diagnostics">`. A screen reader lists them in its
  landmark list with a name, which R-04 lacked. The blank lines inside the `nav` make Markdown emit
  `<p>` inside it, so the content is one paragraph; the generated-by marker above it is a
  link-reference definition and renders nothing.
- No list semantics: 37 and 19 anchors separated by ` · ` in a paragraph. A screen reader gives no
  "list of 37 items", no item position and no next-item key, and may read each middle dot; a `ul`
  with the dots removed by CSS would give all three. The numbers are `ess/31` in a `<code>`, read as
  "ess slash 31", which a reader can say (heuristic 7), and the diagnostics list now carries the class
  name beside the number (`diagnostics.md:12`, `009 UNSUPPORTED`).
- Keyboard: Tab reaches each anchor and Enter jumps; the focus starting point moves to the target
  (browser behaviour), so the next Tab continues from the row. There is still no way past the list but
  37 Tab presses (19 on the other page) or the heading keys; `ess/31` is 31 presses in. The
  navigation landmark helps screen-reader users who jump by landmark, not keyboard-only users.
  Heuristic 9 stays a fail for this reason and the anchor cards (below).
- Landing (`custom.css:44-54`): `.markdown a[id]:empty` gets `scroll-margin-top` of navbar height + 1rem,
  so a jump no longer lands under the sticky navbar (R-04's open question), and
  `.markdown tr:has(a[id]:target)` outlines the row (2 px, primary colour). The two rules are separate,
  so a browser without `:has()` (before Chrome 105, Firefox 121, Safari 15.4) loses the mark and keeps
  the margin. The mark is visual only: nothing announces the target to a screen reader, and the
  outline's contrast against both themes' rows was not measured. Outline on a `tr` is supported in the
  engines the test runs (L1 test, `UXF-027`), but I did not see it drawn.
- Measured: both pages 0 axe nodes at both viewports; `ess-31-release/mobile-390` and
  `diagnostic-type-009/mobile-390` still 4 actions at `glance_cost.p95` 3 (`report.md`, Screen-only
  user), as `notes.md` says, because the phone's first step is the menu.

### 4. What can the site still do about what the builder could not move?

Within the limits (phone menu is the theme's, UXF-013; the visualise frame is the pin's, UXF-005 to
008): the site owns page text, order, `website/product.json` (landing), `docusaurus.config.ts`, the
sidebar and `custom.css` rules on class names. Findings, not decisions:

- Phone menu: the glance class is set by the first step being the menu, so the only lever is fewer
  menu steps. The two cards that start on the landing (`first-conformance-run`, `limitations`) pay for
  "menu, back to main menu, Docs, page" (4 of 3). The landing's hero has two buttons, "Get started"
  and "See the model" (`product.json:10-11`); a link from the landing or `/ess/docs` straight to the
  page a card asks for is one tap. Cost: more first-screen words and a risk of fitting the site to the
  cards, so the list must be what a reader of this site wants (install, first conformance run, limits),
  not the cards' list. `page-a-view` (11 of 5) starts at `/ess/docs`, whose "Where to go next" table
  is below the first screen (`docs/index.md:49`).
- Visualise: 23 of 27 serious nodes (10 light desktop, 11 light mobile, 1 + 1 dark: `round.json`),
  18 of them contrast and light-theme only; the landing's graph holds the other 4 (2 light nodes at
  each viewport). The page's own text is not at fault. What the site can do: say at the top what the
  frame is and that a text version exists, and give a reader without the frame the model as text. The
  `Domain as text` disclosure is inside the frame (`worst-2-desktop-1280-light.png`), so a reader who
  cannot use the frame has it only through the frame. Holding the page out of the navbar or footer
  would hide the finding and the page together (a heading 3 trade: the headline feature behind no
  link); I advise against it and for a decision that names the pin as owner.
- Reading load and order: the three longest pages and the index lists are where order matters; the
  four task cards do not measure them.
- Not available to the site: re-colouring the frame (a token override of a pinned package is an
  override).

## Verdicts

| # | heuristic | verdict | evidence |
| --- | --- | --- | --- |
| 1 | One next thing | partial | Landing at 1280 has no button in the first screen (`shots/contact-light.png`, `landing`; UXF-019); at 390 two are (`mobile-landing`). Docs intro has one named step ("Verify conformance") and now a line that names five areas and shows four (`website/docs/index.md:18`), a second fork with no link. |
| 2 | State in words | partial | Unchanged: theme mode is an unlabelled half-disc icon in every frame (`contact-dark.png`, top right); current sidebar item by tint (`install`, `cli-middle`); `theme-dark-and-back` 3 of 2 and 4 of 2 (UXF-012). Gain: an index jump now marks its row by `:target` outline (`custom.css:52`, source only). |
| 3 | Nothing required hidden | fail | No search (UXF-001, default no). Phone menu opens on the docs panel: `first-conformance-run/mobile-390` 4 of 3, `limitations/mobile-390` 4 of 3, `page-a-view/mobile-390` 11 of 5 (`report.md`; UXF-013; `contact-light.png` `mobile-menu`). Truncation 6 to 0 (`gates.truncation`), the one gain. |
| 4 | Stable layout | pass | Gate 1: 0 spills, 0 page scroll, 139 of 139. `layout_shift.load` filed under light now (T34): 0.014 at 1280, 0.028 at 390; one 0.095 landing shift measured at load 131 and 0 at load 56 (`report.md`, "Values a rerun replaced"; UXF-026). In part only: the wrapped terminal title grows its bar by a line, unmeasured. |
| 5 | Provenance of every value | pass (was partial) | Status page and install page both name 0.97.4 (`where-this-stands.md:13`, `install.md:34`) and a test holds it (`l1-site.spec.ts`, UXF-016 closed); the two index pages and the status record carry a generated-by marker; the wrong `--target interpreted` claim is corrected (`limitations.md`). Caveat: the version is typed by hand in 13 places of `install.md` and 3 of the status page, and the test goes red at the next release rather than preventing the drift. |
| 6 | Errors cheap | pass | A wrong index jump is Back; a wrong phone panel is one "Back to main menu" (`contact-light.png`, `mobile-menu`); 22 of 22 L3 runs complete (`report.md`, Gates 3 and 4). In part only: no run injects a wrong turn. |
| 7 | Words the user can say | partial | Navbar "Example" and "Visualise" against page titles "Specification to contracts" and "Visualise a specification", sidebar "Visualise the billing model", footer "Visualise the billing model" (`docusaurus.config.ts:70-95`, `visualise.md:3`). Gain: the diagnostics index names `009 UNSUPPORTED`. Graph nodes still lack their label in their accessible name (UXF-005; `label-content-name-mismatch`, 6 nodes). |
| 8 | The reader's time respected | partial | Gate 4: still 5 entries over their limit, all but `theme-dark-and-back/desktop-1280` on the phone (`report.md`). `ess-31-release/desktop-1280` reading load 358 to 322, `diagnostic-type-009/mobile-390` 165 to 139; docs intro +16 words on 9 values (`decision.md`). The gate lane still ran 3 of 4 gate runs past 120 s at load 133 to 170 (`report.md`, Runs: `gate-5`, `gate-6`, `gate-7` `timedout`; UXF-025). |
| 9 | Keyboard and pointer parity | fail | Keyboard 23 and 131 against 2 and 4; 21 and 129 against 2 and 4 (`report.md`, Screen-only user; UXF-015), unchanged. The index lists add 37 and 19 tab stops with no skip. `nested-interactive` 2 nodes and `scrollable-region-focusable` 1 node on the visualise page (`worst-1-mobile-390-light.png`; UXF-007, UXF-008). |
| 10 | Recognition over recall | partial | An index turns "scroll and recognise a row" into "tap the number you hold", and the class names now sit beside the numbers (`diagnostics.md:12`). Still recall: no search, 37 versions and the other format families without an index (`spec-versions.md:11` says so and does not give one), and the 77 diagnostic codes under the 19 classes. |
| 11 | Calm by default | pass | `reduced_motion` 0 of 139 pages animate; `motion.running` 0 in all 278 light configurations. No banner or alert in any of the 16 contact-sheet frames. In part only: motion without the reduce preference is not measured. |
| 12 | One state vocabulary | pass | Landing eyebrow "EXECUTABLE SYSTEM SPECIFICATIONS · EXPERIMENTAL" in both themes (`contact-light.png`, `contact-dark.png`, `landing`); one "Latest release" wording on the status page. In part only: no frame shows a status pill or the status page. |
| 13 | A wrong cue recovered in one action | pass | `theme-dark-and-back/desktop-1280` 3 for 2: the no-change first press is recovered by the next. A wrong menu panel is one "Back to main menu". In part only: the search half cannot be judged. |
| 14 | Unmistakable end state | pass | `completion_miss` 0, `false_complete` 0; 18 of 18 pointer and 4 of 4 keyboard runs done (`report.md`, Gates). |
| 15 | Readable in every theme; key data at 14 px | partial | `color-contrast` 18 nodes, all on `/ess/docs/visualise` in light (`round.json`); archive heading 4 to 0 nodes (`custom.css:37-40`; not in a shot). Dark is legible in all 8 dark frames (`contact-dark.png`). The 8 to 10 px-cap uppercase sidebar categories and table headers are unchanged and unmeasured (`contact-light.png`, `cli-middle`; UXF-017). |
| 16 | Focus follows the work | partial | Gate 2 failures 0 at the gated states; `end` state out of view on 135 docs and 136 blog configurations (`report.md`, gate 2, accepted under D15); 1 unnamed page at desktop-1440 (`report.md`, measured-only; UXF-028). An index jump now lands below the navbar and is marked (`custom.css:44-54`, source only). The sidebar's bottom edge is still not checked. |

## Totals

- pass: 4, 5, 6, 11, 12, 13, 14 (7)
- partial: 1, 2, 7, 8, 10, 15, 16 (7)
- fail: 3, 9 (2)
- not judged: none; judged in part only: 6, 11, 12, 13, 16

## Findings filed

None filed (read-only brief). Every fail and every costly partial has a record (UXF-001, 005 to 008,
012, 013, 015, 017, 019, 025 to 028). Three observations have none; they feed changes 2, 4 and 5
below as "new" and belong in the next triage:

| id | title | severity | class | sessions | cost |
| --- | --- | --- | --- | --- | --- |
| new | The index lists have no list semantics: a screen reader hears one paragraph and the middle dots | S3 | affordance | `ess-31-release`, `diagnostic-type-009` (2 of 22 task runs pass through one) | S |
| new | A reader who cannot use the visualise frame has the model as text only through the frame (`Domain as text` is inside it) | S2 | affordance | every visit to `/ess/docs/visualise` and the landing graph (2 of 139 pages) | M |
| new | A release changes the version in 16 hand-typed places and the guard test only goes red afterwards | S3 | provenance | `install-linux-x86_64`, every release | S |

## Changes I would put first in the next round

All are site source (Markdown, generator text, `product.json`, `custom.css`). None replaces a theme or
package component.

| # | change | serves | metric that should move | size |
| --- | --- | --- | --- | --- |
| 1 | The landing and `/ess/docs` lead with the three next steps a reader of this site wants (install, first conformance run, limitations) as in-page links; the five-areas parenthetical on `docs/index.md:18` becomes one link to the CLI reference's area list | UXF-013, UXF-019 (heuristics 1, 3, 8) | `first-conformance-run/mobile-390` and `limitations/mobile-390` `screen_only.actions` 4 to 3 (card limit); `first-conformance-run/desktop-1280` `reading_load.max` 205 to 205 or lower (the 16 words go out as the link goes in); needs the L3 bot to take an in-page link, to be checked first | M |
| 2 | Give the visualise page's reader the model without the frame: a note under the title saying what the frame is, and a Markdown table of entities, states and commands written by `cargo xtask site-data` from the same presentation file and held by its `--check`; record a decision that names the pin as owner of the 27 nodes instead of hiding the page | new (follows UXF-005 to UXF-008) | none in axe (the frame keeps its 23 nodes); proposed L1: the page has a text table whose entity count equals the presentation's; `gates.gate5_accessibility.nodes` 27 stays | M |
| 3 | Set the sidebar's category names and the table column headers to at least 14 px with a `custom.css` rule on `.menu__link--sublist` and `th`, after an L2 probe of their computed size | UXF-017 (heuristic 15) | new `layout.small_key_texts` from about 70 pages to 0; `reading_load` must not rise | S |
| 4 | One source for the release number: the install page, the status page and the rust runner page read the latest version from one value in `website/data` (the status page already reads `status.json`), so a release is one edit and the L1 equality test guards a generator, not a typist | new (follows UXF-016, closed) | L1 count of hand-typed occurrences of the version in `website/docs` 16 to 0 (13 in `install.md`, 3 in `where-this-stands.md`); the equality test stays green across a release | S |
| 5 | Render the two index lists as `ul` (dots removed by CSS) in the generators, and extend the reference pages' coverage to the other format families of `spec-versions` and to the diagnostic codes under their class | UXF-027, new (heuristics 9, 10) | new L1: each index is a list of the expected length; `ess-31-release` and `diagnostic-type-009` `reading_load.max` must not rise above 322 and 139 | M |

Next in line, not in the five: the keyboard path to a sidebar link (UXF-015, which needs a per-press
log of focus before any change), the theme toggle's first press (UXF-012), the missing search
(UXF-001, default no), the one unnamed desktop-1440 page and the sidebar's bottom edge (UXF-028), and
one name per page for "Example" and "Visualise" (heuristic 7).
