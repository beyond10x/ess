//! Checked decoding shared by existing persisted delivery envelopes.
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Stage};
use crate::Identifier;

/// Decode maps directly, before duplicate keys could be discarded by collection.
pub(crate) fn unique_map<'de, D, K, V>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de> + Ord,
    V: Deserialize<'de>,
{
    struct UniqueMap<K, V>(PhantomData<(K, V)>);
    impl<'de, K: Deserialize<'de> + Ord, V: Deserialize<'de>> Visitor<'de> for UniqueMap<K, V> {
        type Value = BTreeMap<K, V>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map with unique keys")
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut result = BTreeMap::new();
            while let Some((key, value)) = map.next_entry()? {
                if result.insert(key, value).is_some() {
                    return Err(M::Error::custom("duplicate map key"));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(UniqueMap(PhantomData))
}

// The wire fields retain their existing names/defaults. Every Serde route, including nested
// decoding, crosses the same validation boundary without a JSON Value intermediate.
macro_rules! checked_deserialize {
    ($name:ident { $( $(#[$attribute:meta])* $visibility:vis $field:ident: $type:ty, )* }) => {
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                #[derive(serde::Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Wire { $( $(#[$attribute])* $field: $type, )* }
                let wire = <Wire as serde::Deserialize>::deserialize(deserializer)?;
                let value = Self { $( $field: wire.$field, )* };
                value.validate().map_err(serde::de::Error::custom)?;
                Ok(value)
            }
        }
    };
}
pub(crate) use checked_deserialize;

pub(crate) fn check(
    diagnostics: &mut Vec<Diagnostic>,
    valid: bool,
    stage: Stage,
    code: DiagnosticCode,
    subject: &Identifier,
    detail: impl Into<String>,
) {
    if !valid {
        diagnostics.push(Diagnostic::new(stage, code, Some(subject.clone()), detail));
    }
}

pub(crate) fn finish(diagnostics: Vec<Diagnostic>) -> Result<(), Diagnostics> {
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(Diagnostics::from(diagnostics))
    }
}
