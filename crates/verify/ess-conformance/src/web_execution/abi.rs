//! Closed bounded linear-memory protocol. Payload values never cross JavaScript JSON parsing.
use super::{bundle::Blob, Error, Installation, Product, Result, MAX_FRAME};
const MAGIC: &[u8; 4] = b"ESBW";
/// A worker-local request/response owner; exported buffers cannot outlive the next reserve.
pub struct Bridge<I> {
    product: Product<I>,
    request: Vec<u8>,
    response: Vec<u8>,
    reserved: bool,
    last_request: u32,
}
impl<I: Installation> Default for Bridge<I> {
    fn default() -> Self {
        Self::new()
    }
}
impl<I: Installation> Bridge<I> {
    /// No installation callback occurs at construction.
    pub fn new() -> Self {
        Self {
            product: Product::new(),
            request: Vec::new(),
            response: Vec::new(),
            reserved: false,
            last_request: 0,
        }
    }
    /// Allocate a checked module-owned input span. Invalidates every prior buffer span.
    pub fn reserve(&mut self, len: u32) -> Result<&mut [u8]> {
        self.reserved = false;
        self.response.clear();
        self.request.clear();
        let len = len as usize;
        if !(24..=MAX_FRAME).contains(&len) {
            self.product.loaded = None;
            return Err(Error::ResourceLimit);
        }
        if self.request.try_reserve(len).is_err() {
            self.product.loaded = None;
            return Err(Error::ResourceLimit);
        }
        self.request.resize(len, 0);
        self.reserved = true;
        Ok(&mut self.request)
    }
    /// Consume exactly the reserved request. No caller-provided raw pointer is read.
    #[allow(clippy::too_many_lines)]
    pub fn dispatch(&mut self, len: u32) -> &[u8] {
        let mut request_id = 0;
        let reserved = std::mem::replace(&mut self.reserved, false);
        let result = if !reserved || len as usize != self.request.len() {
            Err(Error::InvalidFrame)
        } else {
            let mut read = Read::new(&self.request);
            (|| {
                if read.take(4)? != MAGIC || read.word()? != 1 || read.word()? != 0 {
                    return Err(Error::IncompatibleAbi);
                }
                let opcode = read.word()?;
                request_id = read.word()?;
                let length = read.word()? as usize;
                if length != read.remaining() || request_id == 0 || request_id <= self.last_request
                {
                    return Err(Error::InvalidFrame);
                }
                self.last_request = request_id;
                let mut out = Write::new();
                let tag = match opcode {
                    1 => {
                        // Failed Load, including framing refusal, cannot retain earlier authority.
                        self.product.loaded = None;
                        let manifest = read.text(256 * 1024)?;
                        let count = read.word()? as usize;
                        if count > 1026 {
                            return Err(Error::ResourceLimit);
                        }
                        let mut blobs = Vec::new();
                        for _ in 0..count {
                            blobs.push(Blob {
                                path: read.text(1024)?.into(),
                                bytes: read.bytes(MAX_FRAME)?.to_vec(),
                            });
                        }
                        read.end()?;
                        let handle = self.product.load(manifest, blobs)?;
                        let loaded = self.product.loaded(handle)?;
                        out.word(handle)?;
                        out.text(loaded.selected().digest())?;
                        out.text(loaded.presentation())?;
                        1
                    }
                    2 => {
                        let handle = read.word()?;
                        let count = read.word()? as usize;
                        if count > read.remaining() / 4 {
                            return Err(Error::InvalidFrame);
                        }
                        let mut ids = Vec::new();
                        for _ in 0..count {
                            ids.push(
                                crate::ScenarioId::parse(read.text(4096)?)
                                    .map_err(|_| Error::InvalidFrame)?,
                            );
                        }
                        read.end()?;
                        let handle = self.product.select(handle, &ids)?;
                        let loaded = self.product.loaded(handle)?;
                        out.word(handle)?;
                        out.text(loaded.selected().digest())?;
                        out.text(loaded.presentation())?;
                        2
                    }
                    3 => {
                        let handle = read.word()?;
                        let nonce: [u8; 16] =
                            read.take(16)?.try_into().map_err(|_| Error::InvalidFrame)?;
                        let generation = read.word()?;
                        read.end()?;
                        let completed = self.product.run(handle, nonce, generation)?;
                        out.text(&completed.digest)?;
                        out.raw(&completed.nonce)?;
                        out.word(completed.generation)?;
                        out.text(&completed.report)?;
                        out.text(&completed.run)?;
                        out.text(&completed.display)?;
                        3
                    }
                    4 => {
                        let handle = read.word()?;
                        read.end()?;
                        self.product.release(handle)?;
                        4
                    }
                    _ => return Err(Error::InvalidFrame),
                };
                Ok((tag, out.0))
            })()
        };
        let (tag, status, payload) = match result {
            Ok((tag, payload)) => (tag, 0, payload),
            Err(error) => {
                self.product.loaded = None;
                let mut out = Write::new();
                // Static bounded error category only; no observed value or exception crosses ABI.
                let _ = out.text(error.code());
                let _ = out.text("$browser");
                (5, error as u32, out.0)
            }
        };
        self.response = response(request_id, tag, status, &payload);
        &self.response
    }
    /// Current response, invalidated by reserve/dispatch.
    pub fn response(&self) -> &[u8] {
        &self.response
    }
}
fn response(id: u32, tag: u32, status: u32, payload: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(28 + payload.len());
    result.extend_from_slice(MAGIC);
    for word in [
        1,
        0,
        tag,
        id,
        status,
        u32::try_from(payload.len()).unwrap_or(0),
    ] {
        result.extend_from_slice(&word.to_le_bytes());
    }
    result.extend_from_slice(payload);
    result
}
struct Read<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Read<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }
    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(len).ok_or(Error::InvalidFrame)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(Error::InvalidFrame)?;
        self.offset = end;
        Ok(value)
    }
    fn word(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| Error::InvalidFrame)?,
        ))
    }
    fn bytes(&mut self, max: usize) -> Result<&'a [u8]> {
        let length = self.word()? as usize;
        if length > max {
            return Err(Error::ResourceLimit);
        }
        self.take(length)
    }
    fn text(&mut self, max: usize) -> Result<&'a str> {
        std::str::from_utf8(self.bytes(max)?).map_err(|_| Error::InvalidFrame)
    }
    fn end(&self) -> Result<()> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            Err(Error::InvalidFrame)
        }
    }
}
struct Write(Vec<u8>);
impl Write {
    const fn new() -> Self {
        Self(Vec::new())
    }
    fn raw(&mut self, bytes: &[u8]) -> Result<()> {
        if self
            .0
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > MAX_FRAME - 28)
        {
            return Err(Error::ResourceLimit);
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    fn word(&mut self, value: u32) -> Result<()> {
        self.raw(&value.to_le_bytes())
    }
    fn text(&mut self, value: &str) -> Result<()> {
        self.word(u32::try_from(value.len()).map_err(|_| Error::ResourceLimit)?)?;
        self.raw(value.as_bytes())
    }
}
/// Build a bounded Load request in Rust, useful to native callers and parity tests.
pub fn load_request(id: u32, manifest: &str, blobs: &[Blob]) -> Result<Vec<u8>> {
    let mut payload = Write::new();
    payload.text(manifest)?;
    payload.word(u32::try_from(blobs.len()).map_err(|_| Error::ResourceLimit)?)?;
    for blob in blobs {
        payload.text(&blob.path)?;
        payload.word(u32::try_from(blob.bytes.len()).map_err(|_| Error::ResourceLimit)?)?;
        payload.raw(&blob.bytes)?;
    }
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    for word in [
        1,
        0,
        1,
        id,
        u32::try_from(payload.0.len()).map_err(|_| Error::ResourceLimit)?,
    ] {
        out.extend_from_slice(&word.to_le_bytes());
    }
    out.extend_from_slice(&payload.0);
    Ok(out)
}
