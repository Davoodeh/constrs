//! Holds URL parsers.

// TODO add uts46 and domain parser

use crate::{utf8, percent_encoding};

/// Match a [URL codepoint] as in [`is_codepoint`] but only in the ASCII range.
///
/// [URL codepoint]:
/// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#url-code-points
pub const fn is_ascii_codepoint(byte: u8) -> bool {
    matches!(byte,
        b'!' | b'$'
            | b'&'..=b'/' /* &'()*+,-./ */
            | b'0'..=b'9'
            | b':' | b';' | b'=' | b'?' | b'@'
            | b'A'..=b'Z'
            | b'_'
            | b'a'..=b'z'
            | b'~'
    )
}

/// Match a [URL codepoint].
///
/// "The URL code points are ASCII alphanumeric, U+0021 (!), U+0024 ($), U+0026 (&), U+0027 ('),
/// U+0028 LEFT PARENTHESIS, U+0029 RIGHT PARENTHESIS, U+002A (*), U+002B (+), U+002C (,), U+002D
/// (-), U+002E (.), U+002F (/), U+003A (:), U+003B (;), U+003D (=), U+003F (?), U+0040 (@), U+005F
/// (_), U+007E (~), and code points in the range U+00A0 to U+10FFFD, inclusive, excluding
/// surrogates and [noncharacters]."
///
/// [noncharacters]: utf8::is_noncharacter
/// [URL codepoint]:
/// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#url-code-points
pub const fn is_codepoint(c: char) -> bool {
    // since [`char`]: the surrogates are excluded so only noncharacters are left to check
    match c {
        ..='\u{7F}' => is_ascii_codepoint(c as u8),
        '\u{A0}'..='\u{10FFFD}' => !utf8::is_noncharacter(c),
        _ => false,
    }
}


/// Represents a [URL unit].
///
/// [URL unit]:
/// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#url-units
pub enum Unit {
    /// A 3 byte value matching a [`percent_encoding`].
    PercentEncoding(u8),
    /// A value which [`is_codepoint`].
    Codepoint(char),
}

impl Unit {
    /// Parse the start of the string.
    pub const fn prefix(s: &[u8]) -> Option<Self> {
        if let Some(v) = percent_encoding::prefix(s) {
            return Some(Self::PercentEncoding(v));
        }

        if let Ok(c) = utf8::prefix(s)
            && is_codepoint(c)
        {
            return Some(Self::Codepoint(c));
        }

        None
    }

    /// The number of bytes in a `[u8]` representation of the value (1..=4 is guaranteed).
    #[allow(clippy::len_without_is_empty)] // never empty
    pub const fn len(&self) -> usize {
        match self {
            Self::PercentEncoding(_) => 3,
            Self::Codepoint(c) => c.len_utf8(),
        }
    }

    /// Convert the value back to a UTF-8 value (percent encoding is done with uppercase ASCII).
    ///
    /// # Panics
    ///
    /// If a percent encoding, buffer must be 3 or larger or panics
    /// ([`percent_encoding::encode`]). Else see [`char::encode_utf8`].  A four byte buffer is
    /// enough for any condition however the exact size is always equal to [`Self::len`].
    pub const fn encode<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        match self {
            Unit::PercentEncoding(v) => percent_encoding::encode(*v, buf),
            Unit::Codepoint(c) => c.encode_utf8(buf),
        }
    }
}

/// Helpers for hosts.
pub mod host {
    use crate::ip;

    use super::{Unit, is_codepoint as is_url_codepoint};

    /// Validate an opaque host to be parsed with an [opaque host parser].
    ///
    /// A [valid opaque host string] is one or more URL units ([`percent_encoding`] or
    /// [`is_codepoint`]) excluding forbidden host codepoints ([`is_forbidden_codepoint`]) or a
    /// bracketed [`ip::v6`] ([`ip::v6::bracketed`]).
    ///
    /// This all means 0 is an invalid result.
    ///
    /// [valid opaque host string]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#valid-opaque-host-string
    /// [opaque host parser]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#concept-opaque-host-parser
    /// [`percent_encoding`]: super::percent_encoding
    pub const fn valid_opaque_prefix_len(s: &[u8]) -> usize {
        if s.is_empty() {
            return 0;
        }

        match ip::v6::bracketed_prefix(s) {
            Some(Ok((_, len))) => return len,
            Some(Err(_)) => return 0,
            None => {}
        }

        let mut i = 0;
        while i < s.len() {
            match Unit::prefix(s.split_at(i).1) {
                Some(v @ Unit::Codepoint(c)) if is_codepoint(c) => i += v.len(),
                Some(v) => i += v.len(),
                _ => return i,
            }
        }
        i
    }

    /// Check for [forbidden host codepoints] which are all ASCII.
    ///
    /// "A forbidden host code point is U+0000 NULL, U+0009 TAB, U+000A LF, U+000D CR, U+0020 SPACE,
    /// U+0023 (#), U+002F (/), U+003A (:), U+003C (<), U+003E (>), U+003F (?), U+0040 (@), U+005B ([),
    /// U+005C (\), U+005D (]), U+005E (^), or U+007C (|)."
    ///
    /// [forbidden host codepoints]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#forbidden-host-code-point
    pub const fn is_forbidden_ascii_codepoint(byte: u8) -> bool {
        matches!(byte,
            b'\0' | b'\t' | b'\n' | b'\r' | b' ' | b'#' | b'/' | b':'
                 | b'<'..=b'@' /* <>?@ */
                 | b'['..=b'^' /* [\]^ */
                 | b'|'
        )
    }

    /// [`is_forbidden_ascii_codepoint`] taking in a [`char`].
    pub const fn is_forbidden_codepoint(c: char) -> bool {
        c as u32 <= u8::MAX as u32 && is_forbidden_ascii_codepoint(c as u8)
    }

    /// Check if a [URL codepoint] excluding [`is_forbidden_codepoint`]
    ///
    /// [URL codepoint]: is_url_codepoint
    pub const fn is_codepoint(c: char) -> bool {
        is_url_codepoint(c) && !is_forbidden_codepoint(c)
    }
}

/// Helpers for URL paths (e.g. `scheme://somehost/path/and/the/rest/of/it`).
pub mod path {
    use super::Unit;

    /// Get as far as possible into the string and return the length of "path string" at the start.
    ///
    /// A path string URL is either:
    ///
    /// - [absolute]: a [relative] path prefixed with a mandatory "/"
    /// - [relative] (or [opaque]): a nullable collection of [path segment strings] separated by a
    ///   slash
    ///
    /// Each of [path segment strings] is (basically an optional string of [`Unit`]s) either:
    ///
    /// 1. zero or more URL [`Unit`] excluding U+002F (/) and U+003F (?), that together are not a
    ///    single-dot URL path segment or a double-dot URL path segment.
    /// 2. a single-dot URL path U+002E (.) or "%2e" (case-insensitive)
    /// 3. a double-dot URL ".%2e", "%2e.", ".." or "%2e%2e" (case-insensitive)
    ///
    /// So if the aim is not to identify which of the 3 types of [path segment strings] is used,
    /// they are just a possibly empty string of URL [`Unit`]s excluding "/" and "?". All in all, a
    /// path string is a collection of URL [`Unit`]s only excluding "?" since "." is a
    /// [`Unit::Codepoint`], "%2e" ([`percent_encoding`]) and "/" is valid via the join of different
    /// segments.
    ///
    /// On a side note, a [relative] and [opaque] paths are _exactly_ the same, basically, the
    /// "opaque" keyword means "do not invest in parsing it further than the string itself." In
    /// contract, "relative" is "a nullable collection of [path segment strings]" so it can be
    /// parsed into that.  Hence such a function, concerned only with the validity of the input,
    /// treats both the same.
    ///
    /// # Examples
    ///
    /// ## Any Type
    ///
    /// ```rust
    /// # use constrs::url::path::any_prefix_len;
    /// assert_eq!(any_prefix_len(b""),               0);
    /// assert_eq!(any_prefix_len(b"/"),              1);
    /// assert_eq!(any_prefix_len(b"//"),             2);
    /// assert_eq!(any_prefix_len(b"any"),            3);
    /// assert_eq!(any_prefix_len(b"/any"),           4);
    /// assert_eq!(any_prefix_len(b"an?y"),           2);
    /// assert_eq!(any_prefix_len(b"/an?y"),          3);
    /// assert_eq!(any_prefix_len(b"a/n?y"),          3);
    /// assert_eq!(any_prefix_len(b"/a/n?y"),         4);
    /// assert_eq!(any_prefix_len(b"a///n?y"),        5);
    /// assert_eq!(any_prefix_len(b"///a///n?y"),     8);
    /// assert_eq!(any_prefix_len(b"a//n//?y"),       6);
    /// assert_eq!(any_prefix_len(b"///a//n//?y"),    9);
    /// assert_eq!(any_prefix_len(b"///a%//n//?y"),   4);
    /// assert_eq!(any_prefix_len(b"a%2C//n//?y"),    9);
    /// assert_eq!(any_prefix_len(b"///a%2C//n//?y"), 12);
    /// ```
    ///
    /// ## Absolute Type
    ///
    /// ```rust
    /// # use constrs::url::path::absolute_prefix_len;
    /// assert_eq!(absolute_prefix_len(b""),               0);
    /// assert_eq!(absolute_prefix_len(b"/"),              1);
    /// assert_eq!(absolute_prefix_len(b"//"),             2);
    /// assert_eq!(absolute_prefix_len(b"any"),            0);
    /// assert_eq!(absolute_prefix_len(b"/any"),           4);
    /// assert_eq!(absolute_prefix_len(b"an?y"),           0);
    /// assert_eq!(absolute_prefix_len(b"/an?y"),          3);
    /// assert_eq!(absolute_prefix_len(b"a/n?y"),          0);
    /// assert_eq!(absolute_prefix_len(b"/a/n?y"),         4);
    /// assert_eq!(absolute_prefix_len(b"a///n?y"),        0);
    /// assert_eq!(absolute_prefix_len(b"///a///n?y"),     8);
    /// assert_eq!(absolute_prefix_len(b"a//n//?y"),       0);
    /// assert_eq!(absolute_prefix_len(b"///a//n//?y"),    9);
    /// assert_eq!(absolute_prefix_len(b"///a%//n//?y"),   4);
    /// assert_eq!(absolute_prefix_len(b"a%2C//n//?y"),    0);
    /// assert_eq!(absolute_prefix_len(b"///a%2C//n//?y"), 12);
    /// ```
    ///
    /// ## Relative Type
    ///
    /// ```rust
    /// # use constrs::url::path::relative_prefix_len;
    /// assert_eq!(relative_prefix_len(b""),               0);
    /// assert_eq!(relative_prefix_len(b"/"),              0);
    /// assert_eq!(relative_prefix_len(b"//"),             0);
    /// assert_eq!(relative_prefix_len(b"any"),            3);
    /// assert_eq!(relative_prefix_len(b"/any"),           0);
    /// assert_eq!(relative_prefix_len(b"an?y"),           2);
    /// assert_eq!(relative_prefix_len(b"/an?y"),          0);
    /// assert_eq!(relative_prefix_len(b"a/n?y"),          3);
    /// assert_eq!(relative_prefix_len(b"/a/n?y"),         0);
    /// assert_eq!(relative_prefix_len(b"a///n?y"),        5);
    /// assert_eq!(relative_prefix_len(b"///a///n?y"),     0);
    /// assert_eq!(relative_prefix_len(b"a//n//?y"),       6);
    /// assert_eq!(relative_prefix_len(b"///a//n//?y"),    0);
    /// assert_eq!(relative_prefix_len(b"///a%//n//?y"),   0);
    /// assert_eq!(relative_prefix_len(b"a%2C//n//?y"),    9);
    /// assert_eq!(relative_prefix_len(b"///a%2C//n//?y"), 0);
    /// ```
    ///
    /// [absolute]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#path-absolute-url-string
    /// [relative]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#path-relative-url-string
    /// [opaque]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#opaque-path-url-string
    /// [path segment strings]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#url-path-segment-string
    /// [`percent_encoding`]: super::percent_encoding
    pub const fn any_prefix_len(s: &[u8]) -> usize {
        let mut pointer = 0;
        while pointer < s.len() {
            match Unit::prefix(s.split_at(pointer).1) {
                Some(Unit::Codepoint('?')) => break,
                Some(v) => pointer += v.len(),
                _ => break,
            }
        }
        pointer
    }

    /// Same as [`any_prefix_len`] (see for more) but ensures the URL starts with a "/".
    pub const fn absolute_prefix_len(s: &[u8]) -> usize {
        if s.is_empty() || s[0] != b'/' {
            return 0;
        }
        any_prefix_len(s)
    }

    /// Same as [`any_prefix_len`] (see for more) but ensures the URL starts without a "/".
    #[doc(alias("opaque_prefix_len"))]
    pub const fn relative_prefix_len(s: &[u8]) -> usize {
        if s.is_empty() || s[0] == b'/' {
            return 0;
        }
        any_prefix_len(s)
    }
}
