//! Bounded specification-document sampling through actual ESS production APIs.
pub mod carrier;
pub mod observation;
pub mod pipeline;
pub mod regressions;
pub mod replay;
pub mod structured;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
pub fn digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub mod live;
