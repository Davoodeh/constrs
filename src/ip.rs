//! IPv4 and IPv6 utilities.

/// Holds IPv4 parsers and utilities.
pub mod v4 {
    pub use core::net::Ipv4Addr as Addr;

    use crate::num;

    /// If true, the candidate must be parsed with the [`parse`] function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use constrs::ip::v4::ends_in_a_number;
    /// assert!(ends_in_a_number(b"127.0.0.1"));
    /// assert!(ends_in_a_number(b"127.0.0.1."));
    /// assert!(ends_in_a_number(b"127.0.0.1000."));
    /// assert!(ends_in_a_number(b"1"));
    /// assert!(ends_in_a_number(b"1."));
    /// assert!(ends_in_a_number(b"127.0.0.00."));
    /// assert!(ends_in_a_number(b"127.0.0.00"));
    /// assert!(ends_in_a_number(b"127.0.0.0x"));
    /// assert!(ends_in_a_number(b"127.0.0.0X"));
    /// assert!(ends_in_a_number(b"127.0.0.0XFFFFFFF"));
    /// assert!(ends_in_a_number(b"127.0.0.0XFFFFFFF."));
    /// assert!(ends_in_a_number(b"invalid.a.0.0XFFFFFFF."));
    ///
    /// assert!(!ends_in_a_number(b"1.."));
    /// assert!(!ends_in_a_number(b""));
    /// assert!(!ends_in_a_number(b"."));
    /// assert!(!ends_in_a_number(b".."));
    /// assert!(!ends_in_a_number(b"127.0.0.1.."));
    /// ```
    ///
    /// See
    /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ends-in-a-number-checker>
    pub const fn ends_in_a_number(s: &[u8]) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut pointer = s.len();

        // remove the "fully qualified" dot at the end if any
        if s[pointer - 1] == b'.' {
            if s.len() == 1 {
                return false;
            }
            pointer -= 1;
        }

        let end = pointer;

        // since everything is strict, the value is at maximum 4 characters deep from the end
        // ".123"
        let mut start = None;
        while pointer > 0 {
            pointer -= 1;
            if s[pointer] == b'.' {
                start = Some(pointer + 1);
                break;
            }
        }

        let start = match start {
            Some(start) => start,
            None if pointer == 0 => 0,
            None => return false,
        };

        // &s[start..end] in const
        let part = s.split_at(start).1.split_at(end - start).0;

        match number(part) {
            Ok(_) => true,
            Err(NumberError::Empty) => false, // recoverable but explicitly not permitted
            Err(NumberError::NonDecimal(_, _)) | Err(NumberError::OutOfRange) => true,
            Err(e) if e.is_recoverable() => true,
            _ => false,
        }
    }

    /// Errors produced when [`parse`] is called on an invalid IPv4.
    ///
    /// Errors here have higher precedence over their children. For example, if there are
    /// multiple parts, that will short circuit before any other validation is done.
    ///
    /// This changes the definition of some errors like [IPv4 Empty Part]
    /// ([`NumberError::Empty`]). For example, "127.0.0.1." will throw [`Self::TooManyParts`]
    /// but in the examples this is the exact error for the aforementioned empty.
    ///
    /// See [`parse`] for examples and when each is thrown.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Error {
        /// Holds the number of parts seen (guaranteed 0..=3).
        ///
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-too-few-parts>
        TooFewParts(usize),
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-too-many-parts>
        TooManyParts,
        /// Holds the index of the part and its associated error.
        ///
        /// The number is guaranteed to be 0..=3.
        Number(usize, NumberError),
    }

    impl Error {
        /// Is the error a recoverable soft error (non-failure)?
        ///
        /// The [`NumberError::OutOfRange`] is an exception since it is conditionally a failure.
        /// While [`NumberError::is_recoverable`] always returns false for that value, here the
        /// nuance is kept.
        ///
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#validation-error>
        pub const fn is_recoverable(&self) -> bool {
            match self {
                Self::TooFewParts(_) | Self::Number(0..3, NumberError::OutOfRange) => true,
                Self::Number(_, e) if e.is_recoverable() => true,
                _ => false,
            }
        }
    }

    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::TooFewParts(part) => {
                    write!(f, "expected 4 parts for IPv4 and got {part}")
                }
                Self::TooManyParts => "expected 4 parts for IPv4 and got more".fmt(f),
                Self::Number(part, e) => write!(f, "part {part} has numeric error: {e}"),
            }
        }
    }

    impl core::error::Error for Error {}

    /// Unrecoverable errors thrown from [`number`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NumberError {
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-empty-part>
        Empty,
        /// Number is a valid octadecimal or hexadecimal number but no a decimal number.
        NonDecimal(u8, DecimalError),
        /// Number is not in 0..=255 range.
        ///
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-out-of-range-part>
        OutOfRange,
        /// A character outside 0..=0x7F range is seen.
        ///
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-non-ascii-input>
        NonAscii,
        /// The given character is ASCII but not valid for the type of radix given.
        ///
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-non-numeric-part>
        NonNumeric,
    }

    impl NumberError {
        /// Is the error a recoverable soft error (non-failure)?
        ///
        /// The [`Self::OutOfRange`] is an exception since it is conditionally a failure.
        /// Here, that always returns false.
        ///
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#validation-error>
        pub const fn is_recoverable(&self) -> bool {
            matches!(self, Self::Empty | Self::NonAscii | Self::NonDecimal(_, _))
        }
    }

    impl core::fmt::Display for NumberError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::Empty => "expected a number but received nothing".fmt(f),
                Self::NonDecimal(num, e) => write!(
                    f,
                    "expected a decimal number and got {num} but it's not as a decimal: {e}"
                ),
                Self::OutOfRange => "expected a number strictly smaller than 256".fmt(f),
                Self::NonAscii => "expected an ASCII number string".fmt(f),
                Self::NonNumeric => "expected a number".fmt(f),
            }
        }
    }

    impl core::error::Error for NumberError {}

    /// Number is a valid octadecimal or hexadecimal number but not a decimal number.
    ///
    /// See
    /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-non-decimal-part>
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DecimalError {
        /// The given input is a hexadecimal number (prefixed with "0x" or "0X").
        Hex,
        /// The given input is a [`Self::Hex`] but after the prefix nothing is found.
        EmptyHex,
        /// The given input is an octal number (prefixed with "0").
        ///
        /// This does not have a corresponding "EmptyOct" variant since an empty prefix of 0
        /// is parsed as a valid decimal 0.
        Oct,
    }

    impl core::fmt::Display for DecimalError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::Hex => "expected a decimal number but received a hexadecimal \
                                (prefixed with '0x' or '0X')"
                    .fmt(f),
                Self::EmptyHex => {
                    "expected a decimal number but received an empty hexadecimal \
                        ('0x' or '0X')"
                        .fmt(f)
                }
                Self::Oct => "expected a decimal number but received an octal number \
                                (prefixed with an extra zero like '012')"
                    .fmt(f),
            }
        }
    }

    impl core::error::Error for DecimalError {}

    /// Parse an [IPv4 number parser] using a "loose" implementation.
    ///
    /// This implementation tries parsing even if a recoverable error happens.
    ///
    /// [`NumberError::NonAscii`] and [`NumberError::NonNumeric`] have equal precedence in this
    /// parser meaning the first problematic character from left to write throws either proper.
    ///
    /// # Examples
    ///
    /// ## General
    ///
    /// ```rust
    /// # use constrs::ip::v4::{number, NumberError::*, DecimalError::*};
    /// assert_eq!(number(b""),         Err(Empty));
    /// assert_eq!(number(b"0"),        Ok(0));
    /// assert_eq!(number(b"1"),        Ok(1));
    /// assert_eq!(number(b"2"),        Ok(2));
    /// assert_eq!(number(b"9"),        Ok(9));
    /// assert_eq!(number(b"10"),       Ok(10));
    /// assert_eq!(number(b"254"),      Ok(254));
    /// assert_eq!(number(b"255"),      Ok(255));
    /// assert_eq!(number(b"0255"),     Err(NonDecimal(0o255, Oct))); // no leading
    /// assert_eq!(number(b"256"),      Err(OutOfRange));
    /// assert_eq!(number(b"1000"),     Err(OutOfRange));
    /// assert_eq!(number(b"\xDB\xB1"), Err(NonAscii)); // \u{6F1} = \xDB\xB1 = Arabic 1
    /// assert_eq!(number(b"A"),        Err(NonNumeric));
    /// ```
    ///
    /// ## Hexadecimal (`0x` or `0X` prefix)
    ///
    /// ```rust
    /// # use constrs::ip::v4::{number, NumberError::*, DecimalError::*};
    /// assert_eq!(number(b"0x"),         Err(NonDecimal(000, EmptyHex)));
    /// assert_eq!(number(b"0X"),         Err(NonDecimal(000, EmptyHex)));
    /// assert_eq!(number(b"0x1"),        Err(NonDecimal(001, Hex)));
    /// assert_eq!(number(b"0X1"),        Err(NonDecimal(001, Hex)));
    /// assert_eq!(number(b"0xA"),        Err(NonDecimal(010, Hex)));
    /// assert_eq!(number(b"0XA"),        Err(NonDecimal(010, Hex)));
    /// assert_eq!(number(b"0xa"),        Err(NonDecimal(010, Hex)));
    /// assert_eq!(number(b"0Xa"),        Err(NonDecimal(010, Hex)));
    /// assert_eq!(number(b"0xF"),        Err(NonDecimal(015, Hex)));
    /// assert_eq!(number(b"0XF"),        Err(NonDecimal(015, Hex)));
    /// assert_eq!(number(b"0xFF"),       Err(NonDecimal(255, Hex)));
    /// assert_eq!(number(b"0XFF"),       Err(NonDecimal(255, Hex)));
    /// assert_eq!(number(b"0x00000FF"),  Err(NonDecimal(255, Hex)));
    /// assert_eq!(number(b"0\xD70"),     Err(NonAscii)); // mathematical x multiply sign
    /// assert_eq!(number(b"0x\xDB\xB1"), Err(NonAscii)); // \u{6F1} = \xDB\xB1 = Arabic 1
    /// assert_eq!(number(b"0x100"),      Err(OutOfRange));
    /// assert_eq!(number(b"0X100"),      Err(OutOfRange));
    /// assert_eq!(number(b"0x10a"),      Err(OutOfRange));
    /// assert_eq!(number(b"0X10a"),      Err(OutOfRange));
    /// assert_eq!(number(b"1X10a"),      Err(NonNumeric));
    /// assert_eq!(number(b"0xG"),        Err(NonNumeric));
    /// assert_eq!(number(b"0a0"),        Err(NonNumeric));
    /// ```
    ///
    /// ## Octadecimal (`0` prefix)
    ///
    /// ```rust
    /// # use constrs::ip::v4::{number, NumberError::*, DecimalError::*};
    /// assert_eq!(number(b"00"),        Err(NonDecimal(000, Oct)));
    /// assert_eq!(number(b"01"),        Err(NonDecimal(001, Oct)));
    /// assert_eq!(number(b"02"),        Err(NonDecimal(002, Oct)));
    /// assert_eq!(number(b"07"),        Err(NonDecimal(007, Oct)));
    /// assert_eq!(number(b"010"),       Err(NonDecimal(008, Oct)));
    /// assert_eq!(number(b"0376"),      Err(NonDecimal(254, Oct)));
    /// assert_eq!(number(b"0377"),      Err(NonDecimal(255, Oct)));
    /// assert_eq!(number(b"0000377"),   Err(NonDecimal(255, Oct)));
    /// assert_eq!(number(b"0\xDB\xB1"), Err(NonAscii)); // \u{6F1} = \xDB\xB1 = Arabic 1
    /// assert_eq!(number(b"0400"),      Err(OutOfRange));
    /// assert_eq!(number(b"08"),        Err(NonNumeric));
    /// assert_eq!(number(b"0A"),        Err(NonNumeric));
    /// ```
    ///
    /// [IPv4 number parser]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-number-parser
    pub const fn number(s: &[u8]) -> Result<u8, NumberError> {
        use {DecimalError::*, NumberError::*};

        if s.is_empty() {
            return Err(NumberError::Empty);
        }

        /// Parse the given string starting from the given point in a given number system.
        macro_rules! parse_u8 {
            (
                $s:ident[$skip:literal..],
                $radix:literal,
                |$valid:ident $(: u8)?| $expr:expr $(,)?
            ) => {
                match num::u8::decode(s.split_at($skip).1, $radix) {
                    Some($valid) => $expr,
                    None => {
                        let mut i = $skip;
                        let mut e = OutOfRange;
                        while i < s.len() {
                            if !s[i].is_ascii() {
                                e = NonAscii;
                                break;
                            }
                            if num::digit(s[i], $radix).is_none() {
                                e = NonNumeric;
                                break;
                            }
                            i += 1;
                        }
                        Err(e)
                    }
                }
            };
        }

        // if prefixed with 0x parse hex
        if s.len() > 1 && s[0] == b'0' && s[1].eq_ignore_ascii_case(&b'x') {
            if s.len() == 2 {
                return Err(NonDecimal(0, EmptyHex));
            }

            return parse_u8!(s[2..], 16, |v| Err(NonDecimal(v, Hex)));
        }

        // if prefixed with 0 parse oct
        if s.len() > 1 && s[0] == b'0' {
            return parse_u8!(s[1..], 8, |v| Err(NonDecimal(v, Oct)));
        }

        // no prefix, it must be just a decimal
        parse_u8!(s[0..], 10, |v| Ok(v))
    }

    // TODO add (parse_)loose

    /// Parse a [valid IPv4-address] using a "strict" version of [IPv4 Parser].
    ///
    /// This is strict in the sense that it will return on the first validation error. The
    /// algorithm is slightly modified to account for not having a [strict split] (due to
    /// dynamic allocation).
    ///
    /// The root [`Error`]s have higher precedence compared to their children (e.g.
    /// [`Error::TooManyParts`] is first reported compared to [`NumberError::Empty`] when either
    /// is applicable). See [`Error`] documentation for more.
    ///
    /// [valid IPv4-address]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#valid-ipv4-address-string
    /// [IPv4 Parser]: https://url.spec.whatwg.org/#concept-ipv4-parser
    /// [strict split]:
    /// https://infra.spec.whatwg.org/commit-snapshots/3f984adcd24a6d5c53cc26b3e737701808003f3e/#strictly-split
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use constrs::ip::v4::{parse, Addr, Error::*, DecimalError::*, NumberError::*};
    /// assert_eq!(parse(b"127.0.0.1"),      Ok(Addr::new(127, 0, 0, 1)));
    /// assert_eq!(parse(b""),               Err(TooFewParts(0)));
    /// assert_eq!(parse(b"1"),              Err(TooFewParts(1)));
    /// assert_eq!(parse(b"."),              Err(TooFewParts(2)));
    /// assert_eq!(parse(b"1.2"),            Err(TooFewParts(2)));
    /// assert_eq!(parse(b"test.42"),        Err(TooFewParts(2)));
    /// assert_eq!(parse(b".."),             Err(TooFewParts(3)));
    /// assert_eq!(parse(b"1.2.3"),          Err(TooFewParts(3)));
    /// assert_eq!(parse(b"...."),           Err(TooManyParts)); // differs the spec (Empty)
    /// assert_eq!(parse(b"1.2.3.4."),       Err(TooManyParts)); // differs the spec (Empty)
    /// assert_eq!(parse(b".1.2.3.4"),       Err(TooManyParts));
    /// assert_eq!(parse(b"1.2.3.4.5"),      Err(TooManyParts));
    /// assert_eq!(parse(b".1.2.3.4.5."),    Err(TooManyParts));
    /// assert_eq!(parse(b"127.0.0.1.."),    Err(TooManyParts));
    /// assert_eq!(parse(b"127..0.0.1"),     Err(TooManyParts));
    /// assert_eq!(parse(b".127.0.1"),       Err(Number(0, Empty)));
    /// assert_eq!(parse(b"127..0.1"),       Err(Number(1, Empty)));
    /// assert_eq!(parse(b"127.0..1"),       Err(Number(2, Empty)));
    /// assert_eq!(parse(b"127.0.1."),       Err(Number(3, Empty)));
    /// assert_eq!(parse(b"127.0.0.0x0"),    Err(Number(3, NonDecimal(0, Hex))));
    /// assert_eq!(parse(b"127.0.0.0X0"),    Err(Number(3, NonDecimal(0, Hex))));
    /// assert_eq!(parse(b"127.0.0.00"),     Err(Number(3, NonDecimal(0, Oct))));
    /// assert_eq!(parse(b"127.0.0.01"),     Err(Number(3, NonDecimal(1, Oct))));
    /// assert_eq!(parse(b"127.0.0.A"),      Err(Number(3, NonNumeric)));
    /// assert_eq!(parse(b"127.0.0.08"),     Err(Number(3, NonNumeric)));
    /// assert_eq!(parse(b"127.0.0.0xG"),    Err(Number(3, NonNumeric)));
    /// assert_eq!(parse(b"127.0.0.0XG"),    Err(Number(3, NonNumeric)));
    /// assert_eq!(parse(b"127.0.0.x"),      Err(Number(3, NonNumeric)));
    /// assert_eq!(parse(b"1.1.4000.1"),     Err(Number(2, OutOfRange)));
    /// assert_eq!(parse(b"1.1.1.\xDB\xB1"), Err(Number(3, NonAscii))); // \xDB\xB1 = Arabic 1
    /// ```
    pub const fn parse(s: &[u8]) -> Result<Addr, Error> {
        if s.is_empty() {
            return Err(Error::TooFewParts(0));
        }

        let mut address = [0u8; 4];
        let mut piece_index = 0;
        let mut piece_start = 0;
        let mut pointer = 0;

        let mut error = None;

        // the loop goes until the absolute end (len instead of len-1) and sees that as a
        // "virtual dot"
        while pointer <= s.len() {
            if pointer == s.len() || s[pointer] == b'.' {
                if piece_index > 3 {
                    return Err(Error::TooManyParts);
                }

                // this error check allows for TooManyParts to be reported correctly even
                // if sees problematic parts
                if error.is_none() {
                    // no &s[piece_start..pointer] in const fn
                    let part = s.split_at(piece_start).1.split_at(pointer - piece_start).0;
                    // no ? in const fn
                    match number(part) {
                        Ok(v) => address[piece_index] = v,
                        Err(e) => error = Some(Error::Number(piece_index, e)),
                    }
                    piece_start = pointer + 1;
                }
                piece_index += 1;
            }

            pointer += 1;
        }

        // if the loop has ended the indices must on 4 (the absolute end of the `address`)
        if piece_index != 4 {
            return Err(Error::TooFewParts(piece_index));
        }

        // after TooManyParts and TooFewParts, report the first error
        if let Some(e) = error {
            return Err(e);
        }

        // from_octets requires 1.91
        let [a, b, c, d] = address;
        Ok(Addr::new(a, b, c, d))
    }
}

/// Holds IPv6 parsers and utilities.
pub mod v6 {
    pub use core::net::Ipv6Addr as Addr;

    /// Errors produced when [`parse`] is called on an invalid IPv6.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Error {
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv6-invalid-compression>
        InvalidCompression,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv6-too-many-pieces>
        TooManyPieces,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv6-multiple-compression>
        MultipleCompression,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv6-invalid-code-point>
        InvalidCodePoint,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv6-piece-leading-zero>
        PieceLeadingZero,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv6-too-few-pieces>
        TooFewPieces,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-in-ipv6-too-few-parts>
        Ipv4InIpv6TooFewParts,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-in-ipv6-too-many-pieces>
        Ipv4InIpv6TooManyPieces,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-in-ipv6-invalid-code-point>
        Ipv4InIpv6InvalidCodePoint,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv4-in-ipv6-out-of-range-part>
        Ipv4InIpv6OutOfRangePart,
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#ipv6-unclosed>
        Unclosed,
    }

    impl Error {
        /// Is the error a recoverable soft error (non-failure)?
        ///
        /// See
        /// <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#validation-error>
        pub const fn is_recoverable(&self) -> bool {
            matches!(self, Self::PieceLeadingZero)
        }
    }

    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::InvalidCompression => "invalid compression (::)",
                Self::TooManyPieces => "too many pieces",
                Self::MultipleCompression => "multiple compressions (::) used",
                Self::InvalidCodePoint => "invalid codepoint or unexpected end of the string",
                Self::PieceLeadingZero => "a piece cannot have a leading zero",
                Self::TooFewPieces => "expected more pieces",
                Self::Ipv4InIpv6TooFewParts => "IPv4-in-IPv6: too few parts, expected 4",
                Self::Ipv4InIpv6TooManyPieces => "IPv4-in-IPv6: too many parts, expected 4",
                Self::Ipv4InIpv6InvalidCodePoint => "IPv4-in-IPv6: invalid codepoint",
                Self::Ipv4InIpv6OutOfRangePart => {
                    "IPv4-in-IPv6: expected a part strictly smaller than 256"
                }
                Self::Unclosed => "expected an ending of ']' around the IP and never found it",
            }
            .fmt(f)
        }
    }

    impl core::error::Error for Error {}

    /// Parse a [valid IPv6-address] using a "strict" implementation of [IPv6 Parser].
    ///
    /// This is strict in the sense that it will return on the first validation error.
    /// No validation error is spared, even if [`Error::is_recoverable`]. This decision is made
    /// to keep the function signature simple.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use constrs::ip::v6::{parse, Addr, Error::*};
    /// assert_eq!(parse(b"::"),                       Ok(Addr::new(0, 0, 0, 0, 0, 0, 0, 0)));
    /// assert_eq!(parse(b"::1"),                      Ok(Addr::new(0, 0, 0, 0, 0, 0, 0, 1)));
    /// assert_eq!(parse(b"::A"),                      Ok(Addr::new(0, 0, 0, 0, 0, 0, 0, 0xA)));
    /// assert_eq!(parse(b"1::"),                      Ok(Addr::new(1, 0, 0, 0, 0, 0, 0, 0)));
    /// assert_eq!(parse(b"A::"),                      Ok(Addr::new(0xA, 0, 0, 0, 0, 0, 0, 0)));
    /// assert_eq!(parse(b"1:2:3:4:5:6:7:8"),          Ok(Addr::new(1, 2, 3, 4, 5, 6, 7, 8)));
    /// assert_eq!(parse(b":1"),                       Err(InvalidCompression));
    /// assert_eq!(parse(b"1:2:3:4:5:6:7:8:9"),        Err(TooManyPieces));
    /// assert_eq!(parse(b"1::1::1"),                  Err(MultipleCompression));
    /// assert_eq!(parse(b"1:2:3!:4"),                 Err(InvalidCodePoint));
    /// assert_eq!(parse(b"1:2:3:"),                   Err(InvalidCodePoint));
    /// assert_eq!(parse(b"1:2:3"),                    Err(TooFewPieces));
    /// assert_eq!(parse(b"::01"),                     Err(PieceLeadingZero));
    /// assert_eq!(parse(b"1:1:1:1:1:1:1:127.0.0.1"),  Err(Ipv4InIpv6TooManyPieces));
    /// assert_eq!(parse(b"ffff::.0.0.1"),             Err(Ipv4InIpv6InvalidCodePoint));
    /// assert_eq!(parse(b"ffff::127.0.xyz.1"),        Err(Ipv4InIpv6InvalidCodePoint));
    /// assert_eq!(parse(b"ffff::127.0xyz"),           Err(Ipv4InIpv6InvalidCodePoint));
    /// assert_eq!(parse(b"ffff::127.00.0.1"),         Err(Ipv4InIpv6InvalidCodePoint));
    /// assert_eq!(parse(b"ffff::127.0.0.1.2"),        Err(Ipv4InIpv6InvalidCodePoint));
    /// assert_eq!(parse(b"ffff::127.0.0.4000"),       Err(Ipv4InIpv6OutOfRangePart));
    /// assert_eq!(parse(b"ffff::127.0.4000.0"),       Err(Ipv4InIpv6OutOfRangePart));
    /// assert_eq!(parse(b"ffff::127.0.0"),            Err(Ipv4InIpv6TooFewParts));
    /// ```
    ///
    /// [valid IPv6-address]:
    /// https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/#valid-ipv6-address-string
    /// [IPv6 Parser]: https://url.spec.whatwg.org/#concept-ipv6-parser
    pub const fn parse(s: &[u8]) -> Result<Addr, Error> {
        if s.is_empty() {
            return Err(Error::TooFewPieces);
        }

        // 1:
        let mut address = [0u16; 8];
        // 2:
        let mut piece_index = 0;
        // 3:
        let mut compress = None;
        // 4:
        let mut pointer = 0; // represents a shift in `s` initial *const u8 but safer

        // 5:
        if s[pointer] == b':' {
            // 5.1:
            if (s.len() - 1) != pointer && s[pointer + 1] != b':' {
                return Err(Error::InvalidCompression);
            }
            // 5.2:
            pointer += 2;
            // 5.3:
            piece_index += 1;
            compress = Some(piece_index);
        }

        // 6:
        while pointer < s.len() {
            // 6.1:
            if piece_index == 8 {
                return Err(Error::TooManyPieces);
            }

            // 6.2:
            if s[pointer] == b':' {
                // 6.2.1:
                if compress.is_some() {
                    return Err(Error::MultipleCompression);
                }
                // 6.2.2:
                pointer += 1;
                piece_index += 1;
                compress = Some(piece_index);
                continue;
            }

            // 6.3:
            let mut value: u16 = 0;
            let mut length = 0;

            // 6.4: while length < 4 and the next character exists and is an ASCII hex
            while length < 4
                && pointer < s.len()
                && let Some(new_value) = match s[pointer] {
                    v @ b'0'..=b'9' => Some(v - b'0'),
                    v @ b'a'..=b'f' => Some(v - b'a' + 10),
                    v @ b'A'..=b'F' => Some(v - b'A' + 10),
                    _ => None,
                }
            {
                value = (value * 0x10) + (new_value as u16);
                pointer += 1;
                length += 1;
            }

            if pointer < s.len() && s[pointer] == b'.' {
                // 6.5.1:
                if length == 0 {
                    return Err(Error::Ipv4InIpv6InvalidCodePoint);
                }
                // 6.5.3:
                if piece_index > 6 {
                    return Err(Error::Ipv4InIpv6TooManyPieces);
                }
                // 6.5.2:
                pointer -= length;
                // 6.5.4:
                let mut numbers_seen = 0;

                // 6.5.5:
                while pointer < s.len() {
                    // 6.5.5.2:
                    if numbers_seen > 0 {
                        // 6.5.5.2.2:
                        if !(s[pointer] == b'.' && numbers_seen < 4) {
                            return Err(Error::Ipv4InIpv6InvalidCodePoint);
                        }
                        // 6.5.5.2.1:
                        pointer += 1;
                    }

                    // 6.5.5.3: ensure the loop runs at least once
                    if !(pointer < s.len() && s[pointer].is_ascii_digit()) {
                        return Err(Error::Ipv4InIpv6InvalidCodePoint);
                    }

                    // 6.5.5.1:
                    let mut ipv4_piece = None;

                    // 6.5.5.4:
                    while pointer < s.len() && s[pointer].is_ascii_digit() {
                        // 6.5.5.4.1:
                        let number = (s[pointer] - b'0') as u16;
                        let new_ipv4_piece: u16 = match ipv4_piece {
                            // 6.5.5.4.3:
                            Some(0) => return Err(Error::Ipv4InIpv6InvalidCodePoint),
                            // 6.5.5.4.4:
                            Some(v) => (v as u16 * 10) + number,
                            // 6.5.5.4.2:
                            None => number,
                        };
                        // 6.5.5.4.5:
                        // TryInto::<u8>::try_into(new_ipv4_piece)? in const
                        match new_ipv4_piece {
                            byte @ 0..=255 => ipv4_piece = Some(byte as u8),
                            _ => return Err(Error::Ipv4InIpv6OutOfRangePart),
                        }

                        // 6.5.5.4.6:
                        pointer += 1;
                    }

                    // 6.5.5.5:
                    address[piece_index] =
                        (address[piece_index] * 0x100) + ipv4_piece.unwrap() as u16;

                    // 6.5.5.6:
                    numbers_seen += 1;
                    // 6.5.5.7:
                    if numbers_seen == 2 || numbers_seen == 4 {
                        piece_index += 1;
                    }
                }

                // 6.5.6:
                if numbers_seen != 4 {
                    return Err(Error::Ipv4InIpv6TooFewParts);
                }

                // 6.5.7:
                break;
            } else if pointer < s.len() && s[pointer] == b':' {
                // 6.6.1:
                pointer += 1;
                // 6.6.2:
                if pointer >= s.len() {
                    return Err(Error::InvalidCodePoint);
                }
            } else if pointer < s.len() {
                // 6.7:
                return Err(Error::InvalidCodePoint);
            }

            // 6.8:
            if length > 1 && value < 0x10u16.pow((length as u32) - 1) {
                return Err(Error::PieceLeadingZero);
            }

            // 6.9:
            address[piece_index] = value;
            // 6.10:
            piece_index += 1;
        }

        // 7:
        if let Some(compress) = compress {
            // 7.1:
            let mut swaps = piece_index - compress;
            // 7.2:
            piece_index = 7;
            // 7.3:
            while piece_index != 0 && swaps > 0 {
                address.swap(piece_index, compress + swaps - 1);
                piece_index -= 1;
                swaps -= 1;
            }
        } else if piece_index != 8 {
            return Err(Error::TooFewPieces);
        }

        // 1.91 has "from_segments"
        let [a, b, c, d, e, f, g, h] = address;
        Ok(Addr::new(a, b, c, d, e, f, g, h))
    }

    /// [`parse`] when input is surrounded by "[" and "]" or [`None`] if "[" is not seen.
    pub const fn bracketed(mut s: &[u8]) -> Option<Result<Addr, Error>> {
        if (!s.is_empty()) && s[0] == b'[' {
            if s[s.len() - 1] != b']' {
                return Some(Err(Error::Unclosed));
            }
            // &s[1..(s.len() - 1)] not in const fn
            s = s.split_at(1).1.split_at((s.len() - 1) - 1).0;

            // no ? in const fn
            return Some(parse(s));
        }
        None
    }

    /// [`parse`] from the start of the string "[" until the first "]" and return the length.
    pub const fn bracketed_prefix(mut s: &[u8]) -> Option<Result<(Addr, usize), Error>> {
        if s.is_empty() || s[0] != b'[' {
            return None;
        }

        let mut i = 0;
        while i < s.len() && s[i] != b']' {
            i += 1;
        }
        if s[i] == b']' {
            // &s[1..(s.len() - 1)] not in const fn
            s = s.split_at(1).1.split_at((i - 1) - 1).0;
            return Some(match parse(s) {
                Ok(v) => Ok((v, s.len())),
                Err(e) => Err(e),
            });
        }

        Some(Err(Error::Unclosed))
    }
}
