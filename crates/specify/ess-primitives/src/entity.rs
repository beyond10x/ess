//! Versioned semantic kinds and logical locators used by ESS projections.

use std::fmt;
use std::str::FromStr;

use crate::error::ParseError;

const LOCATOR_SCHEME: &str = "ep";

/// A versioned semantic entity type, written `<namespace>.<name>/v<version>`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(into = "String")]
pub struct EntityType {
    namespace: String,
    name: String,
    version: u32,
}

impl EntityType {
    /// Creates a type from its namespace, name and positive version.
    pub fn new(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        version: u32,
    ) -> Result<Self, ParseError> {
        let namespace = namespace.as_ref();
        let name = name.as_ref();
        let rendered = format!("{namespace}.{name}/v{version}");
        let reject =
            |reason: &str| ParseError::identifier("entity type", &rendered, reason.to_owned());
        if version == 0 {
            return Err(reject("type versions start at 1"));
        }
        for (part, label) in [(namespace, "namespace"), (name, "name")] {
            let valid = !part.is_empty()
                && part.split(['.', '-']).all(|segment| {
                    !segment.is_empty()
                        && segment
                            .chars()
                            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
                });
            if !valid {
                return Err(reject(&format!(
                    "the {label} must be lower-case kebab-case, optionally dotted"
                )));
            }
        }
        Ok(Self {
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            version,
        })
    }

    /// Parses `<namespace>.<name>/v<version>`.
    pub fn parse(value: &str) -> Result<Self, ParseError> {
        let reject = |reason: &str| ParseError::identifier("entity type", value, reason.to_owned());
        let (qualified, version) = value
            .rsplit_once('/')
            .ok_or_else(|| reject("expected `<namespace>.<name>/v<version>`"))?;
        let version = version
            .strip_prefix('v')
            .ok_or_else(|| reject("the version is written `v1`"))?
            .parse::<u32>()
            .map_err(|_| reject("the version must be an integer"))?;
        let (namespace, name) = qualified
            .rsplit_once('.')
            .ok_or_else(|| reject("expected a namespace, as in `ess.component/v1`"))?;
        Self::new(namespace, name, version)
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{}/v{}",
            self.namespace, self.name, self.version
        )
    }
}

impl fmt::Debug for EntityType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "EntityType({self})")
    }
}

impl From<EntityType> for String {
    fn from(value: EntityType) -> Self {
        value.to_string()
    }
}

impl<'de> serde::Deserialize<'de> for EntityType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::parse(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for EntityType {
    fn schema_name() -> String {
        "EntityType".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(
            "^[a-z][a-z0-9]*(?:[.-][a-z0-9]+)*\\.[a-z][a-z0-9]*(?:[.-][a-z0-9]+)*/v[1-9][0-9]*$"
                .to_owned(),
        );
        schema.into()
    }
}

/// A logical semantic address, preserving the existing `ep://` serialized form.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(into = "String")]
pub struct EntityLocator {
    organisation: String,
    space: String,
    kind: String,
    key: String,
}

impl EntityLocator {
    /// The pattern published for the serialized locator.
    pub const PATTERN: &'static str =
        "^ep://[A-Za-z0-9._-]+/[A-Za-z0-9._-]+/[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$";

    /// Creates a logical locator.
    pub fn new(
        organisation: impl AsRef<str>,
        space: impl AsRef<str>,
        kind: impl AsRef<str>,
        key: impl AsRef<str>,
    ) -> Result<Self, ParseError> {
        let locator = Self {
            organisation: organisation.as_ref().to_owned(),
            space: space.as_ref().to_owned(),
            kind: kind.as_ref().to_owned(),
            key: key.as_ref().to_owned(),
        };
        let rendered = locator.to_string();
        for (segment, label) in [
            (&locator.organisation, "organisation"),
            (&locator.space, "space"),
            (&locator.kind, "kind"),
            (&locator.key, "key"),
        ] {
            if segment.is_empty()
                || !segment
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
            {
                return Err(ParseError::identifier(
                    "locator",
                    &rendered,
                    format!("the {label} is empty or contains a disallowed character"),
                ));
            }
        }
        Ok(locator)
    }

    /// Parses `ep://<organisation>/<space>/<kind>/<key>`.
    pub fn parse(value: &str) -> Result<Self, ParseError> {
        let reject = |reason: &str| ParseError::identifier("locator", value, reason.to_owned());
        let body = value
            .strip_prefix(&format!("{LOCATOR_SCHEME}://"))
            .ok_or_else(|| reject("must begin with `ep://`"))?;
        let segments: Vec<&str> = body.split('/').collect();
        let [organisation, space, kind, key] = segments.as_slice() else {
            return Err(reject(
                "expected `ep://<organisation>/<space>/<kind>/<key>`, four segments",
            ));
        };
        Self::new(organisation, space, kind, key)
    }

    /// Returns the organization segment.
    pub fn organisation(&self) -> &str {
        &self.organisation
    }

    /// Returns the space segment.
    pub fn space(&self) -> &str {
        &self.space
    }

    /// Returns the semantic kind segment.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Returns the logical key segment.
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl fmt::Display for EntityLocator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{LOCATOR_SCHEME}://{}/{}/{}/{}",
            self.organisation, self.space, self.kind, self.key
        )
    }
}

impl fmt::Debug for EntityLocator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "EntityLocator({self})")
    }
}

impl FromStr for EntityLocator {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl From<EntityLocator> for String {
    fn from(value: EntityLocator) -> Self {
        value.to_string()
    }
}

impl<'de> serde::Deserialize<'de> for EntityLocator {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::parse(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl schemars::JsonSchema for EntityLocator {
    fn schema_name() -> String {
        "EntityLocator".to_owned()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(Self::PATTERN.to_owned());
        schema.into()
    }
}
