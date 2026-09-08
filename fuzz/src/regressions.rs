//! Retained readable originals and independently pinned translation to the ordered carrier.
use crate::carrier::{Bundle, Document};
pub const NAMES: [&str; 3] = ["system-types", "optional-binding", "no-retry"];
pub fn encoded(name: &str) -> &'static [u8] {
    match name {
        "system-types" => include_bytes!("../regressions/carriers/system-types.json"),
        "optional-binding" => include_bytes!("../regressions/carriers/optional-binding.json"),
        "no-retry" => include_bytes!("../regressions/carriers/no-retry.json"),
        _ => panic!("unknown mandatory regression"),
    }
}
pub fn originals(name: &str) -> Bundle {
    let texts: &[(&str, &str)] = match name {
        "system-types" => &[
            (
                "system.yaml",
                include_str!("../regressions/source/system-types/system.yaml"),
            ),
            (
                "core.yaml",
                include_str!("../regressions/source/system-types/core.yaml"),
            ),
        ],
        "optional-binding" => &[
            (
                "system.yaml",
                include_str!("../regressions/source/optional-binding/system.yaml"),
            ),
            (
                "core.yaml",
                include_str!("../regressions/source/optional-binding/core.yaml"),
            ),
            (
                "wiring.yaml",
                include_str!("../regressions/source/optional-binding/wiring.yaml"),
            ),
        ],
        "no-retry" => &[
            (
                "system.yaml",
                include_str!("../regressions/source/no-retry/system.yaml"),
            ),
            (
                "core.yaml",
                include_str!("../regressions/source/no-retry/core.yaml"),
            ),
            (
                "wiring.yaml",
                include_str!("../regressions/source/no-retry/wiring.yaml"),
            ),
        ],
        _ => panic!("unknown mandatory regression"),
    };
    Bundle {
        documents: texts
            .iter()
            .map(|(label, text)| Document {
                label: (*label).into(),
                text: (*text).into(),
            })
            .collect(),
    }
}
pub fn verify() -> crate::Result<()> {
    let manifest: std::collections::BTreeMap<String, String> =
        serde_json::from_str(include_str!("../regressions/original-sha256.json"))?;
    if manifest.len() != 8 {
        return Err("mandatory source inventory changed".into());
    }
    for name in NAMES {
        let original = originals(name);
        for doc in &original.documents {
            if manifest.get(&format!("seeds/{name}/{}", doc.label))
                != Some(&crate::digest(doc.text.as_bytes()))
            {
                return Err(format!("mandatory source changed: {name}/{}", doc.label).into());
            }
        }
        if crate::carrier::decode(encoded(name))? != original || original.encode()? != encoded(name)
        {
            return Err(
                format!("mandatory document boundaries/order/bytes changed: {name}").into(),
            );
        }
    }
    Ok(())
}
