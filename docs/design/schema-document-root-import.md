# Retained Schema Document Roots

## Required Semantics

A JSON Schema document has a root schema as well as its definitions. Importing only
the definitions drops the enclosing record and its typed fields. A root must be an
explicit selected definition in the internal closure, not an invented OpenAPI service
or an author-maintained copy of its properties.

Add `schema import-document --path FILE --root NAME --dialect draft-2020-12`, with
optional repeated `--definition NAME` for additional independently selectable roots.
The input is a JSON Schema 2020-12 document with optional `$defs`. Its declared dialect,
when present, must agree; this path does not translate OpenAPI nullable semantics.
The supplied root name must not collide with a definition. Local `#` references resolve
to the retained root and `#/$defs/NAME` references retain their closure. Other resources
and unsupported schema positions refuse. Boolean document roots are valid inputs.

## Persistence And Compatibility

Component imports continue to emit exactly `ess-schema-bundle/1`. Document-root imports
emit `ess-schema-bundle/2`, adding a required `document_root` identity that names the
source document root. Version 1 cannot carry that field; version 2 cannot omit it.
The new reader supports both through source replay and whole-envelope equality.
An old strict reader must reject v2 rather than lose root identity. No service-interface
format changes, no generic property bag and no new domain IR are needed.

Source bytes remain exact. Projection relocates root/definition references into one
`$defs` collection but retains original source locations in diagnostics and realization
findings: the document root is JSON Pointer `""`, not a fictitious component path.
Defaults, formats and decoder semantics retain the existing qualification boundary.

## Verification

Exercise required versus nullable root fields, references through the root and `$defs`,
recursive root objects, boolean roots, source-name collisions, dialect mismatch,
unsupported references, retained source bytes and tampered replay. Check v1 unchanged
bytes and old-reader rejection of v2. Root-selected schemas and all three realizations
must consume the same sealed bundle without hand-patching output.
