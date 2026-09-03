# Generated schemas

**Do not edit these files.** They are generated from the Rust types by `cargo xtask schema`, and
`task projection-check` fails if they differ from what the types produce.

They are the interoperability contract: anything that produces or consumes these documents can
validate them without linking the Rust crates.

| file | Rust type | describes |
| --- | --- | --- |
| [`ess.schema.json`](ess.schema.json) | `RawSpecFile` | one file of an executable system specification |
