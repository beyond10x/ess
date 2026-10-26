//! Adversary pass 1 of unit F (ESS 0.70.0): the TypeScript realization of a closed struct with
//! `fields: []` (`ess/32`, beyond10x/ess#649).
//!
//! `{}` in TypeScript is the type of every value but `null` and `undefined`: a string, a number and
//! an array are all assignable to it. A struct with a field refuses them structurally; the empty
//! closed struct must too, or the realized type describes no object at all.

#[path = "support/empty_struct.rs"]
mod empty_struct;

fn tsc() -> Option<std::path::PathBuf> {
    let tsc = std::env::var_os("ESS_TYPES_NODE")
        .map(|modules| std::path::PathBuf::from(modules).join(".bin/tsc"))
        .filter(|path| path.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("tsc"));
    std::process::Command::new(&tsc)
        .arg("--version")
        .output()
        .ok()
        .map(|_| tsc)
}

#[test]
fn a_closed_empty_struct_type_admits_no_value_that_is_no_object() {
    let Some(tsc) = tsc() else {
        println!("skipped: no tsc");
        return;
    };
    let declarations = empty_struct::plan().typescript().declarations;
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adv-f-pass1-ts-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("scratch");
    for (label, value) in [("text", "\"x\""), ("number", "42"), ("list", "[]")] {
        let file = root.join(format!("{label}.ts"));
        std::fs::write(
            &file,
            format!("{declarations}\nexport const refused: DemoAckClosed = {value};\n"),
        )
        .expect("source");
        let compiled = std::process::Command::new(&tsc)
            .args(["--strict", "--noEmit", "--target", "es2022"])
            .arg(&file)
            .output()
            .expect("tsc runs");
        assert!(
            !compiled.status.success(),
            "the closed empty struct's type admits the {label} {value}:\n{declarations}"
        );
    }
}
