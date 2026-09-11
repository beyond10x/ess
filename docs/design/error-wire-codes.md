# Declared error wire codes

An error retains its qualified model identity. Optional `naming.wire` declares the code emitted
by generated native HTTP responses. Without an override those responses keep the full qualified
name. Codes may alias: a transport code alone does not identify a semantic refusal.

Errors use the existing typed Naming vocabulary. Display and naming summary remain presentation
metadata; the existing top-level error summary is preserved independently. Empty naming is omitted
from source and compiled serialization, preserving legacy bytes. Explicit naming requires source
`ess/4`. New error naming deltas require coordinated `ess-diff/4`; legacy deltas retain their format.

Rust and Go HTTP emission use the same explicit override and qualified-name fallback. Error
handles, payload types, outcomes and conformance ErrorRef assertions keep their declared identity.
Adapters must use outcome or other declared authority to distinguish errors sharing a code; this
change invents no message discriminator or inverse lookup.
