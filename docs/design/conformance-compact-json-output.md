# Compact JSON for fresh conformance suites

`ess conform synthesize --target ir --compact` writes a newly generated suite as
deterministic compact JSON. The option applies to ordinary and declared-coverage
generation, including component selection and authored inputs. It changes the IR
file and JSON stdout; text summaries and YAML presentation retain their existing
meaning. Other targets refuse the option before resolving source or writing output.
The option is absent from commands that consume or select existing suite files.

Without the option, the existing pretty writer and exact bytes remain unchanged.
Both writers append exactly one newline. The compact writer uses the same typed
Serde representation and ordering, with no indentation or inter-token whitespace;
string contents and escapes, numeric values, array order, model identity, scenario
IDs, steps and coverage inventory remain intact. Coverage uses the same typed
document envelope as its pretty writer, retaining selection, source identities,
refusals and outside obligations. It must not serialize only the execution DTO,
which would lose the inventory. Ordinary output preserves DTO field order;
coverage preserves its existing recursively sorted object order, including the
existing typed-document-to-JSON-value conversion before serialization.

This is an opt-in presentation of a freshly produced artifact, not a new canonical
definition for existing artifacts. `to_canonical_json` remains the pretty contract;
the ordinary DTO gains a separately named `to_compact_json` method. Coverage has a
typed compact document writer alongside its unchanged canonical writer. No suite,
source or report format version changes merely for whitespace. Readers admitting
the corresponding existing format continue to admit the compact bytes.

Admission hashes original UTF-8 bytes. Pretty and compact artifacts therefore have
different exact suite digests while model and contract identity remain equal.
Reports must be regenerated against the chosen artifact. A report bound to pretty
bytes cannot qualify compact bytes. Selection preserves the compact parent bytes
and references their actual digest; existing admitted inputs are never reformatted.

Focused checks first establish the missing CLI option as a real failure, then cover
default byte preservation, deterministic compact file/JSON output, one final newline,
typed-value and inventory equality, report execution and stale pairing refusal,
parent lineage, and refusal for Go output. Released-reader probes use the same
format, separately from current-source tests. Two adopter measurements regenerate
both presentations from the same source and same generator; historical suite sizes
are provenance only. Any current source or target refusal is recorded, not silently
changed to make an adoption claim.
