- The gate lane gives the landing page and the visualise page to the slow lane (the coverage
  decision in `./decision.md`) and prints the host load beside its wall time. Every gate run
  of this round started under the host's load line and ended well inside its budget with no new
  failure; the runs table above holds their times and loads. `UXF-025` is closed on them.
- The probe of the previous round's diagnosis ran before that change (`probe.json`). It did not
  refute the diagnosis: the lane without the known-failing landing and visualise tests was faster in most pairs,
  and in the pair where it was slower it started at a much higher load than its partner. Every probe run
  started above the load line, so the probe is reported, not judged.
- On a phone, `first-conformance-run/mobile-390` and `limitations/mobile-390` reach their page
  through a link in the page instead of the menu (`UXF-013`); their tests pass and left the record's assets. The reading load of the docs
  introduction's screen fell, and so did both phone scenarios' reading load in the measured round.
  A value of `limitations/mobile-390` that rose on an earlier build of the change, before the status
  page's link started its line, is not in the record. No value got worse beyond its tolerance, so
  the ratchet took the round without a decision.
- The newest release is written by `cargo xtask docs --write` from the changelog, and a test of the built
  site refuses a hand-typed copy (`UXF-016` follows it). The merge of `main` had left the status page
  behind the install page again.
- The shared docs theme at its release with the graph fix removed every serious axe node of the
  landing page (`UXF-005`); `UXF-023`, which waited for a committed round to name the landing
  page's contrast, closes as not reproduced. The accessibility gate still fails on the visualise page's frame,
  owned by the templates package.
- The slow lane's layout test of the ess-ui reference page ran to its limit at a load under the line
  and passed in a rerun: `UXF-029`, new.
- Tool defects for the next round: the screen-only user clicks the centre of a link's box, so a
  link wrapped over lines was missed until the status page's link started its line; and the
  test of the reporter's lists prints its canned reporter's lines into every gate log.
