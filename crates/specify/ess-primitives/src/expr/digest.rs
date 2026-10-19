//! The digest and encoding functions of the expression language (beyond10x/ess#584, option 1):
//! SHA-256 (FIPS 180-4), base64url without padding (RFC 4648 § 5) and the padded standard base64
//! a `Bytes` value travels as (RFC 4648 § 4).
//!
//! Written out rather than taken from a crate so that this leaf crate keeps its dependency set,
//! and pinned by the standards' own known-answer vectors in the tests below.

/// The SHA-256 round constants (FIPS 180-4 § 4.2.2).
const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// The SHA-256 initial hash value (FIPS 180-4 § 5.3.3).
const H0: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// The SHA-256 digest of `message`: thirty-two bytes.
#[allow(
    clippy::many_single_char_names,
    reason = "the working variables keep the names FIPS 180-4 gives them"
)]
pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut state = H0;
    let length_bits = u64::try_from(message.len())
        .unwrap_or(u64::MAX)
        .wrapping_mul(8);
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&length_bits.to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut schedule = [0u32; 64];
        for (word, bytes) in schedule.iter_mut().zip(block.chunks_exact(4)) {
            *word = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        }
        for t in 16..64 {
            let s0 = schedule[t - 15].rotate_right(7)
                ^ schedule[t - 15].rotate_right(18)
                ^ (schedule[t - 15] >> 3);
            let s1 = schedule[t - 2].rotate_right(17)
                ^ schedule[t - 2].rotate_right(19)
                ^ (schedule[t - 2] >> 10);
            schedule[t] = schedule[t - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[t - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for (round, word) in K.iter().zip(schedule) {
            let big_s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(big_s1)
                .wrapping_add(choose)
                .wrapping_add(*round)
                .wrapping_add(word);
            let big_s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let t2 = big_s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (held, added) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *held = held.wrapping_add(added);
        }
    }
    let mut digest = [0u8; 32];
    for (bytes, word) in digest.chunks_exact_mut(4).zip(state) {
        bytes.copy_from_slice(&word.to_be_bytes());
    }
    digest
}

/// The standard base64 alphabet (RFC 4648 § 4).
const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
/// The URL- and filename-safe alphabet (RFC 4648 § 5).
const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn encode(bytes: &[u8], alphabet: &[u8; 64], pad: bool) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let group = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let indices = [
            group[0] >> 2,
            ((group[0] & 0b11) << 4) | (group[1] >> 4),
            ((group[1] & 0b1111) << 2) | (group[2] >> 6),
            group[2] & 0b11_1111,
        ];
        let written = chunk.len() + 1;
        for (position, index) in indices.into_iter().enumerate() {
            if position < written {
                out.push(char::from(alphabet[usize::from(index)]));
            } else if pad {
                out.push('=');
            }
        }
    }
    out
}

/// `bytes` in base64url without padding (RFC 4648 § 5): the encoding RFC 7636 § 4.2 names for a
/// PKCE `S256` code challenge.
pub fn base64url(bytes: &[u8]) -> String {
    encode(bytes, URL_SAFE, false)
}

/// `bytes` in padded standard base64 (RFC 4648 § 4): how a `Bytes` value travels.
pub fn base64_standard(bytes: &[u8]) -> String {
    encode(bytes, STANDARD, true)
}

/// The bytes a padded standard base64 text names, or `None` where the text is no such encoding
/// ([`crate::facts::is_padded_base64`]'s grammar, with every unused bit zero so that one value has
/// one spelling).
pub fn decode_base64_standard(text: &str) -> Option<Vec<u8>> {
    if !crate::facts::is_padded_base64(text) {
        return None;
    }
    let value = |byte: u8| -> Option<u8> {
        STANDARD
            .iter()
            .position(|candidate| *candidate == byte)
            .and_then(|at| u8::try_from(at).ok())
    };
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    for group in text.as_bytes().chunks_exact(4) {
        let padding = group.iter().rev().take_while(|byte| **byte == b'=').count();
        let mut sextets = [0u8; 4];
        for (slot, byte) in sextets.iter_mut().zip(group) {
            *slot = if *byte == b'=' { 0 } else { value(*byte)? };
        }
        let bytes = [
            (sextets[0] << 2) | (sextets[1] >> 4),
            ((sextets[1] & 0b1111) << 4) | (sextets[2] >> 2),
            ((sextets[2] & 0b11) << 6) | sextets[3],
        ];
        // Bits a shorter final group leaves unused must be zero: `QQ==` and `QR==` would otherwise
        // name one byte.
        match padding {
            1 if bytes[2] != 0 => return None,
            2 if bytes[1] != 0 || bytes[2] != 0 => return None,
            _ => {}
        }
        out.extend_from_slice(&bytes[..3 - padding]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{base64_standard, base64url, decode_base64_standard, sha256};

    fn hex(bytes: &[u8]) -> String {
        use std::fmt::Write as _;
        bytes.iter().fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
    }

    #[test]
    fn sha256_matches_the_fips_180_4_known_answers() {
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(&sha256(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            hex(&sha256(&vec![b'a'; 1_000_000])),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn base64_encodings_match_the_rfc_4648_vectors() {
        for (bytes, standard, url) in [
            ("", "", ""),
            ("f", "Zg==", "Zg"),
            ("fo", "Zm8=", "Zm8"),
            ("foo", "Zm9v", "Zm9v"),
            ("foob", "Zm9vYg==", "Zm9vYg"),
            ("fooba", "Zm9vYmE=", "Zm9vYmE"),
            ("foobar", "Zm9vYmFy", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_standard(bytes.as_bytes()), standard);
            assert_eq!(base64url(bytes.as_bytes()), url);
            assert_eq!(
                decode_base64_standard(standard).as_deref(),
                Some(bytes.as_bytes())
            );
        }
        assert_eq!(base64url(&[0xfb, 0xff]), "-_8");
        assert_eq!(decode_base64_standard("QR=="), None);
        assert_eq!(decode_base64_standard("Zg"), None);
    }

    #[test]
    fn the_rfc_7636_appendix_b_challenge_is_reproduced() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            base64url(&sha256(verifier.as_bytes())),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }
}
