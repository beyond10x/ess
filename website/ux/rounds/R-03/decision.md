# R-03 decision: does this site get a search of its own in the next round?
Finding: UXF-001. Target metric: template pages without a search control (the L2 site-search check), now 9 of 9, expected 0 of 9.
Yes: R-04 adds a local index plugin to `website/` (an index built at `docusaurus build`, no network, no keys) with its field in the navbar; the adapter's search entry becomes an always-visible element and gate 2 holds it on every merge. shot: shots/decision-yes.png (the field mocked into the built page, not built)
No:  the navbar stays as it is; readers find pages through the navbar, the sidebar and in-page links, as the screen-only user does for every card; UXF-001 stays open and its asset stays a known failure until search arrives through the shared template. shot: shots/decision-no.png
Default if unanswered by the next round: no, because search belongs to the shared documentation layer, not to this repository (facts 1 and 3): a plugin of this site's own would be a second search beside the one the owners would ship, a dependency of 35 packages in this site's build, and the template's own decision on the same question is open with default no.
Answer:

## Facts, read on 2026-11-22

1. `@beyond10x/docs-system` ships no search this site could switch on, neither at the commit
   `website/package.json` pins (`5fd348a9e49a96832f70af7917e6cd2db2c32abe`, 2026-10-22) nor at its
   newest commit (`da660f7b9108c8b310b8e2365c5160d2b7fdf204`, 2026-11-22, read through
   `git ls-remote` and a shallow clone). `withProductSite` adds Prism themes, fonts, the Mermaid
   override and its own plugin, and no search plugin or theme. Its `SearchField` component is an
   input for discovery views inside a page, not a site search. Its own UX loop records the same
   gap as its UXF-005 ("the product-site template ships no search", S2), asked in its R-01
   decision ("Should `withProductSite` add a built-in, offline search field to the navbar of every
   product site?"), unanswered, default no.
2. A local index plugin: `@easyops-cn/docusaurus-search-local` 0.55.3, licence MIT, 615,867
   downloads in the week 2026-11-15 to 2026-11-21 (npm's download count), unpacked size 190 kB,
   17 direct dependencies and peer dependencies on `@docusaurus/theme-common`, React and
   `open-ask-ai`. Added to a scratch copy of `website/package-lock.json`
   (`npm install --package-lock-only`, nothing added to `website/package.json`), it adds 35
   packages to the lock's 1,463: lunr and its language packs, cheerio, undici, and the native
   Chinese word splitter `@node-rs/jieba` with 15 platform builds, one of them installed per
   machine.
3. `AGENTS.md` § "Public documentation operations": Atlas selects each source's bundle, and
   "Website plus Docs System own rendering, shared components, search, and feeds". This
   repository owns its source and the allowlist in `b10x.docs.yaml`, not search.

On a yes the plugin's index step runs in `task site-build` and in `pages.yml`; on a no the request
goes to the template's owner, where UXF-005 of its loop already holds it.

## Ratchet decision `R-03-axe`: the visualise page's axe values rise with the repaired measurement

Change 1 (UXF-021) made gate 5 check inside frames again. The only page with a frame,
`/ess/docs/visualise`, now reports the violations inside its frame that R-02 never saw. Four values
are worse than the R-02 baseline, all in the light theme, where the full rule set runs:

| scenario | key | R-02 baseline | R-03 |
| --- | --- | --- | --- |
| `/ess/docs/visualise@desktop-1280/light` | `axe.serious_critical_nodes` | 3 | 10 |
| `/ess/docs/visualise@desktop-1280/light` | `axe.serious_critical_rules` | 1 | 3 |
| `/ess/docs/visualise@mobile-390/light` | `axe.serious_critical_nodes` | 2 | 11 |
| `/ess/docs/visualise@mobile-390/light` | `axe.serious_critical_rules` | 1 | 4 |

R-02's 3 and 2 nodes were `link-in-text-block` outside the frame, which change 4 removed. R-03's
nodes are inside the frame: `color-contrast` 8 (UXF-006), `label-content-name-mismatch` 1
(UXF-005), `nested-interactive` 1 (UXF-007), and at mobile-390 `scrollable-region-focusable` 1
(UXF-008). The site did not change there; the measurement did. Taken with
`cargo xtask ux-ratchet --round website/ux/rounds/R-03/round.json --update --decision R-03-axe`.

Decided by the coordinator in the brief of R-03, 2026-11-22.

Result, 2026-11-22: 819 values improved and were taken, 4 worse values carry `R-03-axe`, none
dropped; the baseline holds 3,556 values in 578 scenarios, lane `full` and lane `gate` from R-03
(the gate lane checked with `gate-5`, `--update`, and `gate-6`, clean).
