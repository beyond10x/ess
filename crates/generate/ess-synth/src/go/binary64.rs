//! How the Go target carries a finite `Binary64`.

/// `true` once this target's codec carries a finite `Binary64` in every carried position
/// (`crate::failure::binary64` reads it). Flipped by the unit that adds the codec, never before
/// the generated code compiles.
pub(crate) const CARRIED: bool = false;
