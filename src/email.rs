//! Holds WHATWG email parsers in const time without allocation.

use crate::slice;

/// Regex of [WHATWG HTML, 4.10.5.1.5 Email State (`type=email`)] equal to [`is_email`].
///
/// # Examples
///
/// ```rust
/// # use constrs::email::REGEX;
/// let re = regex::Regex::new(REGEX).unwrap();
/// ```
///
/// [WHATWG HTML, 4.10.5.1.5 Email State (`type=email`)]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#email-state-(type=email)
pub const REGEX: &str = r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";

/// Is the given character a value from [RFC 5322 `atext`].
///
/// [RFC 5322 `atext`]: https://www.rfc-editor.org/info/rfc5322/#section-3.2.3
pub const fn is_atext(byte: u8) -> bool {
    matches!(byte,
        b'!' | b'#'..=b'\'' /* #$%&' */
            | b'*' | b'+' | b'-' | b'/'
            | b'0'..=b'9'
            | b'=' | b'?'
            | b'A'..=b'Z'
            | b'_' | b'`'
            | b'a'..=b'~' /* a-z{|}~ */
    )
}

/// Get as far as possible for "before `'@'`" and return the length (0 means invalid).
///
/// According to the [valid email] syntax, an email starts with "at least one [`is_atext`] or `'.'`
/// before `'@'`."
///
/// # Examples
///
/// ```rust
/// # use constrs::email::before_at_prefix_len;
/// assert_eq!(before_at_prefix_len(b""),     0);
/// assert_eq!(before_at_prefix_len(b"\0@"),  0);
/// assert_eq!(before_at_prefix_len(b"\\@"),  0);
/// assert_eq!(before_at_prefix_len(b"@"),    0);
/// assert_eq!(before_at_prefix_len(b"abc"),  3);
/// assert_eq!(before_at_prefix_len(b"abc@"), 3);
/// ```
///
/// [valid email]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#valid-e-mail-address
pub const fn before_at_prefix_len(s: &[u8]) -> usize {
    let mut i = 0;
    while let Some(&c) = slice::get(s, i)
        && (c == b'.' || is_atext(c))
    {
        i += 1;
    }
    i
}

/// Get as far as `@` (inclusive) and return the length (valid range 2..).
///
/// According to the [valid email] syntax, an email starts with "at least one [`is_atext`] or `'.'`
/// before `'@'`."
///
/// # Examples
///
/// ```rust
/// # use constrs::email::over_at_prefix_len;
/// assert_eq!(over_at_prefix_len(b""),     0);
/// assert_eq!(over_at_prefix_len(b"\0@"),  0);
/// assert_eq!(over_at_prefix_len(b"\\@"),  0);
/// assert_eq!(over_at_prefix_len(b"@"),    0);
/// assert_eq!(over_at_prefix_len(b"abc"),  0);
/// assert_eq!(over_at_prefix_len(b"abc@"), 4);
/// ```
///
/// [valid email]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#valid-e-mail-address
pub const fn over_at_prefix_len(s: &[u8]) -> usize {
    if let len @ 1.. = before_at_prefix_len(s)
        && let Some(&b'@') = slice::get(s, len)
    {
        return len + 1;
    }
    0
}

/// Get as far as possible in a [valid email] for a `label` (returns `0..=63`, 0 is invalid).
///
/// # Examples
///
/// ```rust
/// # use constrs::email::label_prefix_len;
/// let f = |i| label_prefix_len(i);
/// assert_eq!(f(b""),     0);
/// assert_eq!(f(b"\0"),   0);
/// assert_eq!(f(b"\\"),   0);
/// assert_eq!(f(b"-"),    0);
/// assert_eq!(f(b"a"),    1);
/// assert_eq!(f(b"2"),    1);
/// assert_eq!(f(b"abc"),  3);
/// assert_eq!(f(b"a0123456789"),  11);
/// assert_eq!(f(b"a0123456789012345678901234567890123456789012345678901234567890e"),  63);
/// assert_eq!(f(b"a-------------------------------------------------------------e"),  63);
/// assert_eq!(f(b"a--------------------------------------------------------------"),  1);
/// assert_eq!(f(b"a------------------------------------------------------------e-"),  62);
/// assert_eq!(f(b"a----------------------------x-------------------------------e-"),  62);
/// assert_eq!(f(b"a----------------------------x------------------------------e--"),  61);
/// assert_eq!(f(b"-a------------------------------------------------------------e"),  0);
/// assert_eq!(f(b"-a-------------------------------------------------------------"),  0);
/// assert_eq!(f(b"-a-----------------------------------------------------------e-"),  0);
/// assert_eq!(f(b"-a---------------------------x-------------------------------e-"),  0);
/// assert_eq!(f(b"-a---------------------------x------------------------------e--"),  0);
/// assert_eq!(f(b"a0123456789012345678901234567890123456789012345678901234567890erestrest"),  63);
/// ```
///
/// [valid email]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#valid-e-mail-address
pub const fn label_prefix_len(s: &[u8]) -> usize {
    // must start with [a-zA-Z0-9]
    if s.is_empty() || !s[0].is_ascii_alphanumeric() {
        return 0;
    }

    // the ending must be ascii alphanumeric but the middle needs not to be so if that is the
    // case, remember what was the last alphanumeric to revert back
    let mut last_acceptable_i = 0;

    // a maximum of 61 characters are acceptable before the start and the end
    let end = if s.len() < 62 { s.len() } else { 62 };
    let mut i = 1;
    while i < end {
        match s[i] {
            c if c.is_ascii_alphanumeric() => last_acceptable_i = i,
            b'-' => {}
            _ => break,
        }
        i += 1;
    }

    // if 62 are parsed to be valid, require the last (63th) to be alphanumeric
    // if the last is not alphanumeric, the previous alphanumeric is the end.
    1 + (if let Some(c) = slice::get(s, end)
        && c.is_ascii_alphanumeric()
    {
        i
    } else {
        last_acceptable_i
    })
}

/// Get the length of the valid prefix of (after `'@'`) `$label $(. $labels)*` (0 is invalid).
///
/// See [`label_prefix_len`] for more information.
///
/// # Examples
///
/// ```rust
/// # use constrs::email::post_at_prefix_len;
/// assert_eq!(post_at_prefix_len(b""),     0);
/// assert_eq!(post_at_prefix_len(b"."),    0);
/// assert_eq!(post_at_prefix_len(b"\0"),   0);
/// assert_eq!(post_at_prefix_len(b"\\"),   0);
/// assert_eq!(post_at_prefix_len(b"-"),    0);
/// assert_eq!(post_at_prefix_len(b"-."),   0);
/// assert_eq!(post_at_prefix_len(b".-"),   0);
/// assert_eq!(post_at_prefix_len(b".a"),   0);
/// assert_eq!(post_at_prefix_len(b"a"),    1);
/// assert_eq!(post_at_prefix_len(b"a."),   1);
/// assert_eq!(post_at_prefix_len(b"a.b"),  3);
/// assert_eq!(post_at_prefix_len(b"a.-b"), 1);
/// assert_eq!(post_at_prefix_len(b"a.b-"), 3);
/// ```
pub const fn post_at_prefix_len(s: &[u8]) -> usize {
    let mut len = label_prefix_len(s);
    // initially, require a label before checking the dot-prefix ones.
    if len == 0 {
        return 0;
    }

    loop {
        // if at the end or the next is not separated by a dot, break
        if s.len() == len || s[len] != b'.' {
            return len;
        }
        let new = label_prefix_len(s.split_at(len + 1).1);
        // nothing else is parsable, the end of valid values is reached
        if new == 0 {
            return len;
        }
        len += 1 + new; // also add '.': no overflow otherwise `s` wouldn't be possible to make
    }
}

/// Return the length of the [valid email] of the start of `s` (the rest is ignored, 0 is invalid).
///
/// # Examples
///
/// ```rust
/// # use constrs::email::email_prefix_len;
/// assert_eq!(email_prefix_len(b""),       0);
/// assert_eq!(email_prefix_len(b"@"),      0);
/// assert_eq!(email_prefix_len(b"@e.com"), 0);
/// assert_eq!(email_prefix_len(b"a@e"),    3);
/// assert_eq!(email_prefix_len(b"a!#$%&'*/-+01239fz=?~_{|}`z@e"),     29);
/// assert_eq!(email_prefix_len(b"a!#$%&'*/-+01239fz=?~_{|}`z@e-"),    29); // no ending accepted
/// assert_eq!(email_prefix_len(b"a!#$%&'*/-+01239fz=?~_{|}`z@-e"),    0);
/// assert_eq!(email_prefix_len(b"a!#$%&'*/-+01239fz=?~_{|}`z@e-e"),   31);
/// assert_eq!(email_prefix_len(b"a!#$%&'*/-+01239fz=?~_{|}`z@e-e."),  31); // no dot ending
/// assert_eq!(email_prefix_len(b"a!#$%&'*/-+01239fz=?~_{|}`z@e-e.a"), 33);
/// ```
///
/// Tests at WPT:
/// <https://github.com/web-platform-tests/wpt/blob/68d964a5f292bc4b2c2eba2f21e2b916190b6ad2/html/semantics/forms/the-input-element/email.html#L1>
///
/// ```rust
/// # use constrs::email::email_prefix_len;
/// assert_eq!(email_prefix_len(b"user2@example.com\n"), 17); // doesn't include \n
/// assert_eq!(email_prefix_len(b"user3@example.com\r"), 17); // doesn't include \r
/// assert_eq!(email_prefix_len(b"user4@example.com"),   17);
/// assert_eq!(email_prefix_len(b"example.com"),         0);
/// ```
///
/// [valid email]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#valid-e-mail-address
pub const fn email_prefix_len(s: &[u8]) -> usize {
    let including_at = over_at_prefix_len(s);
    if including_at == 0 {
        return 0;
    }
    let post_at = post_at_prefix_len(s.split_at(including_at).1);
    if post_at == 0 {
        return 0;
    }

    including_at + post_at
}

/// Is the whole string a [valid email]?
///
/// [valid email]:
/// https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/#valid-e-mail-address
pub const fn is_email(s: &[u8]) -> bool {
    email_prefix_len(s) != 0
}
