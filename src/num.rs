//! Holds helpers for number parsing and encoding.
//!
//! Functions with no prefix are for decoding/parsing and functions with `encode_*` are for
//! encoding.
//!
//! Common utilities here work on [`digit`] and number validations (e.g. [`prefix_len`]). There are
//! separate modules for each type (e.g. [`mod@u8`]) which contain a fixed set of utilities for parsing and
//! decoding. See the [`mod@u8`] module for more examples
//!
//! To keep it consistent with the logic of [`char::to_digit`] and [`char::from_digit`], all the
//! parsers and decoders have variable radices that work consistently with radices `2..=36`.
//
// TODO add a trait extension that adds all the functionality here to `char`, `[u8]` and related
// TODO impl encode for types other than u8

const fn panic_radix() -> ! {
    // assertion copied from [`char::to_digit`] source
    panic!("to_digit: invalid radix -- radix must be in the range 2 to 36 inclusive");
}

const fn assert_radix(radix: u8) {
    if radix < 2 || radix > 36 {
        panic_radix()
    }
}

const fn encode_panic_radix() -> ! {
    panic!("from_digit: radix is too high (maximum 36)");
}

const fn encode_assert_radix(radix: u8) {
    // assertion copied from [`char::from_digit`] source
    if radix > 36 {
        encode_panic_radix()
    }
}

/// Create an ASCII character from a number (taken from [`char::from_digit`] without check).
///
/// See also [`digit`] for a radix aware version.
///
/// The given number is assumed to be valid and the radix is assumed to be the maximum (based on
/// [`char::from_digit`]) which is a radix of 36 (largest digit of `z`).
///
/// If the promise is not kept, the value may overflow. While stable and not causing any
/// undefined behavior (it's safe), it is not intended to be used like this.
///
/// # Examples
///
/// ```rust
/// # use constrs::num::encode_digit_unchecked;
/// assert_eq!(encode_digit_unchecked(00), b'0');
/// assert_eq!(encode_digit_unchecked(09), b'9');
/// assert_eq!(encode_digit_unchecked(10), b'a');
/// assert_eq!(encode_digit_unchecked(15), b'f');
/// assert_eq!(encode_digit_unchecked(16), b'g');
/// assert_eq!(encode_digit_unchecked(35), b'z');
///
/// assert_eq!(encode_digit_unchecked(36), b'z' + 1); // invalid values
/// assert_eq!(encode_digit_unchecked(167), 254); // invalid values
/// assert_eq!(encode_digit_unchecked(168), 255); // invalid values
/// ```
///
/// ```rust,should_panic
/// # use constrs::num::encode_digit_unchecked;
/// encode_digit_unchecked(169); // panics with overflow while calculating 255+1 for u8
/// ```
pub const fn encode_digit_unchecked(v: u8) -> u8 {
    if v < 10 {
        return v + b'0';
    }
    (v - 10) + b'a'
}

/// Same as [`char::from_digit`] but gets and puts [`prim@u8`].
///
/// For a radix unaware version use [`encode_digit_unchecked`] if the radix surely does not panic.
///
/// # Panics
///
/// The same as [`char::from_digit`]: "Panics if given a radix larger than 36."
///
/// ```rust
/// # use constrs::num::encode_digit;
/// assert_eq!(encode_digit(0, 0),   None);
/// assert_eq!(encode_digit(0, 1),   Some(b'0'));
/// assert_eq!(encode_digit(1, 1),   None);
/// assert_eq!(encode_digit(0, 36),  Some(b'0'));
/// assert_eq!(encode_digit(35, 36), Some(b'z'));
/// ```
///
/// ```rust,should_panic
/// # use constrs::num::encode_digit;
/// encode_digit(0, 37); // panics
/// ```
pub const fn encode_digit(v: u8, radix: u8) -> Option<u8> {
    encode_assert_radix(radix);
    if v < radix {
        Some(encode_digit_unchecked(v))
    } else {
        None
    }
}

/// Same as [`char::to_digit`] but gets and puts [`prim@u8`].
///
/// # Panics
///
/// The same as [`char::to_digit`]: "Panics if given a radix smaller than 2 or larger than 36."
///
/// ```rust,should_panic
/// # use constrs::num::digit;
/// digit(0, 0); // panics
/// ```
///
/// ```rust,should_panic
/// # use constrs::num::digit;
/// digit(0, 1); // panics
/// ```
///
/// ```rust,should_panic
/// # use constrs::num::digit;
/// digit(0, 37); // panics
/// ```
pub const fn digit(c: u8, radix: u8) -> Option<u8> {
    // source taken from [`char::to_digit`]
    assert_radix(radix);
    let value = if c > b'9' && radix > 10 {
        (c.wrapping_sub(b'A') & !0x20) + 10
    } else {
        c.wrapping_sub(b'0')
    };
    if value < radix {
        Some(value)
    } else {
        None
    }
}

/// [`char::to_digit`] without any of its checks.
///
/// This does not panic for many values, much like [`encode_digit_unchecked`]. Use only when sure
/// the value is valid. If not, use [`digit`].
pub const fn digit_unchecked(c: u8) -> u8 {
    if c > b'9' {
        (c.wrapping_sub(b'A') & !0x20) + 10
    } else {
        c.wrapping_sub(b'0')
    }
}

/// Return the length of the initial sequence of [`digit`]s.
///
/// # Examples
///
/// ```rust
/// # use constrs::num::prefix_len;
/// assert_eq!(prefix_len(b"",     10), 0);
/// assert_eq!(prefix_len(b"0",    10), 1);
/// assert_eq!(prefix_len(b"00",   10), 2);
/// assert_eq!(prefix_len(b"10",   10), 2);
/// assert_eq!(prefix_len(b"10A",  10), 2);
/// assert_eq!(prefix_len(b"010",  10), 3);
/// assert_eq!(prefix_len(b"010A", 10), 3);
/// assert_eq!(prefix_len(b"999999999999999999999999999999", 10), 30);
/// ```
pub const fn prefix_len(s: &[u8], radix: u8) -> usize {
    let mut i = 0;
    while i < s.len() {
        if digit(s[i], radix).is_none() {
            return i;
        }
        i += 1;
    }
    i
}

/// Return the length of the initial sequence of ASCII `'0'`.
pub const fn zero_prefix_len(s: &[u8]) -> usize {
    let mut i = 0;
    while i < s.len() {
        if s[i] != b'0' {
            return i;
        }
        i += 1;
    }
    i
}

/// Return the start after the leading zeros and the end of digits in a non-zero number.
///
/// # Examples
///
/// ```rust
/// # use constrs::num::prefix_span;
/// assert_eq!(prefix_span(b"",     10), (0, 0));
/// assert_eq!(prefix_span(b"0",    10), (1, 1));
/// assert_eq!(prefix_span(b"00",   10), (2, 2));
/// assert_eq!(prefix_span(b"10",   10), (0, 2));
/// assert_eq!(prefix_span(b"10A",  10), (0, 2));
/// assert_eq!(prefix_span(b"010",  10), (1, 3));
/// assert_eq!(prefix_span(b"010A", 10), (1, 3));
/// assert_eq!(prefix_span(b"999999999", 10), (0, 9));
/// ```
pub const fn prefix_span(s: &[u8], radix: u8) -> (usize, usize) {
    let zeros = zero_prefix_len(s);
    let non_zeros = prefix_len(s.split_at(zeros).1, radix);
    (zeros, zeros + non_zeros)
}

/// Return the length of the initial sequence of [`digit`]s with a non-zero value.
///
/// # Examples
///
/// ```rust
/// # use constrs::num::non_zero_prefix_len;
/// assert_eq!(non_zero_prefix_len(b"",     10), 0);
/// assert_eq!(non_zero_prefix_len(b"0",    10), 0);
/// assert_eq!(non_zero_prefix_len(b"00",   10), 0);
/// assert_eq!(non_zero_prefix_len(b"10",   10), 2);
/// assert_eq!(non_zero_prefix_len(b"10A",  10), 2);
/// assert_eq!(non_zero_prefix_len(b"010",  10), 3);
/// assert_eq!(non_zero_prefix_len(b"010A", 10), 3);
/// assert_eq!(non_zero_prefix_len(b"999999999999999999999999999999", 10), 30);
/// ```
pub const fn non_zero_prefix_len(s: &[u8], radix: u8) -> usize {
    let (start, end) = prefix_span(s, radix);
    if start == end {
        return 0;
    }
    end
}

/// Decode the first [`digit`] at the start of the string.
///
/// # Examples
///
/// ```rust
/// # use constrs::num::digit_prefix;
/// assert_eq!(digit_prefix(b"0rest", 16), Some(0x0));
/// assert_eq!(digit_prefix(b"9rest", 16), Some(0x9));
/// assert_eq!(digit_prefix(b"arest", 16), Some(0xA));
/// assert_eq!(digit_prefix(b"Aaaaa", 16), Some(0xA));
/// assert_eq!(digit_prefix(b"fbbbb", 16), Some(0xF));
/// assert_eq!(digit_prefix(b"Fcccc", 16), Some(0xF));
/// assert_eq!(digit_prefix(b"gdddd", 16), None);
/// assert_eq!(digit_prefix(b"Geeee", 16), None);
/// assert_eq!(digit_prefix(b"",      16), None); // empty string
/// assert_eq!(digit_prefix(b"\0",    16), None); // NUL
/// ```
///
/// # Panics
///
/// The same as [`digit`].
///
/// ```rust,should_panic
/// # use constrs::num::digit_prefix;
/// digit_prefix(b"0", 37); // panics
/// ```
///
/// ```rust,should_panic
/// # use constrs::num::digit_prefix;
/// digit_prefix(b"", 37); // panics
/// ```
pub const fn digit_prefix(s: &[u8], radix: u8) -> Option<u8> {
    if s.is_empty() {
        assert_radix(radix); // ensure it panics even if the string is empty
        return None;
    }
    digit(s[0], radix)
}

/// Decode `u*` but only when exactly `n` number of digits that make a `u*::MAX` prefix.
// TODO export
macro_rules! strict {
    ($ty:ty, $s:expr, $radix:expr $(,$prefix:tt)? $(,)?) => {{
        let required_digits = digits($radix) as usize;
        if strict!(_prefix: $($prefix)?, $s.len(), required_digits) {
            return None;
        }
        decode($s.split_at(required_digits).0, $radix)
    }};
    (_prefix: prefix, $len:expr, $digits:expr) => { $len < $digits };
    (_prefix: , $len:expr, $digits:expr) => { $len != $digits };
}

/// Return the number of [`digit`]s to decode for `ux::MAX` in a given radix.
///
/// This uses a table of radix:
///
/// The starting element shows the number of digits for radix 3. The last shows `n + 3 - 1`th
/// radix. The rest of the radices (`(n + 3)..=36`) will have one less than the last element.
///
/// The general formula for each is `MAX.ilog(RADIX) + 1`. Each table (e.g. [`u8`]) is the `(i +
/// 1) / 2` of the `i`th element of the next one (i.e. [`u32`]) so that's another relation to
/// optimize for.  Return the type of input for `digits_u*` functions given the name.
macro_rules! fn_digits {
    ($ty:ident) => {
        /// Return the number of [`digit`]s to decode for the maximum of the input type.
        ///
        /// # Panics
        ///
        /// If the radix is smaller than 2 or larger than 36 as in [`digit`].
        pub const fn digits(radix: u8) -> u8 {
            const LEN: u8 = TABLE.len() as u8;
            match radix {
                0 | 1 | 37.. => panic_radix(),
                2 => <$ty>::BITS as u8,
                3..LEN => TABLE[(radix - 3) as usize],
                LEN.. => TABLE[TABLE.len() - 1] - 1, // the rest before 36 are the last -1
            }
        }
    };
}

/// Parse a positive number at the start of the given string in the given radix.
// TODO export
macro_rules! prefix {
    ($ty:ty, $s:expr, $radix:expr $(,)?) => {{
        if $s.is_empty() {
            assert_radix($radix); // ensure it panics even if the string is empty
        }

        let mut total_value: $ty = 0;
        let mut i = 0;
        while i < $s.len() {
            let Some(digit) = digit($s[i], $radix) else {
                break;
            };

            // the following equals `total_value = total_value * radix + digit_value;`
            let new_total_value = match total_value.checked_mul($radix as $ty) {
                Some(v) => v,
                None => break,
            };
            match new_total_value.checked_add(digit as $ty) {
                Some(v) => total_value = v,
                None => break,
            }

            i += 1;
        }

        (total_value, i)
    }};
}

/// Parse a positive number.
// TODO export
macro_rules! decode {
    ($ty:ty, $s:expr, $radix:expr $(,)?) => {{
        match prefix($s, $radix) {
            (value, len @ 1..) if len == $s.len() => Some(value),
            _ => None,
        }
    }};
}

/// Impl functions like [`mod@u8`] for other types.
macro_rules! decode_fns {
    ($ty:ident) => {
        /// Decode [`digit`]s if the whole number is equal or below the maximum.
        ///
        /// This will fail if all the given string is not a valid number. If only the prefix  is
        /// needed, try [`prefix`].
        ///
        /// See [`u8::decode`] for examples.
        pub const fn decode(s: &[u8], radix: u8) -> Option<$ty> {
            decode!($ty, s, radix)
        }

        /// Decode as many [`digit`]s while they are below or equal the maximum.
        ///
        /// This doesn't check anything as soon as the number goes invalid (too large or
        /// unparsable), if the
        /// input is required to be totally valid use [`decode`].
        ///
        /// See [`u8::prefix`] for examples.
        pub const fn prefix(s: &[u8], radix: u8) -> ($ty, usize) {
            prefix!($ty, s, radix)
        }

        /// Decode [`digit`]s if the whole string has the same digits as the maximum.
        ///
        /// This will fail if all the given string is not a valid number. If only the prefix
        /// is needed, try [`strict_prefix`].
        ///
        /// See [`u8::strict`] for examples.
        pub const fn strict(s: &[u8], radix: u8) -> Option<$ty> {
            strict!($ty, s, radix)
        }

        /// Decode [`digit`]s equal to the number of digits of the maximum from the start.
        ///
        /// This doesn't check anything as soon as the number goes invalid (too large or
        /// unparsable), if the input is required to be totally valid use [`strict`].
        ///
        /// See [`u8::strict_prefix`] for examples.
        pub const fn strict_prefix(s: &[u8], radix: u8) -> Option<$ty> {
            strict!($ty, s, radix, prefix)
        }
    };
}

/// Helpers for [`prim@u8`].
pub mod u8 {
    use super::*;

    /// Holds radices and values for radix `2..=15`.
    const ENCODE_TABLES: &[&[u8]] = &[
        &[128, 64, 32, 16, 8, 4, 2, 1],
        &[243, 81, 27, 9, 3, 1],
        &[64, 16, 4, 1],
        &[125, 25, 5, 1],
        &[216, 36, 6, 1],
        &[49, 7, 1],
        &[64, 8, 1],
        &[81, 9, 1],
        &[100, 10, 1],
        &[121, 11, 1],
        &[144, 12, 1],
        &[169, 13, 1],
        &[196, 14, 1],
        &[225, 15, 1],
    ];

    /// The code for encoding bytes. See [`encode`] and [`encode_strict`].
    macro_rules! encode_u8 {
        ($value:ident, $radix:ident, $buf:ident, $print_leading_zeros:tt) => {{
            let buf_len = match $radix {
                37.. => panic_radix(),
                0 => 0, // NOTE before value == 0
                _ if !$print_leading_zeros && $value == 0 => {
                    $buf[0] = b'0';
                    1
                }
                1 => {
                    let end = encode_u8!(if $print_leading_zeros {
                        256
                    } else {
                        $value as usize + 1
                    });
                    let mut i = 0;
                    while i < end {
                        $buf[i] = b'0';
                        i += 1;
                    }
                    i
                }
                2..16 => {
                    let mut buf_i = 0;
                    let mut print_zeros = $print_leading_zeros;

                    let mut table_i = 0;
                    let table = ENCODE_TABLES[($radix - 2) as usize];
                    while table_i < table.len() {
                        let digit_value = table[table_i];
                        let count = $value / digit_value;
                        $value %= digit_value;
                        if print_zeros || count != 0 {
                            $buf[buf_i] = encode_digit_unchecked(count);
                            buf_i += 1;
                            print_zeros = true;
                        }
                        table_i += 1;
                    }
                    buf_i
                }
                16..37 => {
                    // if radix 16 or larger, 2 digits at max
                    let count = $value / $radix;
                    $value %= $radix;
                    if $print_leading_zeros || count != 0 {
                        $buf[0] = encode_digit_unchecked(count);
                        $buf[1] = encode_digit_unchecked($value);
                        2
                    } else {
                        $buf[0] = encode_digit_unchecked($value);
                        1
                    }
                }
            };

            unsafe { core::str::from_utf8_unchecked($buf.split_at(buf_len).0) }
        }};

        (if true { $e:expr } else { $_:expr }) => {
            $e
        };

        (if false { $_:expr } else { $e:expr }) => {
            $e
        };

        (if $tt:tt { $_:expr } else { $e:expr }) => {
            compile_error!("invalid macro input given: either give `true` or `false`");
        };
    }

    /// Encode a byte with leading zeros.
    ///
    /// # Panics
    ///
    /// The same as [`digit`]: "Panics if given a radix larger than 36."
    ///
    /// Also, if the buffer cannot fit the value, it will panic.
    ///
    /// The minimum length of buffer for each radix `R` equates the following:
    ///
    /// - `R=0`: 0 (no-op)
    /// - `R=1`: 256 (always prints 256 zeros)
    /// - `R=2`: 8
    /// - `R=3`: 6
    /// - `R` in `4..=6`: 4
    /// - `R` in `7..=15`: 3
    /// - `R` in `16..=36`: 2
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use constrs::num::u8::encode_strict;
    /// fn assert_eq(value: u8, radix: u8, result: &str) {
    ///     let mut buf = [0u8; 256];
    ///     assert_eq!(encode_strict(value, radix, &mut buf), result);
    /// }
    ///
    /// assert_eq(000, 00, "");
    /// assert_eq(000, 01, &"0".repeat(256));
    /// assert_eq(001, 01, &"0".repeat(256));
    /// assert_eq(255, 01, &"0".repeat(256));
    /// assert_eq(000, 02, "00000000");
    /// assert_eq(002, 02, "00000010");
    /// assert_eq(000, 03, "000000");
    /// assert_eq(000, 04, "0000");
    /// assert_eq(000, 06, "0000");
    /// assert_eq(000, 07, "000");
    /// assert_eq(000, 08, "000");
    /// assert_eq(000, 09, "000");
    /// assert_eq(000, 10, "000");
    /// assert_eq(000, 15, "000");
    /// assert_eq(000, 16, "00");
    /// assert_eq(001, 16, "01");
    /// assert_eq(127, 16, "7f");
    /// ```
    pub const fn encode_strict(mut value: u8, radix: u8, buf: &mut [u8]) -> &str {
        // alternatively can print leading zeros based on comparisons with the ENCODE_TABLES and
        // then call encode_u8. This was the previous behavior which was removed since this is
        // simpler.
        encode_u8!(value, radix, buf, true)
    }

    /// Encode a value as a byte (multidigit) at the start of the given buffer.
    ///
    /// # Panics
    ///
    /// The same as [`digit`]: "Panics if given a radix larger than 36."
    ///
    /// Also, if the buffer cannot fit the value, it will panic.
    ///
    /// The minimum length for the buffer is always 1, except when the radix is 0 which is a no-op.
    /// The maximum is the buffer size given in [`encode_strict`] minimum.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use constrs::num::u8::encode;
    /// fn assert_eq(value: u8, radix: u8, result: &str) {
    ///     let mut buf = [0u8; 256];
    ///     assert_eq!(encode(value, radix, &mut buf), result);
    /// }
    ///
    /// assert_eq(0, 0, "");
    /// assert_eq(0, 1, "0");
    /// assert_eq(1, 1, "00");
    /// assert_eq(0, 2, "0");
    /// assert_eq(2, 2, "10");
    /// assert_eq(127, 16, "7f");
    /// ```
    pub const fn encode(mut value: u8, radix: u8, buf: &mut [u8]) -> &str {
        encode_u8!(value, radix, buf, false)
    }

    /// Decode as many [`digit`]s while smaller than the maximum, at the start of the text.
    ///
    /// This include leading zeros.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use constrs::num::u8::prefix;
    /// assert_eq!(prefix(b"",      10), (000, 0));
    /// assert_eq!(prefix(b"0",     10), (000, 1));
    /// assert_eq!(prefix(b"1",     10), (001, 1));
    /// assert_eq!(prefix(b"10",    10), (010, 2));
    /// assert_eq!(prefix(b"000",   10), (000, 3));
    /// assert_eq!(prefix(b"001",   10), (001, 3));
    /// assert_eq!(prefix(b"009",   10), (009, 3));
    /// assert_eq!(prefix(b"010",   10), (010, 3));
    /// assert_eq!(prefix(b"01F",   10), (001, 2));
    /// assert_eq!(prefix(b"01F",   16), (031, 3));
    /// assert_eq!(prefix(b"127",   10), (127, 3));
    /// assert_eq!(prefix(b"254",   10), (254, 3));
    /// assert_eq!(prefix(b"255",   10), (255, 3));
    /// assert_eq!(prefix(b"256",   10), (025, 2));
    /// assert_eq!(prefix(b"1000",  10), (100, 3));
    /// assert_eq!(prefix(b"01000", 10), (100, 4));
    /// assert_eq!(prefix(b"\0",    10), (000, 0)); // NUL
    /// assert_eq!(prefix(b"0x0A",  10), (000, 1));
    /// assert_eq!(prefix(b"0x0A",  10), (000, 1));
    /// assert_eq!(prefix(b"%0A",   10), (000, 0));
    /// ```
    pub const fn prefix(s: &[u8], radix: u8) -> (u8, usize) {
        prefix!(u8, s, radix)
    }

    /// Decode the value if valid and smaller than the maximum of the type.
    ///
    /// This includes leading zeros.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use constrs::num::u8::decode;
    /// assert_eq!(decode(b"0",     10), Some(000));
    /// assert_eq!(decode(b"1",     10), Some(001));
    /// assert_eq!(decode(b"10",    10), Some(010));
    /// assert_eq!(decode(b"000",   10), Some(000));
    /// assert_eq!(decode(b"001",   10), Some(001));
    /// assert_eq!(decode(b"009",   10), Some(009));
    /// assert_eq!(decode(b"010",   10), Some(010));
    /// assert_eq!(decode(b"01F",   10), None);
    /// assert_eq!(decode(b"01F",   16), Some(031));
    /// assert_eq!(decode(b"127",   10), Some(127));
    /// assert_eq!(decode(b"254",   10), Some(254));
    /// assert_eq!(decode(b"255",   10), Some(255));
    /// assert_eq!(decode(b"256",   10), None);
    /// assert_eq!(decode(b"1000",  10), None);
    /// assert_eq!(decode(b"01000", 10), None);
    /// assert_eq!(decode(b"\0",    10), None); // NUL
    /// assert_eq!(decode(b"0x0A",  10), None);
    /// assert_eq!(decode(b"0x0A",  10), None);
    /// assert_eq!(decode(b"%0A",  10), None);
    /// ```
    pub const fn decode(s: &[u8], radix: u8) -> Option<u8> {
        decode!(prefix, s, radix)
    }

    const TABLE: &[u8] = &[6, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3];

    fn_digits!(u8);

    /// Decode [`u8`] but only when exactly `n` number of digits that make a [`u8::MAX`] prefix.
    ///
    /// This includes leading zeros.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use constrs::num::u8::strict_prefix;
    /// assert_eq!(strict_prefix(b"00",    16), Some(0x00));
    /// assert_eq!(strict_prefix(b"01",    16), Some(0x01));
    /// assert_eq!(strict_prefix(b"09",    16), Some(0x09));
    /// assert_eq!(strict_prefix(b"0F",    16), Some(0x0F));
    /// assert_eq!(strict_prefix(b"10",    16), Some(0x10));
    /// assert_eq!(strict_prefix(b"7F",    16), Some(0x7F));
    /// assert_eq!(strict_prefix(b"FE",    16), Some(0xFE));
    /// assert_eq!(strict_prefix(b"FF",    16), Some(0xFF));
    /// assert_eq!(strict_prefix(b"100",   16), Some(0x10));
    /// assert_eq!(strict_prefix(b"\0",    16), None); // NUL
    /// assert_eq!(strict_prefix(b"0",     16), None);
    /// assert_eq!(strict_prefix(b"010",   16), Some(0x01));
    /// assert_eq!(strict_prefix(b"",      16), None);
    /// assert_eq!(strict_prefix(b"0Aaaa", 16), Some(0x0A));
    /// assert_eq!(strict_prefix(b"0Aaaa", 16), Some(0x0A));
    /// assert_eq!(strict_prefix(b"0x0A",  16), None);
    /// assert_eq!(strict_prefix(b"0x0A",  16), None);
    /// assert_eq!(strict_prefix(b"%0A",   16), None);
    /// ```
    ///
    /// [`u8`]: prim@u8
    pub const fn strict_prefix(s: &[u8], radix: u8) -> Option<u8> {
        strict!(u8, s, radix, prefix)
    }

    /// Decode [`u8`] but only when exactly `n` number of digits that make a [`u8::MAX`].
    ///
    /// This includes leading zeros.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use constrs::num::u8::strict;
    /// assert_eq!(strict(b"00",    16), Some(0x00));
    /// assert_eq!(strict(b"01",    16), Some(0x01));
    /// assert_eq!(strict(b"09",    16), Some(0x09));
    /// assert_eq!(strict(b"0F",    16), Some(0x0F));
    /// assert_eq!(strict(b"10",    16), Some(0x10));
    /// assert_eq!(strict(b"7F",    16), Some(0x7F));
    /// assert_eq!(strict(b"FE",    16), Some(0xFE));
    /// assert_eq!(strict(b"FF",    16), Some(0xFF));
    /// assert_eq!(strict(b"100",   16), None);
    /// assert_eq!(strict(b"\0",    16), None); // NUL
    /// assert_eq!(strict(b"0",     16), None);
    /// assert_eq!(strict(b"",      16), None);
    /// assert_eq!(strict(b"0Aaaa", 16), None);
    /// assert_eq!(strict(b"0Aaaa", 16), None);
    /// assert_eq!(strict(b"0x0A",  16), None);
    /// assert_eq!(strict(b"0x0A",  16), None);
    /// assert_eq!(strict(b"%0A",   16), None);
    /// ```
    ///
    /// [`u8`]: prim@u8
    pub const fn strict(s: &[u8], radix: u8) -> Option<u8> {
        strict!(u8, s, radix)
    }
}

/// Helpers for [`prim@usize`].
pub mod usize {
    use super::*;

    /// Return the number of [`digit`]s to decode for [`usize::MAX`] in a given radix.
    ///
    /// # Panics
    ///
    /// If the radix is smaller than 2 or larger than 36 as in [`digit`].
    pub const fn digits(radix: u8) -> u8 {
        #[cfg(target_pointer_width = "16")]
        {
            u16::digits(radix)
        }
        #[cfg(target_pointer_width = "32")]
        {
            u32::digits(radix)
        }
        #[cfg(target_pointer_width = "64")]
        {
            u64::digits(radix)
        }
    }

    decode_fns!(usize);
}

/// Helpers for [`prim@u16`].
///
/// [`u16::prefix`] in this module is almost euqal to [port state] parser (knowing it accepts
/// leading zeros) without the default port preferences.
///
/// [port state]:
/// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#port-state
pub mod u16 {
    use super::*;

    const TABLE: &[u8] = &[11, 8, 7, 7, 6, 6, 6, 5, 5, 5, 5, 5, 5];

    fn_digits!(u16);

    decode_fns!(u16);
}

/// Helpers for [`prim@u32`].
pub mod u32 {
    use super::*;

    const TABLE: &[u8] = &[
        21, 16, 14, 13, 12, 11, 11, 10, 10, 9, 9, 9, 9, 8, 8, 8, 8, 8, 8, 8, 8,
    ];

    fn_digits!(u32);

    decode_fns!(u32);
}

/// Helpers for [`prim@u64`].
pub mod u64 {
    use super::*;

    const TABLE: &[u8] = &[
        41, 32, 28, 25, 23, 22, 21, 20, 19, 18, 18, 17, 17, 16, 16, 16, 16, 15, 15, 15, 15, 14, 14,
        14, 14, 14, 14, 14,
    ];

    fn_digits!(u64);

    decode_fns!(u64);
}

/// Helpers for [`prim@u128`].
pub mod u128 {
    use super::*;

    const TABLE: &[u8] = &[
        81, 64, 56, 50, 46, 43, 41, 39, 38, 36, 35, 34, 33, 32, 32, 31, 31, 30, 30, 29, 29, 28, 28,
        28, 27, 27, 27, 27, 26, 26, 26, 26,
    ];

    fn_digits!(u128);
    decode_fns!(u128);
}

#[cfg(test)]
mod tests {
    use super::*;

    mod encode_u8 {
        use super::*;

        /// Encode and check a radix.
        ///
        /// If given inputs like `b!(R=radix, number => "string", number2 => "string2")`
        /// selective checks can happen, if given an list like `b!(radix, "string",
        /// "string2", ...)` numbers are checked in the given order (provide a exactly 256
        /// elements for 0 to 255).
        macro_rules! b {
            ($r:literal, $v:expr => $s:expr $(, $vs:expr => $ss:expr)+$(,)?) => {
                b!($r, $v => $s);
                $(b!($r, $vs => $ss);)+
            };

            ($r:literal, $v:expr => $s:expr$(,)?) => {
                {
                    let mut buf = [0u8; 255];
                    let result = u8::encode($v, $r, &mut buf);
                    assert_eq!(result, $s, concat!(
                        stringify!($v),
                        " expected to return ",
                        stringify!($s),
                        " in radix ",
                        stringify!($r),
                    ));
                }
            };

            ($r:literal, $s:literal $(, $ss:literal)* $(,)?) => {
                {
                    let arr = [$s $(, $ss)*];
                    if arr.len() != 256 {
                        panic!("expected exactly 256 ordered strings for each of u8 variants")
                    }
                    for (i, s) in arr.into_iter().enumerate() {
                        b!($r, i as u8 => s);
                    }
                }
            };
        }

        #[test]
        fn test_r0() {
            b!(
                0, "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "",
                "", "", "", "", ""
            );
        }

        #[test]
        fn test_r1() {
            // radix 1 is the number of zeros - 1.
            b!(1, 0 => "0", 1 => "00", 2 => "000");
        }

        #[test]
        fn test_r2() {
            for i in 0..=255 {
                let mut buf = [0u8; 8];
                let result = u8::encode(i, 2, &mut buf);
                assert_eq!(result, format!("{i:b}"));
            }
        }

        #[test]
        fn test_r3() {
            b!(
                3, "0", "1", "2", "10", "11", "12", "20", "21", "22", "100", "101", "102", "110",
                "111", "112", "120", "121", "122", "200", "201", "202", "210", "211", "212", "220",
                "221", "222", "1000", "1001", "1002", "1010", "1011", "1012", "1020", "1021",
                "1022", "1100", "1101", "1102", "1110", "1111", "1112", "1120", "1121", "1122",
                "1200", "1201", "1202", "1210", "1211", "1212", "1220", "1221", "1222", "2000",
                "2001", "2002", "2010", "2011", "2012", "2020", "2021", "2022", "2100", "2101",
                "2102", "2110", "2111", "2112", "2120", "2121", "2122", "2200", "2201", "2202",
                "2210", "2211", "2212", "2220", "2221", "2222", "10000", "10001", "10002", "10010",
                "10011", "10012", "10020", "10021", "10022", "10100", "10101", "10102", "10110",
                "10111", "10112", "10120", "10121", "10122", "10200", "10201", "10202", "10210",
                "10211", "10212", "10220", "10221", "10222", "11000", "11001", "11002", "11010",
                "11011", "11012", "11020", "11021", "11022", "11100", "11101", "11102", "11110",
                "11111", "11112", "11120", "11121", "11122", "11200", "11201", "11202", "11210",
                "11211", "11212", "11220", "11221", "11222", "12000", "12001", "12002", "12010",
                "12011", "12012", "12020", "12021", "12022", "12100", "12101", "12102", "12110",
                "12111", "12112", "12120", "12121", "12122", "12200", "12201", "12202", "12210",
                "12211", "12212", "12220", "12221", "12222", "20000", "20001", "20002", "20010",
                "20011", "20012", "20020", "20021", "20022", "20100", "20101", "20102", "20110",
                "20111", "20112", "20120", "20121", "20122", "20200", "20201", "20202", "20210",
                "20211", "20212", "20220", "20221", "20222", "21000", "21001", "21002", "21010",
                "21011", "21012", "21020", "21021", "21022", "21100", "21101", "21102", "21110",
                "21111", "21112", "21120", "21121", "21122", "21200", "21201", "21202", "21210",
                "21211", "21212", "21220", "21221", "21222", "22000", "22001", "22002", "22010",
                "22011", "22012", "22020", "22021", "22022", "22100", "22101", "22102", "22110",
                "22111", "22112", "22120", "22121", "22122", "22200", "22201", "22202", "22210",
                "22211", "22212", "22220", "22221", "22222", "100000", "100001", "100002",
                "100010", "100011", "100012", "100020", "100021", "100022", "100100", "100101",
                "100102", "100110"
            );
        }

        #[test]
        fn test_r4() {
            b!(
                4, "0", "1", "2", "3", "10", "11", "12", "13", "20", "21", "22", "23", "30", "31",
                "32", "33", "100", "101", "102", "103", "110", "111", "112", "113", "120", "121",
                "122", "123", "130", "131", "132", "133", "200", "201", "202", "203", "210", "211",
                "212", "213", "220", "221", "222", "223", "230", "231", "232", "233", "300", "301",
                "302", "303", "310", "311", "312", "313", "320", "321", "322", "323", "330", "331",
                "332", "333", "1000", "1001", "1002", "1003", "1010", "1011", "1012", "1013",
                "1020", "1021", "1022", "1023", "1030", "1031", "1032", "1033", "1100", "1101",
                "1102", "1103", "1110", "1111", "1112", "1113", "1120", "1121", "1122", "1123",
                "1130", "1131", "1132", "1133", "1200", "1201", "1202", "1203", "1210", "1211",
                "1212", "1213", "1220", "1221", "1222", "1223", "1230", "1231", "1232", "1233",
                "1300", "1301", "1302", "1303", "1310", "1311", "1312", "1313", "1320", "1321",
                "1322", "1323", "1330", "1331", "1332", "1333", "2000", "2001", "2002", "2003",
                "2010", "2011", "2012", "2013", "2020", "2021", "2022", "2023", "2030", "2031",
                "2032", "2033", "2100", "2101", "2102", "2103", "2110", "2111", "2112", "2113",
                "2120", "2121", "2122", "2123", "2130", "2131", "2132", "2133", "2200", "2201",
                "2202", "2203", "2210", "2211", "2212", "2213", "2220", "2221", "2222", "2223",
                "2230", "2231", "2232", "2233", "2300", "2301", "2302", "2303", "2310", "2311",
                "2312", "2313", "2320", "2321", "2322", "2323", "2330", "2331", "2332", "2333",
                "3000", "3001", "3002", "3003", "3010", "3011", "3012", "3013", "3020", "3021",
                "3022", "3023", "3030", "3031", "3032", "3033", "3100", "3101", "3102", "3103",
                "3110", "3111", "3112", "3113", "3120", "3121", "3122", "3123", "3130", "3131",
                "3132", "3133", "3200", "3201", "3202", "3203", "3210", "3211", "3212", "3213",
                "3220", "3221", "3222", "3223", "3230", "3231", "3232", "3233", "3300", "3301",
                "3302", "3303", "3310", "3311", "3312", "3313", "3320", "3321", "3322", "3323",
                "3330", "3331", "3332", "3333",
            );
        }

        #[test]
        fn test_r5() {
            b!(
                5, "0", "1", "2", "3", "4", "10", "11", "12", "13", "14", "20", "21", "22", "23",
                "24", "30", "31", "32", "33", "34", "40", "41", "42", "43", "44", "100", "101",
                "102", "103", "104", "110", "111", "112", "113", "114", "120", "121", "122", "123",
                "124", "130", "131", "132", "133", "134", "140", "141", "142", "143", "144", "200",
                "201", "202", "203", "204", "210", "211", "212", "213", "214", "220", "221", "222",
                "223", "224", "230", "231", "232", "233", "234", "240", "241", "242", "243", "244",
                "300", "301", "302", "303", "304", "310", "311", "312", "313", "314", "320", "321",
                "322", "323", "324", "330", "331", "332", "333", "334", "340", "341", "342", "343",
                "344", "400", "401", "402", "403", "404", "410", "411", "412", "413", "414", "420",
                "421", "422", "423", "424", "430", "431", "432", "433", "434", "440", "441", "442",
                "443", "444", "1000", "1001", "1002", "1003", "1004", "1010", "1011", "1012",
                "1013", "1014", "1020", "1021", "1022", "1023", "1024", "1030", "1031", "1032",
                "1033", "1034", "1040", "1041", "1042", "1043", "1044", "1100", "1101", "1102",
                "1103", "1104", "1110", "1111", "1112", "1113", "1114", "1120", "1121", "1122",
                "1123", "1124", "1130", "1131", "1132", "1133", "1134", "1140", "1141", "1142",
                "1143", "1144", "1200", "1201", "1202", "1203", "1204", "1210", "1211", "1212",
                "1213", "1214", "1220", "1221", "1222", "1223", "1224", "1230", "1231", "1232",
                "1233", "1234", "1240", "1241", "1242", "1243", "1244", "1300", "1301", "1302",
                "1303", "1304", "1310", "1311", "1312", "1313", "1314", "1320", "1321", "1322",
                "1323", "1324", "1330", "1331", "1332", "1333", "1334", "1340", "1341", "1342",
                "1343", "1344", "1400", "1401", "1402", "1403", "1404", "1410", "1411", "1412",
                "1413", "1414", "1420", "1421", "1422", "1423", "1424", "1430", "1431", "1432",
                "1433", "1434", "1440", "1441", "1442", "1443", "1444", "2000", "2001", "2002",
                "2003", "2004", "2010",
            );
        }

        #[test]
        fn test_r6() {
            b!(
                6, "0", "1", "2", "3", "4", "5", "10", "11", "12", "13", "14", "15", "20", "21",
                "22", "23", "24", "25", "30", "31", "32", "33", "34", "35", "40", "41", "42", "43",
                "44", "45", "50", "51", "52", "53", "54", "55", "100", "101", "102", "103", "104",
                "105", "110", "111", "112", "113", "114", "115", "120", "121", "122", "123", "124",
                "125", "130", "131", "132", "133", "134", "135", "140", "141", "142", "143", "144",
                "145", "150", "151", "152", "153", "154", "155", "200", "201", "202", "203", "204",
                "205", "210", "211", "212", "213", "214", "215", "220", "221", "222", "223", "224",
                "225", "230", "231", "232", "233", "234", "235", "240", "241", "242", "243", "244",
                "245", "250", "251", "252", "253", "254", "255", "300", "301", "302", "303", "304",
                "305", "310", "311", "312", "313", "314", "315", "320", "321", "322", "323", "324",
                "325", "330", "331", "332", "333", "334", "335", "340", "341", "342", "343", "344",
                "345", "350", "351", "352", "353", "354", "355", "400", "401", "402", "403", "404",
                "405", "410", "411", "412", "413", "414", "415", "420", "421", "422", "423", "424",
                "425", "430", "431", "432", "433", "434", "435", "440", "441", "442", "443", "444",
                "445", "450", "451", "452", "453", "454", "455", "500", "501", "502", "503", "504",
                "505", "510", "511", "512", "513", "514", "515", "520", "521", "522", "523", "524",
                "525", "530", "531", "532", "533", "534", "535", "540", "541", "542", "543", "544",
                "545", "550", "551", "552", "553", "554", "555", "1000", "1001", "1002", "1003",
                "1004", "1005", "1010", "1011", "1012", "1013", "1014", "1015", "1020", "1021",
                "1022", "1023", "1024", "1025", "1030", "1031", "1032", "1033", "1034", "1035",
                "1040", "1041", "1042", "1043", "1044", "1045", "1050", "1051", "1052", "1053",
                "1054", "1055", "1100", "1101", "1102", "1103",
            );
        }

        #[test]
        fn test_r7() {
            b!(
                7, "0", "1", "2", "3", "4", "5", "6", "10", "11", "12", "13", "14", "15", "16",
                "20", "21", "22", "23", "24", "25", "26", "30", "31", "32", "33", "34", "35", "36",
                "40", "41", "42", "43", "44", "45", "46", "50", "51", "52", "53", "54", "55", "56",
                "60", "61", "62", "63", "64", "65", "66", "100", "101", "102", "103", "104", "105",
                "106", "110", "111", "112", "113", "114", "115", "116", "120", "121", "122", "123",
                "124", "125", "126", "130", "131", "132", "133", "134", "135", "136", "140", "141",
                "142", "143", "144", "145", "146", "150", "151", "152", "153", "154", "155", "156",
                "160", "161", "162", "163", "164", "165", "166", "200", "201", "202", "203", "204",
                "205", "206", "210", "211", "212", "213", "214", "215", "216", "220", "221", "222",
                "223", "224", "225", "226", "230", "231", "232", "233", "234", "235", "236", "240",
                "241", "242", "243", "244", "245", "246", "250", "251", "252", "253", "254", "255",
                "256", "260", "261", "262", "263", "264", "265", "266", "300", "301", "302", "303",
                "304", "305", "306", "310", "311", "312", "313", "314", "315", "316", "320", "321",
                "322", "323", "324", "325", "326", "330", "331", "332", "333", "334", "335", "336",
                "340", "341", "342", "343", "344", "345", "346", "350", "351", "352", "353", "354",
                "355", "356", "360", "361", "362", "363", "364", "365", "366", "400", "401", "402",
                "403", "404", "405", "406", "410", "411", "412", "413", "414", "415", "416", "420",
                "421", "422", "423", "424", "425", "426", "430", "431", "432", "433", "434", "435",
                "436", "440", "441", "442", "443", "444", "445", "446", "450", "451", "452", "453",
                "454", "455", "456", "460", "461", "462", "463", "464", "465", "466", "500", "501",
                "502", "503", "504", "505", "506", "510", "511", "512", "513",
            );
        }

        #[test]
        fn test_r8() {
            for i in 0..=255 {
                let mut buf = [0u8; 3];
                let result = u8::encode(i, 8, &mut buf);
                assert_eq!(result, format!("{i:o}"));
            }
        }

        #[test]
        fn test_r9() {
            b!(
                9, "0", "1", "2", "3", "4", "5", "6", "7", "8", "10", "11", "12", "13", "14", "15",
                "16", "17", "18", "20", "21", "22", "23", "24", "25", "26", "27", "28", "30", "31",
                "32", "33", "34", "35", "36", "37", "38", "40", "41", "42", "43", "44", "45", "46",
                "47", "48", "50", "51", "52", "53", "54", "55", "56", "57", "58", "60", "61", "62",
                "63", "64", "65", "66", "67", "68", "70", "71", "72", "73", "74", "75", "76", "77",
                "78", "80", "81", "82", "83", "84", "85", "86", "87", "88", "100", "101", "102",
                "103", "104", "105", "106", "107", "108", "110", "111", "112", "113", "114", "115",
                "116", "117", "118", "120", "121", "122", "123", "124", "125", "126", "127", "128",
                "130", "131", "132", "133", "134", "135", "136", "137", "138", "140", "141", "142",
                "143", "144", "145", "146", "147", "148", "150", "151", "152", "153", "154", "155",
                "156", "157", "158", "160", "161", "162", "163", "164", "165", "166", "167", "168",
                "170", "171", "172", "173", "174", "175", "176", "177", "178", "180", "181", "182",
                "183", "184", "185", "186", "187", "188", "200", "201", "202", "203", "204", "205",
                "206", "207", "208", "210", "211", "212", "213", "214", "215", "216", "217", "218",
                "220", "221", "222", "223", "224", "225", "226", "227", "228", "230", "231", "232",
                "233", "234", "235", "236", "237", "238", "240", "241", "242", "243", "244", "245",
                "246", "247", "248", "250", "251", "252", "253", "254", "255", "256", "257", "258",
                "260", "261", "262", "263", "264", "265", "266", "267", "268", "270", "271", "272",
                "273", "274", "275", "276", "277", "278", "280", "281", "282", "283", "284", "285",
                "286", "287", "288", "300", "301", "302", "303", "304", "305", "306", "307", "308",
                "310", "311", "312", "313",
            );
        }

        #[test]
        fn test_r10() {
            for i in 0..=255 {
                let mut buf = [0u8; 3];
                let result = u8::encode(i, 10, &mut buf);
                assert_eq!(result, format!("{i}"));
            }
        }

        #[test]
        fn test_r11() {
            b!(
                11, "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "10", "11", "12", "13",
                "14", "15", "16", "17", "18", "19", "1a", "20", "21", "22", "23", "24", "25", "26",
                "27", "28", "29", "2a", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39",
                "3a", "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4a", "50", "51",
                "52", "53", "54", "55", "56", "57", "58", "59", "5a", "60", "61", "62", "63", "64",
                "65", "66", "67", "68", "69", "6a", "70", "71", "72", "73", "74", "75", "76", "77",
                "78", "79", "7a", "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "8a",
                "90", "91", "92", "93", "94", "95", "96", "97", "98", "99", "9a", "a0", "a1", "a2",
                "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aa", "100", "101", "102", "103", "104",
                "105", "106", "107", "108", "109", "10a", "110", "111", "112", "113", "114", "115",
                "116", "117", "118", "119", "11a", "120", "121", "122", "123", "124", "125", "126",
                "127", "128", "129", "12a", "130", "131", "132", "133", "134", "135", "136", "137",
                "138", "139", "13a", "140", "141", "142", "143", "144", "145", "146", "147", "148",
                "149", "14a", "150", "151", "152", "153", "154", "155", "156", "157", "158", "159",
                "15a", "160", "161", "162", "163", "164", "165", "166", "167", "168", "169", "16a",
                "170", "171", "172", "173", "174", "175", "176", "177", "178", "179", "17a", "180",
                "181", "182", "183", "184", "185", "186", "187", "188", "189", "18a", "190", "191",
                "192", "193", "194", "195", "196", "197", "198", "199", "19a", "1a0", "1a1", "1a2",
                "1a3", "1a4", "1a5", "1a6", "1a7", "1a8", "1a9", "1aa", "200", "201", "202", "203",
                "204", "205", "206", "207", "208", "209", "20a", "210", "211", "212",
            );
        }

        #[test]
        fn test_r12() {
            b!(
                12, "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "10", "11", "12",
                "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "20", "21", "22", "23", "24",
                "25", "26", "27", "28", "29", "2a", "2b", "30", "31", "32", "33", "34", "35", "36",
                "37", "38", "39", "3a", "3b", "40", "41", "42", "43", "44", "45", "46", "47", "48",
                "49", "4a", "4b", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "5a",
                "5b", "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6a", "6b", "70",
                "71", "72", "73", "74", "75", "76", "77", "78", "79", "7a", "7b", "80", "81", "82",
                "83", "84", "85", "86", "87", "88", "89", "8a", "8b", "90", "91", "92", "93", "94",
                "95", "96", "97", "98", "99", "9a", "9b", "a0", "a1", "a2", "a3", "a4", "a5", "a6",
                "a7", "a8", "a9", "aa", "ab", "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8",
                "b9", "ba", "bb", "100", "101", "102", "103", "104", "105", "106", "107", "108",
                "109", "10a", "10b", "110", "111", "112", "113", "114", "115", "116", "117", "118",
                "119", "11a", "11b", "120", "121", "122", "123", "124", "125", "126", "127", "128",
                "129", "12a", "12b", "130", "131", "132", "133", "134", "135", "136", "137", "138",
                "139", "13a", "13b", "140", "141", "142", "143", "144", "145", "146", "147", "148",
                "149", "14a", "14b", "150", "151", "152", "153", "154", "155", "156", "157", "158",
                "159", "15a", "15b", "160", "161", "162", "163", "164", "165", "166", "167", "168",
                "169", "16a", "16b", "170", "171", "172", "173", "174", "175", "176", "177", "178",
                "179", "17a", "17b", "180", "181", "182", "183", "184", "185", "186", "187", "188",
                "189", "18a", "18b", "190", "191", "192", "193",
            );
        }

        #[test]
        fn test_r13() {
            b!(
                13, "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "10", "11",
                "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "1c", "20", "21", "22",
                "23", "24", "25", "26", "27", "28", "29", "2a", "2b", "2c", "30", "31", "32", "33",
                "34", "35", "36", "37", "38", "39", "3a", "3b", "3c", "40", "41", "42", "43", "44",
                "45", "46", "47", "48", "49", "4a", "4b", "4c", "50", "51", "52", "53", "54", "55",
                "56", "57", "58", "59", "5a", "5b", "5c", "60", "61", "62", "63", "64", "65", "66",
                "67", "68", "69", "6a", "6b", "6c", "70", "71", "72", "73", "74", "75", "76", "77",
                "78", "79", "7a", "7b", "7c", "80", "81", "82", "83", "84", "85", "86", "87", "88",
                "89", "8a", "8b", "8c", "90", "91", "92", "93", "94", "95", "96", "97", "98", "99",
                "9a", "9b", "9c", "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aa",
                "ab", "ac", "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9", "ba", "bb",
                "bc", "c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9", "ca", "cb", "cc",
                "100", "101", "102", "103", "104", "105", "106", "107", "108", "109", "10a", "10b",
                "10c", "110", "111", "112", "113", "114", "115", "116", "117", "118", "119", "11a",
                "11b", "11c", "120", "121", "122", "123", "124", "125", "126", "127", "128", "129",
                "12a", "12b", "12c", "130", "131", "132", "133", "134", "135", "136", "137", "138",
                "139", "13a", "13b", "13c", "140", "141", "142", "143", "144", "145", "146", "147",
                "148", "149", "14a", "14b", "14c", "150", "151", "152", "153", "154", "155", "156",
                "157", "158", "159", "15a", "15b", "15c", "160", "161", "162", "163", "164", "165",
                "166", "167", "168",
            );
        }

        #[test]
        fn test_r14() {
            b!(
                14, "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "10",
                "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "1c", "1d", "20",
                "21", "22", "23", "24", "25", "26", "27", "28", "29", "2a", "2b", "2c", "2d", "30",
                "31", "32", "33", "34", "35", "36", "37", "38", "39", "3a", "3b", "3c", "3d", "40",
                "41", "42", "43", "44", "45", "46", "47", "48", "49", "4a", "4b", "4c", "4d", "50",
                "51", "52", "53", "54", "55", "56", "57", "58", "59", "5a", "5b", "5c", "5d", "60",
                "61", "62", "63", "64", "65", "66", "67", "68", "69", "6a", "6b", "6c", "6d", "70",
                "71", "72", "73", "74", "75", "76", "77", "78", "79", "7a", "7b", "7c", "7d", "80",
                "81", "82", "83", "84", "85", "86", "87", "88", "89", "8a", "8b", "8c", "8d", "90",
                "91", "92", "93", "94", "95", "96", "97", "98", "99", "9a", "9b", "9c", "9d", "a0",
                "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aa", "ab", "ac", "ad", "b0",
                "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9", "ba", "bb", "bc", "bd", "c0",
                "c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8", "c9", "ca", "cb", "cc", "cd", "d0",
                "d1", "d2", "d3", "d4", "d5", "d6", "d7", "d8", "d9", "da", "db", "dc", "dd",
                "100", "101", "102", "103", "104", "105", "106", "107", "108", "109", "10a", "10b",
                "10c", "10d", "110", "111", "112", "113", "114", "115", "116", "117", "118", "119",
                "11a", "11b", "11c", "11d", "120", "121", "122", "123", "124", "125", "126", "127",
                "128", "129", "12a", "12b", "12c", "12d", "130", "131", "132", "133", "134", "135",
                "136", "137", "138", "139", "13a", "13b", "13c", "13d", "140", "141", "142", "143",
            );
        }

        #[test]
        fn test_r15() {
            b!(
                15, "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e",
                "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "1c", "1d",
                "1e", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "2a", "2b", "2c",
                "2d", "2e", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "3a", "3b",
                "3c", "3d", "3e", "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4a",
                "4b", "4c", "4d", "4e", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59",
                "5a", "5b", "5c", "5d", "5e", "60", "61", "62", "63", "64", "65", "66", "67", "68",
                "69", "6a", "6b", "6c", "6d", "6e", "70", "71", "72", "73", "74", "75", "76", "77",
                "78", "79", "7a", "7b", "7c", "7d", "7e", "80", "81", "82", "83", "84", "85", "86",
                "87", "88", "89", "8a", "8b", "8c", "8d", "8e", "90", "91", "92", "93", "94", "95",
                "96", "97", "98", "99", "9a", "9b", "9c", "9d", "9e", "a0", "a1", "a2", "a3", "a4",
                "a5", "a6", "a7", "a8", "a9", "aa", "ab", "ac", "ad", "ae", "b0", "b1", "b2", "b3",
                "b4", "b5", "b6", "b7", "b8", "b9", "ba", "bb", "bc", "bd", "be", "c0", "c1", "c2",
                "c3", "c4", "c5", "c6", "c7", "c8", "c9", "ca", "cb", "cc", "cd", "ce", "d0", "d1",
                "d2", "d3", "d4", "d5", "d6", "d7", "d8", "d9", "da", "db", "dc", "dd", "de", "e0",
                "e1", "e2", "e3", "e4", "e5", "e6", "e7", "e8", "e9", "ea", "eb", "ec", "ed", "ee",
                "100", "101", "102", "103", "104", "105", "106", "107", "108", "109", "10a", "10b",
                "10c", "10d", "10e", "110", "111", "112", "113", "114", "115", "116", "117", "118",
                "119", "11a", "11b", "11c", "11d", "11e", "120",
            );
        }

        #[test]
        fn test_r16() {
            for i in 0..=255 {
                let mut buf = [0u8; 2];
                let result = u8::encode(i, 16, &mut buf);
                assert_eq!(result, format!("{i:x}"));
            }
        }

        #[test]
        fn test_r17() {
            b!(
                17, "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f",
                "g", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b", "1c",
                "1d", "1e", "1f", "1g", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29",
                "2a", "2b", "2c", "2d", "2e", "2f", "2g", "30", "31", "32", "33", "34", "35", "36",
                "37", "38", "39", "3a", "3b", "3c", "3d", "3e", "3f", "3g", "40", "41", "42", "43",
                "44", "45", "46", "47", "48", "49", "4a", "4b", "4c", "4d", "4e", "4f", "4g", "50",
                "51", "52", "53", "54", "55", "56", "57", "58", "59", "5a", "5b", "5c", "5d", "5e",
                "5f", "5g", "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6a", "6b",
                "6c", "6d", "6e", "6f", "6g", "70", "71", "72", "73", "74", "75", "76", "77", "78",
                "79", "7a", "7b", "7c", "7d", "7e", "7f", "7g", "80", "81", "82", "83", "84", "85",
                "86", "87", "88", "89", "8a", "8b", "8c", "8d", "8e", "8f", "8g", "90", "91", "92",
                "93", "94", "95", "96", "97", "98", "99", "9a", "9b", "9c", "9d", "9e", "9f", "9g",
                "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", "a8", "a9", "aa", "ab", "ac", "ad",
                "ae", "af", "ag", "b0", "b1", "b2", "b3", "b4", "b5", "b6", "b7", "b8", "b9", "ba",
                "bb", "bc", "bd", "be", "bf", "bg", "c0", "c1", "c2", "c3", "c4", "c5", "c6", "c7",
                "c8", "c9", "ca", "cb", "cc", "cd", "ce", "cf", "cg", "d0", "d1", "d2", "d3", "d4",
                "d5", "d6", "d7", "d8", "d9", "da", "db", "dc", "dd", "de", "df", "dg", "e0", "e1",
                "e2", "e3", "e4", "e5", "e6", "e7", "e8", "e9", "ea", "eb", "ec", "ed", "ee", "ef",
                "eg", "f0",
            );
        }

        #[test]
        fn test_r36() {
            b!(
                36, "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f",
                "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v",
                "w", "x", "y", "z", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19",
                "1a", "1b", "1c", "1d", "1e", "1f", "1g", "1h", "1i", "1j", "1k", "1l", "1m", "1n",
                "1o", "1p", "1q", "1r", "1s", "1t", "1u", "1v", "1w", "1x", "1y", "1z", "20", "21",
                "22", "23", "24", "25", "26", "27", "28", "29", "2a", "2b", "2c", "2d", "2e", "2f",
                "2g", "2h", "2i", "2j", "2k", "2l", "2m", "2n", "2o", "2p", "2q", "2r", "2s", "2t",
                "2u", "2v", "2w", "2x", "2y", "2z", "30", "31", "32", "33", "34", "35", "36", "37",
                "38", "39", "3a", "3b", "3c", "3d", "3e", "3f", "3g", "3h", "3i", "3j", "3k", "3l",
                "3m", "3n", "3o", "3p", "3q", "3r", "3s", "3t", "3u", "3v", "3w", "3x", "3y", "3z",
                "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "4a", "4b", "4c", "4d",
                "4e", "4f", "4g", "4h", "4i", "4j", "4k", "4l", "4m", "4n", "4o", "4p", "4q", "4r",
                "4s", "4t", "4u", "4v", "4w", "4x", "4y", "4z", "50", "51", "52", "53", "54", "55",
                "56", "57", "58", "59", "5a", "5b", "5c", "5d", "5e", "5f", "5g", "5h", "5i", "5j",
                "5k", "5l", "5m", "5n", "5o", "5p", "5q", "5r", "5s", "5t", "5u", "5v", "5w", "5x",
                "5y", "5z", "60", "61", "62", "63", "64", "65", "66", "67", "68", "69", "6a", "6b",
                "6c", "6d", "6e", "6f", "6g", "6h", "6i", "6j", "6k", "6l", "6m", "6n", "6o", "6p",
                "6q", "6r", "6s", "6t", "6u", "6v", "6w", "6x", "6y", "6z", "70", "71", "72", "73",
            );
        }
    }
}
