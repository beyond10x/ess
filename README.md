# Executable System Specification

ESS is a standalone Rust workspace for describing systems as typed data, compiling those
descriptions into deterministic IR, projecting concrete artifacts, and checking implementations or
infrastructure against what was declared.

The canonical command is `ess`:

```console
cargo run --bin ess -- validate --path examples/billing
cargo run --bin ess -- compile --path examples/billing --format json
cargo run --bin ess -- generate --path examples/billing --kind openapi --out generated
cargo run --bin ess -- conform synthesize --path examples/billing --out suite.json
cargo run --bin ess -- conform run --suite suite.json --target billing --report-out report.json
cargo run --bin ess -- import openapi --path api.yaml --out interface.json
cargo run --bin ess -- project openapi --ir interface.json --out normalized-api.yaml
cargo run --bin ess -- schema validate instances --schemas schemas
cargo run --bin ess -- schema typescript urn:example:registry:1 --root Registry --schemas schemas
```

Adapters use one explicit contract:

- `ess import <adapter>` reads a concrete source and reports coverage, diagnostics, and unresolved
  references while producing typed IR where the adapter supports the source.
- `ess project <adapter>` writes reviewable artifacts, obligations, and refusals. It never applies
  infrastructure or mutates an external system.
- Kubernetes import accepts a sanitized bundle or performs a live scan at the credential edge.
  Secret values are digested before any serialization or filesystem write.
- OpenAPI import produces `ess-service-interface/1`: typed service, operation, JSON-message, local
  reference, and interface-type structures plus coverage gaps and unresolved references. Projection
  accepts that IR or a native ESS specification. The supported subset has an IR → OpenAPI → IR
  semantic round-trip guarantee; unsupported protocol features are normalized, reported, or refused.

`EssIr`, `InfraIr`, and the adapter-specific service-interface representation intentionally remain
distinct. The compiler IRs keep compiler-minted handles, total lookups, ordered collections, and
deterministic serialization. New typed constructs are added only when an importer or projector
establishes their semantics; there is no generic property bag or facet registry.

Run the complete offline gate with:

```console
task check
```

The historical ESS, infrastructure, schema, examples, suites, and generated artifacts were
extracted with filtered Git history. The Kubernetes adapter additionally carries the imported
history of the former standalone scanner.
