//! YAML for ESS.
//!
//! The only crate in this workspace that names a YAML library. Every other crate reads and writes
//! YAML through it (`infra-*` through `ess_primitives::yaml`, so that crate stays their only ESS
//! dependency), so the backend is this crate's business alone
//! (`docs/design/yaml-parser-replacement.md`, beyond10x/ess#541).
//!
//! The API is the subset of `serde_yaml` 0.9 that ESS uses, with the same names and meanings:
//! scalar resolution, duplicate-key behaviour per target, alias limits, `<<` as an ordinary key,
//! tags, the tag form of enums, error wording with its ` at line L column C` suffix, and the bytes
//! [`to_string`] writes.
//!
//! The backend today is `serde_norway`, a fork of `serde_yaml` 0.9 that is byte- and
//! behaviour-identical by measurement and does not depend on `unsafe-libyaml`. It is a stopgap:
//! an ESS-owned parser and emitter are planned behind this same API.

#![forbid(unsafe_code)]

pub use serde_norway::{
    from_slice, from_str, from_value, to_string, to_value, Deserializer, Error, Mapping, Number,
    Value,
};
