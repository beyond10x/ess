# CLI consumer coverage through authored input

Accepted bounded decision, 2026-09-09. Owner: story:cli-presentation-binding.
Independent review: review-result:cli-pipeline-decision-r1-20260909.
This decision changes the three new, unlanded CLI coverage profiles. It does not
change the accepted consumer-coverage policy, cell algebra, inventory or baseline.

## Outcome and boundary

Connectors needs to validate an authored ESS model and presentation binding,
generate its CLI fixture, and exercise the resulting process contract. The three
profiles will measure those complete pipelines, with separate terminal stages:

1. Source admission and assembly, ESS compilation, binding compilation.
2. The same admitted input, followed by the deterministic Rust artifact map.
3. The same admitted input, followed by direct execution of the source process
   runtime with a recording handler.

Their exact entry lists include RawSpecFile::parse, Specification::assemble,
ess_compiler::compile and ess_cli_contract::compile. Emission adds
ess_cli_project::project; execution adds ess_cli_project::runtime::run.
Specification::assemble performs model validation; compilation resolves its
typed references. Tests must exercise these real implementations.

This corresponds to the source admission performed before the CLI edge's
binding compilation in crates/edge/ess-cli/src/cli_binding.rs. The direct runtime
profile remains a test of source runtime behavior after model admission. It does
not claim that a generated installed binary parses ESS documents on each command,
executes generators, evaluates model commands/views, or supplies application
handlers. Generated-package subprocess tests remain separate workspace evidence.

## Why the original profiles are insufficient

The initial profile boundaries began at compile(&EssIr, &Binding).
The accepted inventory also includes every structural RawSpecFile wire rule.
A missing required replica floor or an extra member of a closed workload object
cannot reach that typed input. Even the wire schema's minimum zero for a replica
floor is refused by workload validation before an EssIr exists.

The independent review retained those nine exact topology IDs unqualified.
Its diagnosis is preserved in the wave's review evidence; no existing successful
test is retroactively described as a parser refusal. The 470-ID witness checkpoint
likewise remains a record of the earlier boundaries and its 4,029 outstanding
pairs. Revised profiles acquire new fingerprints and new qualification receipts.

## Attribution rules

A Supported cell still requires an exact executable observation. For a no-effect
claim, an admitted changed/control pair must assert the actual source or resolved
distinction and compare the selected terminal stage's concrete result. Merely
parsing a document does not qualify binding, emission or runtime behavior.

A Refused cell may name the source reader, assembly validator, ESS compiler or
binding compiler as its actual rejection boundary. The case must:

- Supply the exact malformed or unsupported input and assert the particular
  diagnostic or structured error and the stage that returned it.
- Assert that no later stage was entered and no handler or artifact emission ran.
- Include an admitted control that reaches the selected terminal stage and
  asserts its binding, artifacts or observed invocation as appropriate.

Every registered requirement remains a finite reviewed list of exact model IDs.
Schema snapshots alone, broad families, prefix expansion, incidental nearby
fields and an arbitrary nonzero exit cannot qualify a cell. Raw and Rust IDs
retain separate attribution. For schema unions, numeric bounds, defaults and
normalization, a witness must exercise the particular represented distinction;
otherwise that ID stays outstanding.

## Compatibility and verification

Preserve all 87 existing profiles, their fingerprints and the exact accepted
BaselineUnknown manifest. The three new CLI profiles were never in that
baseline. Changing their claim boundary and entry list deliberately invalidates
their candidate fingerprints; it requires fresh extraction, reviewed case
registration and execution. No persisted coverage format or ESS format changes.
Do not copy prior qualified receipts across the profile change.

Before implementing this decision, obtain an independent review of its boundary
and attribution. Start with the nine previously unqualified topology rules as
a concrete pilot. Extend finite witnesses only where the remaining model
inventory requires them, retaining named gaps. Full task check and site-build
remain required before landing the ESS work; focused tests are intermediate
evidence. Connectors remains on its exact source pin and must regenerate and
validate against the final reviewed ESS commit.
