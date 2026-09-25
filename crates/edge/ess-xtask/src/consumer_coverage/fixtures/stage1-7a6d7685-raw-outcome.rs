pub mod command {
    use ess_primitives::predicate::Predicate;
    use crate::name::QualifiedName;
    use crate::refs::Refs;

    pub struct OutcomeName;
    pub struct PayloadDeclaration;
    pub struct PayloadTable;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    pub struct RawOutcome {
        pub name: OutcomeName,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub when: Option<Predicate>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub external: Option<String>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        pub wrong_state: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub refuses: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub creates: Option<QualifiedName>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub moves: Option<QualifiedName>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub updates: Option<QualifiedName>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub instance: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub emits: Vec<QualifiedName>,
        #[serde(default, skip_serializing_if = "PayloadDeclaration::is_empty")]
        pub payload: PayloadDeclaration,
        #[serde(default, skip_serializing_if = "PayloadTable::is_empty")]
        pub sets: PayloadTable,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub error: Option<QualifiedName>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub summary: Option<String>,
        #[serde(default, skip_serializing_if = "crate::refs::is_empty")]
        pub refs: Refs,
    }
}

pub mod name {
    pub struct QualifiedName;
}

pub mod refs {
    pub struct Refs;
}
