//! Test-only producer-version projection; every other raw byte is retained.

pub const BASELINE_VERSION: &str = "0.19.0";

pub fn report_at_baseline(source: &str, current: &str) -> Result<String, String> {
    let report: serde_json::Value = serde_json::from_str(source).map_err(|e| e.to_string())?;
    if report["generator_version"].as_str() != Some(current) {
        return Err("report producer version does not match the current generator".to_owned());
    }
    let before = format!(
        "  \"generator_version\": {},\n",
        serde_json::to_string(current).unwrap()
    );
    let after = format!("  \"generator_version\": \"{BASELINE_VERSION}\",\n");
    let offsets: Vec<_> = source
        .match_indices(&before)
        .filter(|(index, _)| *index == 0 || source.as_bytes()[index - 1] == b'\n')
        .map(|(index, _)| index)
        .collect();
    if offsets.len() != 1 {
        return Err("expected exactly one unchanged top-level producer-version line".to_owned());
    }
    let mut projected = source.to_owned();
    projected.replace_range(offsets[0]..offsets[0] + before.len(), &after);
    Ok(projected)
}
