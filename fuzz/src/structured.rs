//! Original bytes deterministically render bounded actual ESS sources; no source mutation afterward.
use crate::carrier::{Bundle, Document, MAX_ENCODED};
const PRIMITIVES: [&str; 8] = [
    "String",
    "Boolean",
    "Integer",
    "Decimal",
    "Timestamp",
    "Duration",
    "Uuid",
    "Bytes",
];
fn byte(input: &[u8], at: usize) -> u8 {
    input.get(at).copied().unwrap_or(0)
}
fn reference(input: &[u8], at: usize, prior: usize, owner: &str) -> String {
    let selector = byte(input, at);
    let mut leaf = if prior > 0 && selector & 8 != 0 {
        format!("{owner}.T{}", usize::from(selector) % prior)
    } else {
        PRIMITIVES[usize::from(selector) % PRIMITIVES.len()].to_owned()
    };
    for depth in 0..usize::from(byte(input, at + 1) % 5) {
        leaf = match (usize::from(selector) + depth) % 3 {
            0 => format!("Optional<{leaf}>"),
            1 => format!("List<{leaf}>"),
            _ => format!("Map<String, {leaf}>"),
        };
    }
    leaf
}
pub fn render(input: &[u8]) -> crate::Result<Bundle> {
    if input.len() > MAX_ENCODED {
        return Err("structured input outside sampling budget".into());
    }
    let count = 1 + usize::from(byte(input, 0) % 8);
    let system_owned = byte(input, 1) & 1 != 0;
    let owner = if system_owned { "demo" } else { "demo.core" };
    let mut types = String::from("types:\n");
    for i in 0..count {
        let at = 2 + i * 3;
        types.push_str(&format!("  - name: {owner}.T{i}\n"));
        let kind = byte(input, at) % 4;
        let ty = reference(input, at + 1, i, owner);
        match kind {
            0 => types.push_str(&format!("    kind: newtype\n    of: {ty}\n")),
            1 => {
                types.push_str("    kind: struct\n    fields:\n");
                for field in 0..1 + usize::from(byte(input, at + 2) % 8) {
                    types.push_str(&format!("      - {{name: field{field}, type: \"{ty}\"}}\n"));
                }
            }
            2 => {
                types.push_str("    kind: enum\n    variants:\n");
                for variant in 0..1 + usize::from(byte(input, at + 2) % 8) {
                    types.push_str(&format!("      - V{variant}\n"));
                }
            }
            3 => {
                types.push_str("    kind: union\n    tag: kind\n    variants:\n");
                for variant in 0..1 + usize::from(byte(input, at + 2) % 8) {
                    types.push_str(&format!("      v{variant}: \"{ty}\"\n"));
                }
            }
            _ => unreachable!(),
        }
    }
    let holder=format!("  - name: demo.core.Holder\n    kind: struct\n    fields:\n      - {{name: value, type: {owner}.T{}}}\n",count-1);
    let mut system =
        String::from("format: ess/1\nsystem: demo\nversion: v1\ndomains: [demo.core]\n");
    let mut core = String::from("domain: demo.core\n");
    if system_owned {
        system.push_str(&types);
        core.push_str("types:\n");
        core.push_str(&holder);
    } else {
        core.push_str(&types);
        core.push_str(&holder);
    }
    let bundle = Bundle {
        documents: vec![
            Document {
                label: "system.yaml".into(),
                text: system,
            },
            Document {
                label: "core.yaml".into(),
                text: core,
            },
        ],
    };
    bundle.validate()?;
    bundle.encode()?;
    Ok(bundle)
}
/// Sixteen independent vectors: four kind families × two ownership choices × two shapes.
pub fn vectors() -> Vec<Vec<u8>> {
    (0..16u8)
        .map(|i| vec![0, (i / 4) % 2, i % 4, 16 + i, (i / 8) + 1])
        .collect()
}
/// Fixed reviewed rendered-source identities; a renderer edit requires deliberate corpus review.
pub fn verify_vectors() -> crate::Result<()> {
    let expected: Vec<String> =
        serde_json::from_str(include_str!("../regressions/structured-source-sha256.json"))?;
    let actual = vectors()
        .iter()
        .map(|v| {
            render(v)
                .and_then(|b| b.encode())
                .map(|b| crate::digest(&b))
        })
        .collect::<crate::Result<Vec<_>>>()?;
    if actual != expected {
        return Err("structured rendered-source identities changed".into());
    }
    Ok(())
}
