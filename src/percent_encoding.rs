//! Helpers for percent encodings (a string of 3 bytes which matches regex `%[0-9a-fA-F]{2}`).
//!
//! The aim of this module is to provide helpers for the following exact specification:
//! <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#percent-decode>

use crate::num;

/// Length of a single percent encoding string in bytes.
const BYTES: usize = 3;

/// Convert the value back to a UTF-8 value (defaults to lowercase as in [`char::from_digit`]).
///
/// The output is lowercase, panics and returns `&str` to match the behavior of
/// [`char::from_digit`].
///
/// # Panics
///
/// If the buffer is not at least 3 in length, panics.
pub const fn encode(value: u8, buf: &mut [u8]) -> &str {
    buf[0] = b'%';
    num::u8::encode_strict(value, 16, buf.split_at_mut(1).1);
    // SAFETY: it's just a % and two hex characters, definitely valid.
    unsafe { core::str::from_utf8_unchecked(buf.split_at(3).0) }
}

/// Decode the percent encoded value at the beginning of the text.
///
/// If a parse is successful, the rest of the string is guaranteed to be at index 3.
pub const fn prefix(s: &[u8]) -> Option<u8> {
    if s.len() < BYTES || s[0] != b'%' {
        return None;
    }
    num::u8::strict(s.split_at(1).1.split_at(2).0, 16) // &s[1..3]
}

/// Search for the first substring that matches `%[0-9a-fA-F]{2}` and also decode it.
pub const fn next(s: &[u8]) -> Option<(usize, u8)> {
    let mut i = 0;
    // the last values will not contain anything anyways so no point in checking
    while i < s.len() - BYTES {
        if let Some(v) = prefix(s.split_at(i).1) {
            return Some((i, v));
        }
        i += 1;
    }
    None
}

/// A basic common parser for a percent encoded string (`return (overflown, len)`).
macro_rules! parse {
    ($s:ident, $buf:ident) => {{
        let mut s_i = 0;
        let mut buf_i = 0;

        let mut overflow = false;

        while s_i < $s.len() {
            if buf_i >= $buf.len() {
                overflow = true;
                break;
            }
            match prefix($s.split_at(s_i).1) {
                Some(v) => {
                    $buf[buf_i] = v;
                    s_i += BYTES;
                }
                None => {
                    $buf[buf_i] = $s[s_i];
                    s_i += 1;
                }
            }
            buf_i += 1;
        }

        while buf_i < $buf.len() {
            $buf[buf_i] = 0; // fill the rest with NUL
            buf_i += 1;
        }

        (overflow, buf_i)
    }};
}

/// Decode a [percent encoded] string in itself and return the new length.
///
/// If the returned length is different from the given, the result is necessarily changed.
///
/// Everything after the new length is set to zero (`\0`, `NUL`) so the results are valid C
/// strings.
///
/// [percent encoded]:
/// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#percent-decode
//
// `&mut` in const is added in 1.83 which is not getting added to percent_encoding crate
pub const fn inplace(s: &mut [u8]) -> usize {
    parse!(s, s).1 // .0 is ignored since this never overflows.
}

// TODO pub const fn copy(s: &[u8], buf: &mut [u8]) -> Option<usize> /*overflow: None*/
// TODO pub fn parse(s: impl AsRef<[u8]>) -> Vec<u8>
// TODO pub fn str(s: &str) -> String
// TODO pub fn string_inplace(s: &mut String)
