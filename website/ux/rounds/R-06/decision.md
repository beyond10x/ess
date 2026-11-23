# R-06 decisions

## Coverage decision `R-06-gate-kinds`: the gate lane gives up two page kinds to the slow lane

Finding: UXF-025. Target metric: the gate step's wall time against its 120 s budget.

The diagnosis of R-05 (`../R-05/diagnosis-gate-time.md` § 4) shows that the four browser tests of
the landing page and the visualise page are about a third of the gate's test-seconds. Its probe,
run in this round (`probe.json` beside this file: three interleaved pairs, the gate with and
without the four tests), did not refute it: the run without them took 0.81, 1.23 and 0.31 of the
run with them, above 0.85 in one pair of three (that pair's B run started at a load 27 higher
than its A run), and no B run took over 110 s. Every run started above a load of 60, so the probe
is taken above load 60. They are the regression assets of open findings
owned by pinned packages (UXF-005 to UXF-008: the templates package's frame and the shared docs
theme's graph), so in the gate they can only fail as expected; the gate exists for new failures.

The gate lane gives up these two page kinds:

- `landing`
- `doc-without-sidebar`

The slow lane keeps them, in these tests of `specs/l2-layout.spec.ts` (every page of the
sitemap at both gating viewports, in its six slices):

- `L2 /ess/ desktop-1280`
- `L2 /ess/ mobile-390`
- `L2 /ess/docs/visualise desktop-1280`
- `L2 /ess/docs/visualise mobile-390`

What the gate no longer checks on every merge: a layout, accessibility or always-visible
regression that only the landing page or a doc page without a sidebar shows. Every always-visible
element both kinds show (the navbar, the theme toggle at desktop, the menu button on the phone)
stays in the gate on the other three kinds. The L1 test of the template pages holds this list
against `SLOW_LANE_KINDS` in `adapter.ts`.

Default if unanswered by the next round: yes, because the four tests can only fail as expected
in the gate while their findings are open.
Answer: yes (coordinator's brief of R-06, change 1), 2026-11-23
