//! The engine callback boundary is distinct from the shared pipeline's attempt boundary.
use crate::{
    observation::{self, Blob, Entry, Record, Writer},
    pipeline::{self, Control},
    Result,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Callback {
    pub callback: u64,
    pub entry: Entry,
    pub input: Blob,
}
pub struct Session {
    writer: Writer,
    callbacks: File,
    next: u64,
    entry: Entry,
    no_work: bool,
}
impl Session {
    pub fn create(root: &Path, entry: Entry, no_work: bool) -> Result<Self> {
        let writer = Writer::create(root)?;
        let callbacks = File::create(root.join("callbacks.frames"))?;
        Ok(Self {
            writer,
            callbacks,
            next: 1,
            entry,
            no_work,
        })
    }
    pub fn callback(&mut self, input: &[u8]) -> Result<()> {
        let record = Callback {
            callback: self.next,
            entry: self.entry,
            input: self.writer.blob(input)?,
        };
        self.next = self
            .next
            .checked_add(1)
            .ok_or("callback identity overflow")?;
        let bytes = serde_json::to_vec(&record)?;
        if bytes.len() > observation::MAX_RECORD {
            return Err("callback frame exceeds bound".into());
        }
        self.callbacks
            .write_all(&(bytes.len() as u32).to_be_bytes())?;
        self.callbacks.write_all(&bytes)?;
        self.callbacks.flush()?;
        if !self.no_work {
            pipeline::run(&mut self.writer, self.entry, input, Control::None)?;
        }
        Ok(())
    }
}
fn read_callback(reader: &mut impl Read) -> Result<Option<Callback>> {
    let mut h = [0; 4];
    if reader.read(&mut h[..1])? == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut h[1..])?;
    let n = u32::from_be_bytes(h) as usize;
    if n == 0 || n > observation::MAX_RECORD {
        return Err("callback frame outside bound".into());
    }
    let mut bytes = vec![0; n];
    reader.read_exact(&mut bytes)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}
pub fn admit(root: &Path, entry: Entry) -> Result<observation::Summary> {
    let summary = observation::admit(root)?;
    let mut callbacks = File::open(root.join("callbacks.frames"))?;
    let mut attempts = File::open(root.join("observations.frames"))?;
    let mut count = 0;
    while let Some(callback) = read_callback(&mut callbacks)? {
        count += 1;
        if callback.callback != count || callback.entry != entry {
            return Err("callback identity/entry mismatch".into());
        }
        observation::read_blob(root, &callback.input)?;
        let start = loop {
            match observation::read_frame(&mut attempts)? {
                Some(r @ Record::Start { .. }) => break Some(r),
                Some(_) => continue,
                None => break None,
            }
        };
        if start
            != Some(Record::Start {
                attempt: count,
                entry,
                input: callback.input,
            })
        {
            return Err("engine callback lacks its exact shared pipeline attempt".into());
        }
    }
    if count != summary.attempts {
        return Err("shared pipeline attempt lacks engine callback".into());
    }
    observation::qualify_live(&summary, entry)?;
    fs::write(
        root.join("admitted-summary.json"),
        serde_json::to_vec_pretty(&summary)?,
    )?;
    Ok(summary)
}
