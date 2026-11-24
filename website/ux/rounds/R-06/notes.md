- The gate lane gives the landing page and the visualise page to the slow lane (the coverage
  decision in `./decision.md`) and prints the host load beside its wall time. Every gate run
  of this round started under the host's load line and ended well inside its budget with no new
  failure. The runs table above holds the wall times and no load; each run's loads are in
  `round.json` (`timing.load` of each gate run) and in the round values of `UXF-025`.
- `UXF-025` is closed because the host's load rule judges a test budget at a load under the line
  and the gate runs of the round, all under that line, held it. The runs do not show that the change made the
  gate faster: runs under the line held the budget before the change too, and the probe
  (`probe.json`) ran above the line on an earlier engine and is reported, not judged. The probe
  of the previous round's diagnosis did not refute it: the lane without the known-failing landing
  and visualise tests was faster in most pairs, and in the pair where it was slower it started at
  a much higher load than its partner.
- On a phone, `first-conformance-run/mobile-390`, `limitations/mobile-390` and
  `install-linux-x86_64/mobile-390` reach their page through a link in the page instead of the
  menu (`UXF-013`), and each lost an action; the tests of the pair the record names pass and
  left its assets, and `install-linux-x86_64/mobile-390` was within its limit before. Both phone keyboard runs
  (`first-conformance-run/mobile-390/keyboard`, `install-linux-x86_64/mobile-390/keyboard`) fell by
  about the same key presses (`UXF-015`); the desktop keyboard runs did not move. The reading
  load of the docs introduction's screen fell, and so did the phone scenarios' reading load in
  the measured round; the merge of `main` in the same round can move a reading load without a
  change of the round, so the cause of the desktop fall is not isolated. A value of
  `limitations/mobile-390` that rose on an earlier build of the change, before the status page's
  link started its line, is not in the record.
- The ratchet took the round without a decision: improved values were taken in, no value got worse
  beyond its tolerance, and a few got worse inside it, named by the ratchet's notes:
  `diagnostic-type-009/desktop-1280` reading_load.max, `target-process/desktop-1280`
  reading_load.median and `target-process/mobile-390` reading_load.median. They equal the values
  of the previous round; the baseline held lower ones from earlier rounds. The counts and the
  recomputation are in `./review-report.md`.
- The newest release is written by `cargo xtask docs --write` from the changelog, and a test of the built
  site refuses a hand-typed copy (`UXF-016` follows it). The merge of `main` had left the status page
  behind the install page again.
- The shared docs theme at its release with the graph fix removed every serious axe node of the
  landing page (`UXF-005`). `UXF-023` waited for a committed round to name the landing page's
  contrast and was closed as not reproduced; the report review found that its asset never failed
  for contrast, so the record is open again (see the record). The accessibility gate still fails
  on the visualise page's frame, owned by the templates package.
- The slow lane's layout test of the ess-ui reference page ran to its limit at a load under the line
  and passed in a rerun: `UXF-029`, new.
- Tool defects for the next round: the screen-only user clicks the centre of a link's box, so a
  link wrapped over lines was missed until the status page's link started its line
  (`UXF-033`); and the test of the reporter's lists prints its canned reporter's lines into every
  gate log (`UXF-034`).

## Corrected after the review of this round

The heuristic review and the report review of this round are filed beside these notes
(`./review-heuristics.md`, `./review-report.md`). The report review called the round not fit to
send as it stood. Each correction below is against a record the review recomputed from, and none
needed a new measurement.

- The claim that the runs table holds the loads: it holds none. Source: `round.json`, `runs[].timing.load`;
  the loads are in `UXF-025`.
- The report named only some of the phone scenarios that lost an action, and not the phone keyboard runs that fell. Source:
  `round.json` against `../R-05/round.json`.
- The ratchet's summary and its worse-inside-tolerance values, named above. Source:
  `round.json` against the baseline as committed before the round (`git show` of its parent).
- `UXF-025` closed on the gate runs of the round: restated as above, in the record. Source: the runs of
  `../R-04/round.json` and `../R-05/round.json` that passed under the line.
- `UXF-023` closed as not reproduced: reopened, because the record format gives a closed record
  of high severity only with an asset that failed before. Source: `UXF-023` itself, which says the landing
  page never held a contrast node in a committed round.
- The review's proposals and tool defects have records: `UXF-030` to `UXF-035`. The baseline's
  gate lane still lists the landing and visualise scenarios (`UXF-032`, read from the source, not
  run).
