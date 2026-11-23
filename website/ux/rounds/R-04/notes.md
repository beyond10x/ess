- The host's load decided this round's times. `gate-9`, `gate-10` and `gate-11` ran at a
  1-minute load of 115.29 to 140.34 and were cut at the 120 s bound (118.2, 117.7 and 118.3 s of
  wall time, status `timedout`), with no new failure. `gate-12`, `gate-13` and `gate-14` ran at
  20.01 to 71.61 and passed in 35.1, 17.9 and 51.5 s. UXF-025 stays open: the gate does not hold
  its bound on a loaded host.
- The slow lane's six L2 jobs ran at 117.83 to 170.96; their timed-out tests ran again in
  `slow-rerun-a` and `slow-rerun-b` at 54.25 to 68.73 and passed (24.6 and 5.8 s of wall time).
- Gate 4 lists 5 runs over their card's limit. `ess-31-release`, `diagnostic-type-009` and
  `target-process` left the list (changes 4 and 5); the phone navigation of
  `first-conformance-run`, `limitations` and `page-a-view` is the shared theme's (UXF-013).
- Gate 2 passes: the sidebar stays in view in the middle state on short pages (UXF-022).
- Nine values are worse and carry `R-04-worse` (`decision.md`): the reading load of the two
  index pages, one keyboard action on two phone cards, and one layout shift each on two pages
  measured under load, which two probe runs afterwards measured as 0.
