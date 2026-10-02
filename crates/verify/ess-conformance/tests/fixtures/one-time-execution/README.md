# Shared live observer controls

`manifest.json` binds nineteen suite/34 documents to the stateful Rust target in
`tests/support_one_time/mod.rs`. Each document executes exactly one scenario.
The manifest fixes its status, all five scenario counters, required diagnostic
code, and the ordered value-free callback trace. The native vector test executes
each target; these are not recordings replayed as implementations.

Instantiate `Service::new(Mode)` separately per case and forward real target
callbacks to that same instance from each runtime. Do not return a precomputed
native transcript. Use the existing deterministic runner clock (100 ms per
clock read) when comparing exact callback traces. The target supplies actual
elapsed-window observations; advancing the runner clock alone is not evidence
that the target's publication window closed.

`ESS-CF-DISCLOSURE` names a plaintext non-disclosure verdict. Malformed origin
values use the existing `ESS-CF-PAYLOAD`; target errors and unsupported/resource
observations retain `ESS-CF-TARGET` with distinct `error` and `unsupported`
statuses. No case may serialize the synthetic captured value into diagnostics,
count reports, traces, or failure evidence. The source-owned suite and this test
target are input authority, never persisted observations.

These controls isolate observer semantics using the closed admitted suite
vocabulary. They are not evidence that the production synthesizer has completed
the full command/view/actor inventory. Generated and authored inventory, actual
interpreted-target execution, coverage/35, multi-field/actor-switch controls and
the full runtime package gates remain separate required evidence. Additive
controls must preserve these pinned bytes and expectations unless independently
reviewed evidence establishes a contract correction.
