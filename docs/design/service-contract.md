# Service contract extraction

This implements the semantic ownership boundary in
[ESS evolution](ess-evolution/dependency-policy.md). `ess-service-contract` selects one component
from a validated `EssIr` and exposes its complete accepted commands, published/emitted events,
declared errors, owned entities and owned-domain views. A view retains its parameters, filter,
ordering and consistency; a command retains each outcome, condition, subject, payload and field
assignment. Zero-event outcomes stay present. External conditions remain explicit binding work.

`ServiceIr` borrows the original compiler model. It neither serializes a second semantic tree nor
mints cross-model handles. The component's declared domain ownership selects entities and views;
referenced entities and types remain resolvable through the same original model, without claiming
ownership of their state. This selection describes a contract, not an authorization boundary.
Compilation, persistence, transport and execution are separate operations.

The default crate depends only on compiler/domain types. Its optional `synthesis` feature extracts
the SDK's existing required-capability resolution: exactly one matching capability is required;
an explicit refusal remains a refusal; generated versus obligation dispositions retain their
existing serde representation. Target-specific synthesis capability is not a semantic guarantee
and this module does not mint or authenticate synthesis plans. Callers still bind plan provenance
to the compiler source and apply their runtime-specific validation.

Service SDK retains its public `RequiredDisposition` path through a re-export and maps extraction
errors into existing runtime diagnostic codes and paths. Its operation order, authorization,
external content, provider catalog and `service-runtime-ir/3` reader remain SDK-owned. Exact
runtime fixtures must compare without regeneration; an extraction that changes those bytes is
not compatible. ER decision/replay convergence remains subsequent work.

The extracted disposition reader also closes an existing malformed-input hole: Serde's internally
tagged unit variant ignored extra fields despite `deny_unknown_fields`. An empty struct wire helper
now rejects them while retaining the public unit variant and identical valid serialized bytes.
This changes admission of malformed input, not the runtime format or any valid semantic value.

The initial extraction is grounded in Service SDK commit
`c70c954da43c063143b34601ef8af7a1c511e5aa`, `service-runtime-ir/src/lib.rs` functions
`required_disposition`, `resolved_command_fields` and `resolved_view_fields`. Synthetic tests
check component isolation, complete per-outcome semantics and selected state/view ownership.
The SDK's existing compilation and persisted-input refusal tests verify consumer compatibility.
