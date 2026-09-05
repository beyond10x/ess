# Schema-only Component Import

## Boundary

A component collection is not necessarily a valid service contract. This adapter
selects named schemas from `/components/schemas` without changing the existing
OpenAPI service importer. Its caller must explicitly select JSON Schema 2020-12;
the source envelope's declared dialect is retained, never promoted to a claim
that the source was a valid OpenAPI service.

The structural schemas stay JSON Schema, the domain representation already owned
by `schema-contract`. They do not become lifecycle entities or arbitrary metadata
on `EssIr`. No network, source directory discovery or application code runs here.

## Persisted Contract

`ess-schema-bundle/1` is a new, strict import-result envelope. It contains the
original UTF-8 JSON source, SHA-256 of those exact bytes, the source's declared
envelope dialect, the explicitly selected structural dialect, sorted selected
component roots and the transformed reference closure as named JSON Schemas.
Unknown envelope fields and unknown format versions are refused. This adds a
new input format; no existing persisted ESS format or reader changes meaning.

The public value is sealed. Reload parses a private wire representation,
reimports its retained source with its recorded root/dialect selection, and
compares the complete result. Altered schemas, source digests, qualifications or
root closures cannot be admitted merely because the JSON parses. Changing the
recorded source and selection together is a different declared input, not proof
that an outside party signed it. This is integrity and repeatability, not trust
in an author or a cryptographic signature.

## Structural Interpretation

JSON Schema schema positions are traversed by their defined keywords. Annotation
values such as `default` and `examples` are not traversed as schemas. Constraints
and annotations are preserved exactly; only local component references change
from `#/components/schemas/...` to `#/$defs/...` in projected schemas. URI fragments
and JSON Pointers determine reference identity, including escaped names.

Root selection includes the complete transitive closure in deterministic order.
References outside the component collection, unresolved references, unknown
keywords and nested resource/anchor declarations are explicit refusals. A source
whose embedded `$schema` contradicts the selected dialect is refused. Unsupported
constructs never become a successful partial import. Validation uses the existing
offline JSON Schema engine and its 2020-12 meta-schema, not a new validator.

Boolean schemas, nullable type arrays, scalar enums/constants, compositional schemas,
tuples, object openness and the standard assertion/annotation vocabulary remain
JSON Schema semantics. A default annotation does not become decoder behavior;
unknown application aliases or flattening rules are not fabricated.

## Projection and Validation

`schema import-bundle` writes the qualified envelope only after all selected roots
validate. `schema project-bundle` reloads and checks it, requires one selected root
and an explicit absolute schema id, then emits a standalone JSON Schema with its
reference closure. A concrete `x-ess-source` annotation carries source digest,
declared and selected dialect, root and the explicit-interpretation qualification.
It is provenance, not a general extension/facet registry.

Instances are validated against an explicitly selected root, not a fabricated
union of selected roots and not a new `schema` property inserted into user data.
Target-language declarations remain a separate projection; successful import
does not claim language code, runtime decoders or consumer adoption already exist.

## Evidence

Synthetic fixtures cover composition, nullability, tuple limits, true/false schemas,
open/closed objects, annotations containing literal `$ref` text, source-pointer
escaping, cycles, malformed schemas and persisted tampering. Compare source-schema
validation under the explicitly selected dialect with projected validation across
positive and negative instances. Old or unknown envelope versions refuse. CLI
tests prove refusal precedes output changes. No private adopter fixture enters
this public repository.
