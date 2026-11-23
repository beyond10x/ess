- The gate lane runs 13 browser tests (change 1, UXF-025), about half of R-04's. It still does not
  hold its 120 s on a loaded host: `gate-5`, `gate-6` and `gate-7` started at a 1-minute load of
  133.47 to 169.55 and were cut at 116.2 to 117.2 s of wall time (status `timedout`, no new
  failure); `gate-8` started at 60.06 and passed in 19.5 s. UXF-025 records three earlier runs on
  the engine of change 1, which this `round.json` does not hold. UXF-025 stays open.
- The slow lane's nine jobs ran at 55.92 to 131.15. One test timed out, `L2
  /ess/docs/reference/ess-ui desktop-1280` in `slow-l2-5` (load 78.21), and passed in
  `slow-rerun-a`; the ratchet takes `slow-l2-5` because a later run of the round passed the test
  it failed (T41).
- Truncated texts went from 6 to 0 and the release archive's contrast nodes from 4 to 0 (change 4,
  UXF-009 closed, UXF-006's archive part); gate 5 still fails on the visualise page's frame and the
  landing page's graph (UXF-005 to UXF-008), owned by the templates package and the shared docs
  theme.
- The index lists are named navigations (change 5, UXF-027). `glance_cost.p95` of
  `ess-31-release/mobile-390` and `diagnostic-type-009/mobile-390` stays 3: on a phone the first
  step opens the menu, and that step is already outside the primary region. UXF-027 stays open on
  its metric.
- Eleven reading-load values are worse than the baseline beyond their tolerance and carry
  `R-05-worse` (`decision.md`); nine of them come from `main`'s text on the docs introduction.
  Fourteen more are worse inside their tolerance; the ratchet names each in a note (T36).
- `layout_shift.load` and `motion.running` are filed under the light configuration (T34): 556 dark
  values dropped and 556 light values new under `R-05-worse`. The two R-04 shifts taken under load
  (`../R-04/decision.md`) measure 0 under light, and in `slow-rerun-b` at load 55.92. The landing page's
  one shift of 0.0948 in `slow-l2-1` at load 131.15 measured 0 in `slow-rerun-b`, and `R-05-worse`
  takes that 0 by name.
