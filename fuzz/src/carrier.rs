//! Closed ordered source bundle. Labels are diagnostic identities, never filesystem paths.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_ENCODED: usize = 65_536;
pub const MAX_DOCUMENTS: usize = 8;
pub const MAX_LABEL: usize = 64;
pub const MAX_TEXT: usize = 16_384;
pub const MAX_TOTAL_TEXT: usize = 32_768;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub label: String,
    pub text: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub documents: Vec<Document>,
}
impl Bundle {
    pub fn validate(&self) -> crate::Result<()> {
        if self.documents.is_empty() || self.documents.len() > MAX_DOCUMENTS {
            return Err("document count outside sampling budget".into());
        }
        let mut labels = BTreeSet::new();
        let mut total = 0usize;
        for doc in &self.documents {
            if doc.label.is_empty() || doc.label.len() > MAX_LABEL {
                return Err("label outside sampling budget".into());
            }
            if !labels.insert(&doc.label) {
                return Err("duplicate diagnostic label".into());
            }
            if doc.text.len() > MAX_TEXT {
                return Err("document text outside sampling budget".into());
            }
            total += doc.text.len();
        }
        if total > MAX_TOTAL_TEXT {
            return Err("total text outside sampling budget".into());
        }
        Ok(())
    }
    pub fn encode(&self) -> crate::Result<Vec<u8>> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        if bytes.len() > MAX_ENCODED {
            return Err("encoded carrier outside sampling budget".into());
        }
        Ok(bytes)
    }
}
pub fn decode(bytes: &[u8]) -> crate::Result<Bundle> {
    if bytes.len() > MAX_ENCODED {
        return Err("encoded carrier outside sampling budget".into());
    }
    // Typed deserialization rejects nested wrong shapes; its recursion limit remains enabled.
    let bundle: Bundle = serde_json::from_slice(bytes)?;
    // The observation retains the pretty encoding, so the ceiling is decided on that form here:
    // a compact carrier that fits while its retained form does not is an input refusal, not a
    // later observation failure.
    bundle.encode()?;
    Ok(bundle)
}
