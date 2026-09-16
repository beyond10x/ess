# Compact live scenarios

An authored live scenario composes setup recipes and actual observations over
original pinned component models. It does not construct a merged model or run
source snippets. Bounded absence, observed ordering and snapshot stability use
the native occurrence ledger. Unsupported verbs are refused during compilation.

The closed `ess-scenario/3` surface has `given` fixture uses, an ordered `when`
sequence of commands or observation barriers, and `then` observations. An action
may capture a required response field. References name earlier actual captures,
including public recipe outputs. Repeating a recipe under a different alias
allocates different capture slots. Aliases are never backend identifiers.

Native lowering emits existing command steps plus `capture_response` and
`eventually_matching_event`. The latter matches dotted payload paths, including
the actual captured call and participant identities. Those meanings require
ordinary suite/10 or inventory-bearing suite/11. Earlier formats retain their
bytes and meanings and refuse the new vocabulary before target invocation.
Both Rust and generated Go evaluate the same suite. A missing response, unrelated
event, malformed matched payload or interrupted observation cannot pass.

Composition authority stays separate from a service IR: the suite names the
composition and its canonical digest, while each lookup keeps the original
service owner and transitive type declarations. Compilation retains original
authored and recipe bytes and expansion locations. The native `ess-live-inputs/1`
manifest retains those bytes, each original canonical model and their individual
digests. Suite provenance binds the exact manifest bytes through
`live_inputs_digest`; `Session.CheckLiveInputs` checks that reference before
an adopter acquires resources. Archive both files and the native report.
Ambiguous same-named commands or events across selected models are refused until
the executable vocabulary can carry the service identity explicitly.

Fixture operations belong to the component implementing them. A test harness may
declare its own setup port; that does not grant it authority to emit another
service's success events. Subscribe before stimulus and obtain assertions from
the observed service. Cleanup belongs to the execution session, including after
partial setup and cancellation.

Coverage is complete only for the explicit authored input set. This does not
claim generated obligations or whole-service conformance. Native session report/2
binds execution to exact admitted input bytes; source files, generated input,
report and compiler identity travel together as retained run evidence.

## Temporal execution extension

Temporal scenarios use one subscribed observation lifetime per scenario,
started before fixtures or stimulus. Its source set is fixed by the compiled
scenario. The target reports actual occurrences with contiguous ordinals and a
monotonic receipt time, plus an exclusive completeness watermark. Appending a
receipt and reading the watermark must share a synchronization boundary. The
target cannot claim completeness by sampling independent histories whose older
events might still be queued for decoding.

The native `check_live` vocabulary names an actual matching occurrence as an anchor,
requires another occurrence after an anchor, checks strict order, and checks bounded
absence or stability from an anchor. An absence interval excludes its exact end.
Stability checks its baseline and every subsequent scoped occurrence; the fields
being asserted cannot also filter which snapshots count. Event paths resolve
against their original owner's type declarations, and matchers use prior actual
response captures. The [metrics extension](compact-live-metrics.md) adds declared
event captures, integer bounds and quiet baselines in source /4 and suite /12–13.
The original /3 remains unchanged and refuses these additions.

Transport adapters provide facts and wait to extend the observed interval. They
never answer whether a window passed. The native ledger retains the first gap or
counterexample. A disconnected, malformed, overflowing, cancelled, truncated or
reordered lifetime cannot qualify, even if later buffered frames look correct.
Rust and generated Go execute the same persisted checks and counterexample fixtures.
The compact surface supplies `anchor`, `after`, `since`, `absent_for_ms`,
`stable_for_ms` and `required`. This is part of the same unreleased suite/10 and
suite/11 tranche; released formats retain their original bytes and refuse these steps.

## Host selection

The callable Go runtime narrows an admitted coverage input before execution using
the same explicit-ID selection semantics as the Rust selector. It preserves the
original parent bytes, moves omitted candidates to `selection_filter` records,
and re-admits the complete carrier before creating a new session. Selection never
edits an executing session or discards unknown/refused coverage. Repeated CLI
`--scenarios` arguments compile an explicit set of original source files.
