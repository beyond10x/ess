# R-02 decision: must the sidebar be wholly in view at the end of a page?
Finding: UXF-002, UXF-003 (tool defect T4). Target metric: gate 2 `always_visible.failures` per page configuration at desktop-1280, R-01 271 failures (136 docs sidebar, 135 blog sidebar), all in the `end` state; expected 0 gated, the `end` failures reported as measured only.
Yes: gate 2 checks the docs and blog sidebars at the top, the middle and the end of every page; every doc or release page longer than a screen fails at the end, because the stock theme's sticky sidebar column stops where the footer starts. shot: shots/decision-yes.png
No:  gate 2 checks the sidebars at the top and the middle; the end state is still measured and printed in the report's gate 2 row as "measured only, not gated". shot: shots/decision-no.png
Default if unanswered by the next round: no, because at the end the reader is at the footer and the pushed-up sidebar is the theme's design, while at the top and the middle a lost sidebar would be a real regression.
Answer: no: top and middle only, 2026-11-22 (D15, the coordinator)

## Ratchet decision `R-02-baseline`: the baseline is replaced by R-02's values

R-01's baseline (`baseline.json` before R-02) was written by tools the review of R-01 found unfit
(`../R-01/review-report.md`, T1 to T15): an unmeasured page counted as clean, per-site counts that
moved with coverage and content, search counted 548 times, the sidebar gated at the footer, and a
screen-only user that read the DOM instead of the screen. Its values are not a base any later round
can be held to.

R-02 replaces the whole baseline with its own full-coverage run (`round.json` of this round,
lane `full`), through `cargo xtask ux-ratchet --round website/ux/rounds/R-02/round.json --update
--decision R-02-baseline`: worse values carry the decision id, and every R-01 scenario R-02 no
longer measures (the per-site `site/*` keys, replaced by one scenario per page configuration) is
listed under `dropped` with it. The gate lane's scope is then taken from the gate's own run
(`--round website/ux/.out/gate-7/round.json --update`, the first of the three timed gate runs).

Decided by the coordinator in the brief of R-02, 2026-11-22.

Result, 2026-11-22: the baseline holds 3,556 values in 578 scenarios (556 page configurations, 22
task runs); 23 values worse than R-01's carry `R-02-baseline`, and R-01's 26 lost values are listed
under `dropped` (5 entries). The gate lane holds 44 scenarios, taken from `gate-7`; `gate-8` and the
round check clean against it.
