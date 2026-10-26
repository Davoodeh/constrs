//! Helpers for const time processing of characters, ASCII and unicode.
//!
//! Functions with no prefix are for decoding/parsing and functions with `encode_*` are for
//! encoding.
//!
//! For numeric utilities refer [`num`] module.
//!
//! [`num`]: crate::num.

/// An iterator over outputs of [`prefix_at`].
#[derive(Debug, Clone)]
pub struct Chars<'a> {
    /// The data in the parser
    data: &'a [u8],
    /// The pointer pointing to [`Self::data`]
    pointer: usize,
    /// Initially unknown, if true, the string is a valid string and else it's just `[u8]`.
    validity: Option<bool>,
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // it may contain no valid point or if ASCII all the bytes are valid
        (0, Some(self.data.len()))
    }
}

impl<'a> core::iter::FusedIterator for Chars<'a> {}

impl<'a> Chars<'a> {
    /// Get a copy of the reference passed to this parser.
    pub const fn data(&self) -> &'a [u8] {
        self.data
    }

    /// Return true if valid and false if invalid and [`None`] if not exhausted the data.
    pub const fn validity(&self) -> Option<bool> {
        self.validity
    }

    /// Return where in the [`Self::data`] is the cursor.
    pub const fn pointer(&self) -> usize {
        self.pointer
    }

    /// Create a new parser for the given data.
    pub const fn new(s: &'a [u8]) -> Self {
        Self {
            data: s,
            pointer: 0,
            validity: None,
        }
    }

    /// Recreate the parser
    ///
    /// This does not change any results but enables reiterating.
    ///
    /// Since this parser is a [`FusedIterator`], it's more logically aligned to return a new
    /// instance here instead of applying the value on itself.
    ///
    /// [`FusedIterator`]: core::iter::FusedIterator
    pub const fn renew(self) -> Self {
        Self::new(self.data)
    }

    /// Progress the parser and get the next character.
    ///
    /// This returns [`None`] if the iterator is exhausted or the [`Self::data`] contains
    /// invalid characters.
    pub const fn next(&mut self) -> Option<char> {
        // if validity is true, the iteration is over
        if self.validity.is_some() {
            return None;
        }
        match prefix_at(self.data, self.pointer) {
            Ok(c) => {
                self.pointer += c.len_utf8();
                Some(c)
            }
            Err(v) => {
                self.validity = Some(v);
                None
            }
        }
    }

    /// Return the string from the start as far as it is valid.
    pub const fn prefix(mut self) -> &'a str {
        while self.next().is_some() {}
        unsafe { core::str::from_utf8_unchecked(self.data.split_at(self.pointer).0) }
    }
}

/// Given the first byte of the UTF-8 character, what is the [`char::len_utf8`] (`1..=4`).
pub const fn expected_len(first_byte: u8) -> Option<u8> {
    match first_byte {
        0x00..=0x7F => Some(1),
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

/// Get the next character [`Err`] is the end and its true means was valid till the end.
///
/// This ignores anything before the given `at` cursor.
///
/// This is intended to be used in URLs and in const environments. Use [`str::chars`] if the
/// environment allows. If a "lossy" behavior is intended (replace invalid characters with a
/// replacement, usually `'\u{FFFD}'`), use standard functions.
///
/// All in all, there is no need for this to exist if not for [`str::chars`] not being usable in
/// const.
///
/// If the given index is over `s` will return `Err(true)` regardless of the validity.
///
/// This is extensively tested to ensure it works and equates [`str::chars`] following a
/// [`str::from_utf8`] (see the source, the disabled tests).
///
/// The reverse of this operation is [`char::encode_utf8`].
///
/// See
/// <https://encoding.spec.whatwg.org/commit-snapshots/a985b62a9b45c17da3e17a9f0a0b4e30c34c4a8a/#utf-8-decoder>
//
// NOTE on each change, enable test_u32 module
pub const fn prefix_at(s: &[u8], mut at: usize) -> Result<char, bool> {
    // What is the codepoint being collected
    let mut codepoint = 0;
    // The number of bytes inside the codepoint already (0..=3)
    let mut bytes_seen = 0;
    // The number of bytes to see in [`Self::bytes_seen`] for the codepoint (0..=3)
    let mut bytes_needed = 0;
    // The allowed lower bound of the next byte for the codepoint (inclusive)
    let mut byte_lower_boundary = 0x80;
    // The allowed upper bound of the next byte for the codepoint (inclusive)
    let mut byte_upper_boundary = 0xBF;

    while at < s.len() {
        let byte = s[at];
        at += 1;

        if bytes_needed == 0 {
            match byte {
                0..=0x7F => return Ok(byte as char),
                0xC2..=0xDF => {
                    bytes_needed = 1;
                    codepoint = (byte & 0x1F) as u32;
                }
                0xE0..=0xEF => {
                    if byte == 0xE0 {
                        byte_lower_boundary = 0xA0;
                    }
                    if byte == 0xED {
                        byte_upper_boundary = 0x9F;
                    }
                    bytes_needed = 2;
                    codepoint = (byte & 0xF) as u32;
                }
                0xF0..=0xF4 => {
                    if byte == 0xF0 {
                        byte_lower_boundary = 0x90;
                    }
                    if byte == 0xF4 {
                        byte_upper_boundary = 0x8F;
                    }
                    bytes_needed = 3;
                    codepoint = (byte & 0x7) as u32;
                }
                _ => {
                    return Err(false);
                }
            }

            continue;
        }

        // check boundary and reset it
        if byte < byte_lower_boundary || byte > byte_upper_boundary {
            return Err(false);
        }
        byte_lower_boundary = 0x80;
        byte_upper_boundary = 0xBF;

        // https://encoding.spec.whatwg.org/commit-snapshots/a985b62a9b45c17da3e17a9f0a0b4e30c34c4a8a/#utf-8
        codepoint = (codepoint << 6) | ((byte & 0x3F) as u32);

        bytes_seen += 1;
        if bytes_seen != bytes_needed {
            continue;
        }

        // SAFETY: the ranges are already validated above.
        // test_u32_brute returns valid for all the u32 range implying this never fails.
        return Ok(unsafe { char::from_u32_unchecked(codepoint) });
    }

    Err(true) // since validity is None and the value is empty return true
}

/// Alias for [`prefix_at`] starting `at` 0.
pub const fn prefix(s: &[u8]) -> Result<char, bool> {
    prefix_at(s, 0)
}

/// Return the length of valid UTF-8 bytes at the start of the string.
pub const fn prefix_len(s: &[u8]) -> usize {
    let mut len = 0;
    while let Ok(c) = prefix_at(s, len) {
        len += c.len_utf8();
    }
    len
}

/// Parse the string as far as its prefix is valid (see also [`prefix_len`]).
///
/// To parse a full string and fail if invalid (in other words, if `parse_str` -without the
/// prefix- is needed), use [`str::from_utf8`].
pub const fn str_prefix(s: &[u8]) -> &str {
    // SAFETY: the validity is ensured
    unsafe { core::str::from_utf8_unchecked(s.split_at(prefix_len(s)).0) }
}

/// Check if a [`u32`] is a ["noncharacter"] or not.
///
/// See also <https://github.com/open-i18n/rust-unic/issues/290>
/// and
/// <https://infra.spec.whatwg.org/commit-snapshots/3f984adcd24a6d5c53cc26b3e737701808003f3e/#noncharacter>
///
/// ["noncharacter"]: https://www.unicode.org/faq/private_use.html#noncharacters
pub const fn is_noncharacter_u32(c: u32) -> bool {
    // there are two cases (66 variants):
    // 1. must be in FDD0..=FDEF range (32 variants)
    // 2. either must end in FFFE or FFFF (34 variants, only if a valid [`char`]):
    //    if the lower bits are either FFFE or FFFF
    //    the higher bits can be anything in range of 0..=0x10, but since 0x10FFFF is the maximum
    //    and the input is already proven to be [`char`] with a [`char::MAX`] of 0x10FFFF,
    //    there is no need to double check (unless the input is raw [`u32`]).
    (0xFDD0 <= c && c <= 0xFDEF) || (c & 0xFFFF >= 0xFFFE && c <= 0x10FFFF)
}

/// Check if a [`char`] is a ["noncharacter"] or not.
///
/// See also <https://github.com/open-i18n/rust-unic/issues/290>
/// and
/// <https://infra.spec.whatwg.org/commit-snapshots/3f984adcd24a6d5c53cc26b3e737701808003f3e/#noncharacter>
///
/// ["noncharacter"]: https://www.unicode.org/faq/private_use.html#noncharacters
pub const fn is_noncharacter(c: char) -> bool {
    is_noncharacter_u32(c as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check if a character by character parse equals the string.
    fn stream_eq(input: &[u8], output: &str) {
        let mut i = Chars::new(input);
        let mut o = output.chars();
        while let Some(ic) = i.next() {
            assert_eq!(Some(ic), o.next());
        }
        assert_eq!(i.next(), o.next());
        assert_eq!(i.next(), o.next());
        assert_eq!(i.next(), o.next());

        assert_eq!(str_prefix(input), output);
    }

    /// Parse both with character machine and the full text parser.
    fn parsed_eq(input: &[u8], output: &str) {
        stream_eq(input, output);
        assert_eq!(str::from_utf8(input), Ok(output));
    }

    #[test]
    fn test_u32_encode() {
        // anything larger than 0xFFFFFF will most likely fail to decode even
        // There is a more extensive check in test_u32_brute which can be used to check ALL the
        // possible codes and compare it with str::from_utf8
        for i in 0u32..u32::MAX {
            let mut buf = [0u8; 4];
            match char::from_u32(i) {
                Some(c) => {
                    c.encode_utf8(&mut buf);
                    // println!("valid:   0x{:X?}={:X?}", i, buf);
                    assert_eq!(
                        prefix(&buf),
                        Ok(c),
                        "{:?}={:X?} was valid but could not decode",
                        c,
                        buf
                    );
                }
                None => {
                    // even char::from_32_unchecked doesn't allow for an invalid value
                    #[allow(unnecessary_transmutes)]
                    let c = unsafe { std::mem::transmute::<u32, char>(i) };

                    // This produces A LOT of invalid values but sometimes fails, meaning an
                    // invalid char will produce another false code which is valid. For example,
                    // 0x400000 produces the buffer for 0x10000 which is valid.
                    //
                    // Post 3FFFFF may decode into valid bytes again so a double check below
                    // will ensure the test does not assume the `buf` is invalid when it is
                    // not
                    c.encode_utf8(&mut buf);
                    if str::from_utf8(&buf).is_ok() {
                        // println!(
                        //     "cannot produce more invalid values post: 0x{:X?}!={:X?}",
                        //     i, buf
                        // );
                        break;
                    }

                    // println!("invalid: 0x{:X?}={:X?}", i, buf);
                    assert_eq!(
                        prefix(&buf),
                        Err(false),
                        "0x{:X?} (={:X?}?) was invalid but decoded",
                        i,
                        buf
                    );
                }
            }
        }
    }

    /// Test EVERY SINGLE [`u32`] for UTF-8 encoding using the custom parser above.
    //
    // This module takes an extensive amount of time (2 minutes) and it's mostly worthless
    // compared to the computation it takes. hence, it is disabled by default
    #[cfg(false)]
    mod test_u32_brute {
        use super::*;

        fn test_brute(start: u32, end: u32) {
            for a in start..=end {
                let buf = [
                    (a >> 24) as u8,
                    ((a >> 16) & 0xFF) as u8,
                    ((a >> 8) & 0xFF) as u8,
                    (a & 0xFF) as u8,
                ];
                match str::from_utf8(&buf) {
                    Ok(s) => {
                        assert_eq!(s, prefix(&buf), "text was not parsed successfully");
                    }
                    Err(_) => {
                        let parsed = str_prefix(&buf);

                        let mut i = 0;
                        let mut v = vec![];
                        loop {
                            let result = prefix_at(&buf, i);
                            v.push(result);

                            match result {
                                Ok(c) => i += c.len_utf8(),
                                Err(_) => break,
                            }
                        }

                        assert_ne!(
                            buf.len(),
                            parsed.len(),
                            "text was parsed successfully while it must have \
                                             failed: {:X?}={v:?}",
                            buf,
                        );
                    }
                }
            }
        }

        // this split helps run the functions in parallel and greatly speed up the tests.
        //
        // all the u32 range should run in about 2 minute on a good modern CPU
        macro_rules! test_u32_brute {
                ($batch:expr) => {
                    seq_macro::seq!(P in 1..=32768 {
                        #[test]
                        #[allow(non_snake_case)]
                        fn test_u32_brute_~P() {
                            let b: u32 = $batch;
                            let start: u32 = b * (P - 1);
                            let end: u32 = b.saturating_mul(P);
                            test_brute(start, end);
                        }
                    });
                };
            }
        test_u32_brute!(u32::MAX / 32768);
    }

    #[test]
    fn test_empty() {
        parsed_eq(&[], "");
    }

    #[test]
    fn test_wpt_encoding_streams_decode_utf8() {
        // https://github.com/web-platform-tests/wpt/blob/172abd268d46f326b3fe937a5a84b99ce0390907/encoding/streams/decode-utf8.any.js#L8
        parsed_eq(
            &[
                73, 32, 240, 159, 146, 153, 32, 115, 116, 114, 101, 97, 109, 115,
            ],
            "I \u{1F499} streams",
        );
    }

    #[test]
    fn test_wpt_encoding_textdecoder_streaming() {
        // https://github.com/web-platform-tests/wpt/blob/172abd268d46f326b3fe937a5a84b99ce0390907/encoding/textdecoder-streaming.any.js#L6
        parsed_eq(
            &[
                0x00, 0x31, 0x32, 0x33, 0x41, 0x42, 0x43, 0x61, 0x62, 0x63, 0xC2, 0x80, 0xC3, 0xBF,
                0xC4, 0x80, 0xE1, 0x80, 0x80, 0xEF, 0xBF, 0xBD, 0xF0, 0x90, 0x80, 0x80, 0xF4, 0x8F,
                0xBF, 0xBF,
            ],
            "\0123ABCabc\u{80}\u{FF}\u{100}\u{1000}\u{FFFD}\u{10000}\u{10FFFF}",
        );
        const VALUES: &[&[u8]] = &[
            &[0xC1],
            &[0xF5],
            &[0xE0],
            &[0xE0, 0x41],
            &[0xE0, 0x80],
            &[0x80],
            &[0xA0],
            &[0xED],
            &[0xED, 0xA0],
            &[0x80],
            &[0xF0, 0x41],
            &[0xF0, 0x80],
            &[0x80],
            &[0xF4, 0xA0],
            &[0xF0, 0x90, 0x41],
            &[0xF0, 0x9F, 0x92], // missing byte
        ];
        for i in VALUES {
            assert!(prefix(i).is_err(), "invalid value parsed: {:?}", i);
        }

        assert_eq!(prefix(&[0x41]), Ok('A'), "A returned invalid");
        assert_eq!(prefix(&[0x42]), Ok('B'), "B returned invalid");
        assert_eq!(prefix(&[0xF0, 0x9F, 0x92, 0xA9]), Ok('\u{1F4A9}')); // not missing now
    }

    #[test]
    fn test_wpt_encoding_textdecoder_fatal_invalid_code() {
        const VALUES: &[&[u8]] = &[
            &[0xFF],                               // invalid code
            &[0xC0],                               // ends early
            &[0xE0],                               // ends early 2
            &[0xC0, 0x00],                         // invalid trail
            &[0xC0, 0xC0],                         // invalid trail 2
            &[0xE0, 0x00],                         // invalid trail 3
            &[0xE0, 0xC0],                         // invalid trail 4
            &[0xE0, 0x80, 0x00],                   // invalid trail 5
            &[0xE0, 0x80, 0xC0],                   // invalid trail 6
            &[0xFC, 0x80, 0x80, 0x80, 0x80, 0x80], // > 0x10FFFF
            &[0xFE, 0x80, 0x80, 0x80, 0x80, 0x80], // obsolete lead byte
        ];
        for i in VALUES {
            assert!(prefix(i).is_err());
        }
    }

    #[test]
    fn test_wpt_encoding_textdecoder_fatal_overlong_encoding() {
        const VALUES: &[&[u8]] = &[
            &[0xC0, 0x80],                         // overlong U+0000 - 2 bytes
            &[0xE0, 0x80, 0x80],                   // overlong U+0000 - 3 bytes
            &[0xF0, 0x80, 0x80, 0x80],             // overlong U+0000 - 4 bytes
            &[0xF8, 0x80, 0x80, 0x80, 0x80],       // overlong U+0000 - 5 bytes
            &[0xFC, 0x80, 0x80, 0x80, 0x80, 0x80], // overlong U+0000 - 6 bytes
            &[0xC1, 0xBF],                         // overlong U+007F - 2 bytes
            &[0xE0, 0x81, 0xBF],                   // overlong U+007F - 3 bytes
            &[0xF0, 0x80, 0x81, 0xBF],             // overlong U+007F - 4 bytes
            &[0xF8, 0x80, 0x80, 0x81, 0xBF],       // overlong U+007F - 5 bytes
            &[0xFC, 0x80, 0x80, 0x80, 0x81, 0xBF], // overlong U+007F - 6 bytes
            &[0xE0, 0x9F, 0xBF],                   // overlong U+07FF - 3 bytes
            &[0xF0, 0x80, 0x9F, 0xBF],             // overlong U+07FF - 4 bytes
            &[0xF8, 0x80, 0x80, 0x9F, 0xBF],       // overlong U+07FF - 5 bytes
            &[0xFC, 0x80, 0x80, 0x80, 0x9F, 0xBF], // overlong U+07FF - 6 bytes
            &[0xF0, 0x8F, 0xBF, 0xBF],             // overlong U+FFFF - 4 bytes
            &[0xF8, 0x80, 0x8F, 0xBF, 0xBF],       // overlong U+FFFF - 5 bytes
            &[0xFC, 0x80, 0x80, 0x8F, 0xBF, 0xBF], // overlong U+FFFF - 6 bytes
            &[0xF8, 0x84, 0x8F, 0xBF, 0xBF],       // overlong U+10FFFF - 5 bytes
            &[0xFC, 0x80, 0x84, 0x8F, 0xBF, 0xBF], // overlong U+10FFFF - 6 bytes
        ];
        for i in VALUES {
            assert!(prefix(i).is_err(), "invalid value parsed: {:?}", i);
        }
    }

    #[test]
    fn test_wpt_encoding_textdecoder_fatal_surrogates() {
        const VALUES: &[&[u8]] = &[
            &[0xED, 0xA0, 0x80],                   // lead surrogate
            &[0xED, 0xB0, 0x80],                   // trail surrogate
            &[0xED, 0xA0, 0x80, 0xED, 0xB0, 0x80], // surrogate pair
        ];
        for i in VALUES {
            assert!(prefix(i).is_err(), "invalid value parsed: {:?}", i);
        }
    }
}
