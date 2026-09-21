# Input eligibility for an external outcome

An input predicate may constrain whether an external cause can act without deciding whether it
does act. A retry-only path, for example, cannot exercise a provider rejection on the ordinary
send path. Source `ess/6` permits `when:` beside `external:` and resolves the combination as
`ExternalWhen { cause, predicate }`. Formats 1–5 retain their refusal of that combination.

The new condition remains `InjectFault`. Synthesis satisfies its predicate and still emits the
external-outcome arrangement step. Its predicate does not participate in the partition of ordinary
input-selected branches: the provider can accept the same eligible input. An unsatisfiable guard
produces a synthesis refusal. The observed command result remains the verdict authority.

Legacy source, IR and emitted suites retain their bytes. The new condition is distinguishable in
the typed IR; the suite uses existing input, arrangement and assertion steps, so no new runner
operation or suite envelope is needed. Go and TypeScript must execute the same generated input and
fail when a configured rejection actually succeeds. Consumer migration remains explicit and does
not authorize weakening the existing response-payload completeness rules in source formats 4+.
