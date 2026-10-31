# Wave: downstream gaps D325

Integration branch `integration/ess-d325`, cut from `origin/main` `f692aabbb` (0.77.0). One pull
request to `main` from a `queue/` branch, merged on a green pipeline, then one release (0.78.0).
Skill: `aep:implementing` 0.25.1, wave mode, non-interactive; the record is
`approval-record:ess-d325-adopter-gaps`.

## Triage on `main` (0.77.0), 2026-10-31

Each reported reproduction was re-run with the 0.77.0 release binary, which is `main`. None is fixed.

| gap | reporter | result on 0.77.0 | story |
|---|---|---|---|
| G1 fixed structured input | a downstream deployment | same output: `{}` sent for an open struct; `example:` refused on struct, `Json` and field | `story:a-structured-input-may-carry-a-fixed-example` |
| G2 external fixture | a downstream deployment | same output: `ESS-CF-TARGET` on the interpreted target, `ESS-SYNTH-004` in synthesis | `story:a-fixture-may-be-a-row-the-deployment-already-holds` |
| G3 transition and state summary | the harness specification | same refusals for 5 of 8 variants | `story:a-transition-and-a-state-may-carry-a-summary` |
| G4 partial nested response literal | the harness specification | same `ESS-AUTHOR-001 … invalid_input` | `story:an-authored-claim-may-compare-a-nested-field` (existing draft) |
| G5 five identifier-guard gaps | a downstream gateway specification | reported on 0.77.0; not re-run (no standalone reproduction) | `story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type`, next wave |
| G6 reached `Binary64` at validate | the Loom specification | still `ESS-AUTHOR-036` | `story:a-binary64-value-round-trips-through-a-conformance-suite` (existing draft), next wave |

## Units

At most four. Order: G1 and G2 (consumer-blocked), then G3 and G4. G2 needed a design decision
(`decision-blocker:provided-entity-rows-for-conformance`); the operator's proxy decided option B
(`architecture-decision-record:a-deployment-may-provide-an-entitys-rows`), and under the
orchestrator's slot rule (D327, consumer-blocked work first) the construct took G4's slot. G4
(`story:an-authored-claim-may-compare-a-nested-field`, fit review written) moves to the next wave.
So the wave has three units.

| unit | story | managed tree | branch | scratch |
|---|---|---|---|---|
| U1 structured-example | G1 | `ess-d325-u1` | `unit/ess-d325-u1` | `ess-d325-wave/u1` under the user cache |
| U2 provided-rows | G2 (option B and the `fixture_inputs` defect) | `ess-d325-u2` | `unit/ess-d325-u2` | `ess-d325-wave/u2` |
| U3 summaries | G3 | `ess-d325-u3` | `unit/ess-d325-u3` | `ess-d325-wave/u3` |

### Selection

`aep plan artifact waves --kind story --status active` (aep 0.78.0) puts the three stories in three
separate waves, because each scope names `crates/specify/ess-domain/src/system.rs`, `CHANGELOG.md`
and, for U2 and U3, `crates/specify/ess-domain/src/entity.rs` and the generated schema. The verb is
right that they collide. They run together anyway under the split-by-symbol fallback, because all
three open the one source format `ess/33`:

- `system.rs`: each unit adds the same `V33` constant, the same `33` in `SUPPORTED_FORMATS` and the
  same `ess/33` variant in `models/toolchain/domains/specify.yaml`, with text fixed in the briefs, so
  the identical hunks merge cleanly; each adds its own sentences to the `ess/33` `FORMAT_HISTORY`
  entry, which the coordinator unions at merge, then reruns `cargo xtask format-history`.
- `entity.rs`: U2 owns `RawEntitySpec` and the entity marker; U3 owns `Transition`, `SpelledState`,
  `RawState` and `StateMachine.naming`.
- `CHANGELOG.md` and `schemas/generated/ess.schema.json`: the coordinator unions the lines and
  regenerates the schema after each merge.
- Suite pair `ess-conformance/60`/`61` is U2's alone; `ess-diff/22` is U3's alone.

Each managed tree builds into its own `target/`. Builds run one at a time through the host's shared
build slot, with line tables only and no incremental compilation.

## Commits this wave makes

One or more per unit on its unit branch, the merges into `integration/ess-d325`, the store commits,
the release commit, the `queue/ess-0.78.0` pull request and its merge, the tag.

## Unit state

| unit | stage | head |
|---|---|---|
| U1 | dispatched | — |
| U2 | dispatched | — |
| U3 | dispatched | — |
