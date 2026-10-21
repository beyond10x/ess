# One-time disclosure reads by an actor who cannot originate the value

A design request from a downstream adopter's report of 2026-10-20 (entry 8), written 2026-10-20
against `origin/main` at `96bd165ea1` (ess 0.63.0). Status: decided 2026-10-21, option 2, from
suite majors 56 and 57 (beyond10x/ess#614); see [Decision](#decision). The contract it extends is
[one-time response values](one-time-response-values.md). "What 0.63.0 does" below is still what
every suite below `ess-conformance/56` holds.

## The use case

One actor issues a one-time value (an API secret, a pairing code) through a command whose outcome
declares `one_time_response:`. Other actors may not run that command but may read views over the
entity it creates. The question the adopter wants the suite to answer is: does the value issued to
the first actor stay out of everything a second actor reads? Their model has 128 cells of this
kind, all refused.

A minimal model is `crates/verify/ess-conformance/tests/fixtures/one-time-caller/with-view.yaml`:
`Admin` may issue a key and label it, `Operator` may only label it, `Viewer` may do nothing, and
the view `keys.api.Keys` projects the key's row. The cells in question are
`keys.api.IssueKey/disclosure/issued/secret/read/keys.api.Keys/as/actor/keys.api.Operator` and
`.../as/actor/keys.api.Viewer`.

## What 0.63.0 does

- **Inventory.** Each declared actor gets one `read/<view>` cell per declared view, after the
  follow-up cells (`followups`, `crates/verify/ess-conformance/src/synthesize/disclosure.rs`,
  the `read_cells` call; `read_cells` below it).
- **Refusal.** A cell whose actor is not granted the originating command is refused with "the
  read's originating caller has no grant to issue this value" (`read_cells`, the `actor.is_some_and`
  test before the arrangement). The `help:` line says the refusal is by design
  (`RefusalCause::repair`, the `disclosure::INVENTORY` arm in
  `crates/verify/ess-conformance/src/synthesize.rs`). Since 0.63.0 every such refusal is linked to
  the first one of its field and aspect (`link_causes`, commit `fed44de1e0`), so the CLI prints one
  block per cause, but the refusal count still counts each cell.
- **Why: the cell's actor is the originator, not the reader.** `read_cells` assigns the cell's
  actor to the originating command ("The cell's caller binds the actual originating invocation, as
  admission requires; it does not invent a view grant"). An actor with no grant cannot originate,
  so there is nothing for its cell to send. Originating as some other actor instead would make the
  cell a copy of that actor's own cell, for the reason in the next point.
- **The read is sent as no actor.** The cell writes `QueryView` with no `ReadAs` step before it
  (`read_cells`, the `ScenarioStep::QueryView` it pushes twice). A read before any `ReadAs` is sent
  as no actor (`ScenarioStep::ReadAs`, `crates/verify/ess-conformance/src/scenario.rs`). So in a
  `read/<view>` cell, `as/actor/<A>` names who issued the value; the read itself is anonymous. No
  cell today reads as a persona other than the one that originated, or as any persona at all.
- **Read-granted views.** Probed on 2026-10-20 with `with-view.yaml` raised to `format: ess/27`
  and `keys.api.Keys` added to `Admin`'s `may:`: the `Admin` read cell still carries no `ReadAs`
  step and passes on the interpreted target, and the `Operator` and `Viewer` cells are refused as
  before. Whether a served target refuses that read sent as no actor was not checked.

The contract asks for more than this. Step 5 of "Finite conformance contract" in
[one-time response values](one-time-response-values.md) says: "repeat applicable follow-up
witnesses for every declared actor ... Permission equivalence does not imply identity
equivalence. The actor switch uses the actual target caller seam; no fabricated authority." A read
sent as no actor does not switch the actor through any seam.

## Options

| | What a `read/<view>` cell for actor A sends | Cells for an A not granted the origin | Changes what an existing id asserts |
|---|---|---|---|
| 1. Keep | origin as A, read as no actor | refused, by design | no |
| 2. Reader cells | origin as A when A is granted, else as the command's first granted actor (`granted_actors`, as the follow-up and denied cells already arrange their origin); then `ReadAs {actor: A}` and the read | built | yes: every read cell gains `ReadAs` and now reads as A |
| 3. Reader cells for the ungranted only | as 2, but only where A is not granted the origin | built | no, but `as/actor/<A>` then means the originator in one cell and the reader in the next |
| 4. Leave them out | no cell | not inventoried, no refusal | no |
| 5. Leave them out, with a note | no cell; one inventory note per field and view saying the read by a non-originating actor is not checked | not inventoried, a note | no |

Notes on each:

- **2** is the adopter's first request and what step 5 of the contract describes. `ReadAs` is
  carried by suite/34 and suite/35, the majors a one-time suite is already written in
  (`crate::view_grant::ORDINARY` is `crate::one_time_response::ORDINARY`), so no new suite major is
  needed for the step. For a read-granted view that A may not read, the cell would expect
  `ExpectNotGranted` after the read, or be left to the view-grant family, which already writes
  `<view>/grant/read/denied` (`crates/verify/ess-conformance/src/view_grant.rs`). The digests of
  every suite holding a read cell move, which `tests/synthesis_suite_bytes_table.rs` would show and
  the changelog must say.
- **3** moves no existing byte but gives one id segment two meanings.
- **4** is the adopter's second request. It conflicts with the contract text "Unreachable,
  unarrangeable ... cells are explicitly refused or inventoried as unsupported with their reason,
  never silently omitted", unless the contract is amended to say such a cell is not part of the
  inventory. The worked counts in the guide
  (`website/docs/guides/verify/one-time-responses.md`, "A worked count", held by
  `tests/one_time_caller.rs`) would drop.
- **5** is 4 with the gap still visible, as a note rather than a refusal.

## The questions it put (answered in Decision)

1. Does the actor in a `read/<view>` cell name the reader or the originator?
2. Which option for an actor not granted the origin: 2, 3, 4 or 5?
3. If 2: is moving the bytes of every existing read cell acceptable in a minor release, or does it
   wait for the next suite major?

The author's reading, not a decision: option 2 is what the contract already says and is the only
option that checks the adopter's question. Options 4 and 5 make the count smaller and check
nothing.

## Decision

Taken 2026-10-21 for the 0.67.0 batch (beyond10x/ess#614):

1. The actor in a `read/<view>` cell names the **reader**.
2. **Option 2.** The value is issued as the cell's actor where that actor is granted the issuing
   command, else as the command's first granted actor (`granted_actors`); then a `read_as` step
   names the cell's actor and the view is read, before and after a rotation. A model that declares
   no actor keeps its `as/anonymous` cell unchanged: it reads as no actor and gains no step.
3. **The next suite major.** A suite holding a read cell for a named actor is written at
   `ess-conformance/56` (`/57` with coverage), cumulative over every pair below
   (`crates/verify/ess-conformance/src/one_time_reader.rs`). Below `/56` every reader keeps the
   old meaning, so an existing suite keeps its bytes and what they assert; a suite holding no such
   cell keeps its earlier format. Regenerating a suite that holds one moves it to `/56`.

What the cell does where the read cannot be made:

- No declared actor is granted the issuing command: the cell is refused, "no declared actor is
  granted the command that issues this value".
- The view is read-granted and its grant does not name the cell's actor: the read would be refused
  before anything is read, so the cell is refused, "the view's read grant does not name the
  reader". Where a served component answers the view, `<view>/grant/read/denied` checks that
  refusal.
- The rotation the cell appends checks its own outcome by reading views. While the cell's reader is
  in force, a read there of a read-granted view the reader may not read is sent as the view's
  lowest-named reader, as the ordinary suite sends it, and the cell's reader is put back for the
  closing read.

Admission (Rust, and the generated Go and TypeScript runtimes) holds a `/56` read cell to its
meaning: some read of the view after the origin is sent as the cell's actor (the actor of the last
`read_as` before it, or no actor where none stands before it). Below `/56` the old rule stands: the
origin is sent as the cell's actor and the view is read after it.
