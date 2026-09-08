//! One libFuzzer worker calls the same observed production pipeline as stable replay.
use ess_specification_fuzz::{live::Session, observation::Entry, Result};
use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
};
static SESSION: OnceLock<Mutex<Session>> = OnceLock::new();
fn run(entry: Entry, input: &[u8]) -> Result<()> {
    if SESSION.get().is_none() {
        let root = PathBuf::from(
            std::env::var_os("ESS_FUZZ_OUTPUT")
                .ok_or("ESS_FUZZ_OUTPUT must name fresh task-owned output")?,
        );
        let no_work = match std::env::var("ESS_FUZZ_CONTROL").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("none") => false,
            Ok("no-work") => true,
            _ => return Err("unknown engine mutation control".into()),
        };
        SESSION
            .set(Mutex::new(Session::create(&root, entry, no_work)?))
            .map_err(|_| "overlapping engine initialization")?;
    }
    SESSION
        .get()
        .unwrap()
        .lock()
        .map_err(|_| "engine session poisoned after panic")?
        .callback(input)
}
pub fn callback(entry: Entry, input: &[u8]) {
    if let Err(error) = run(entry, input) {
        eprintln!("observation failure: {error}");
        std::process::exit(74)
    }
}
