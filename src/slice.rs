//! Holds helpers for a more unconditionally `const` slice operations.

/// Get a slice element with a [`usize`] index, uncoditionally const.
///
/// This aims to be a replacement for the [slice primitive `get` method] until it's const
/// stable.
///
/// [slice primitive `get` method]:
/// https://doc.rust-lang.org/core/primitive.slice.html#method.get
pub const fn get<T>(s: &[T], index: usize) -> Option<&T> {
    if index < s.len() {
        return Some(&s[index]);
    }
    None
}

/// Get a slice element with a [`usize`] index, uncoditionally const (mutable).
///
/// This aims to be a replacement for the [slice primitive `get` method] until it's const
/// stable.
///
/// [slice primitive `get` method]:
/// https://doc.rust-lang.org/core/primitive.slice.html#method.get
pub const fn get_mut<T>(s: &mut [T], index: usize) -> Option<&mut T> {
    if index < s.len() {
        return Some(&mut s[index]);
    }
    None
}

/// Get the first slice element, uncoditionally const.
pub const fn first<T>(s: &[T]) -> Option<&T> {
    if s.is_empty() {
        return None;
    }
    Some(&s[0])
}

/// Get the first slice element, uncoditionally const (mutable).
pub const fn first_mut<T>(s: &mut [T]) -> Option<&mut T> {
    if s.is_empty() {
        return None;
    }
    Some(&mut s[0])
}

/// Get the last slice element, uncoditionally const.
pub const fn last<T>(s: &[T]) -> Option<&T> {
    if s.is_empty() {
        return None;
    }
    Some(&s[s.len() - 1])
}

/// Get the last slice element, uncoditionally const (mutable).
pub const fn last_mut<T>(s: &mut [T]) -> Option<&mut T> {
    if s.is_empty() {
        return None;
    }
    Some(&mut s[s.len() - 1])
}

/// Return `&s[start..end]` in const.
///
/// To include the `end` (`&s[start..=end]`), use [`range_inclusive_get`]. There is no
/// `range_to_get` (`&s[..end]`). Use this function with a `start = 0` or see
/// [`[]::split_at`].
///
/// # Panics
///
/// If the `start` or `end` is strictly larger than `s.len()` or if `end` is strictly smaller than
/// `start`.
///
/// ```rust,should_panic
/// # use constrs::slice::range_get;
/// range_get(b"012345", 6, 0); // start > end
/// ```
///
/// ```rust,should_panic
/// # use constrs::slice::range_get;
/// range_get(b"012345", 0, 8); // end > len
/// ```
///
/// # Examples
///
/// ```rust
/// # use constrs::slice::range_get;
/// let v = b"012345";
/// assert_eq!(range_get(v, 2, 4), &v[2..4]); // "23"
/// assert_eq!(range_get(v, 0, 6), &v[0..6]); // "012345": end == len
/// assert_eq!(range_get(v, 6, 6), &v[6..6]); // "":     start == end
/// ```
pub const fn range_get<T>(s: &[T], start: usize, end: usize) -> &[T] {
    match range_get_checked(s, start, end) {
        Some(v) => v,
        None => panic!("`0 <= start <= end <= len` violated"),
    }
}

/// [`range_get`] but not panicking.
pub const fn range_get_checked<T>(s: &[T], start: usize, end: usize) -> Option<&[T]> {
    if start <= end && end <= s.len() {
        // SAFETY: Checked `0 <= start <= end <= len`
        return Some(unsafe { range_get_unchecked(s, start, end) });
    }
    None
}

/// Get `&s[start..end]` without any checks (see [`range_get`]).
///
/// # Safety
///
/// The caller must guarantee the following: `0 <= start <= end <= len`.
pub const unsafe fn range_get_unchecked<T>(s: &[T], start: usize, end: usize) -> &[T] {
    // SAFETY: caller already promised validity
    unsafe { core::slice::from_raw_parts(s.as_ptr().add(start), end.unchecked_sub(start)) }
}

/// Return `&s[start..=end]` in const.
///
/// To exclude the `end` (`&s[start..end]`), use [`range_get`]. There is no `range_to_get`
/// (`&s[..=end]`). Use this function with a `start = 0` or see [`[]::split_at`].
///
/// # Panics
///
/// If the `start` is strictly larger than `s.len()` or `end` is larger or equal `s.len()` or if
/// `end` is strictly smaller than `start - 1`.
///
/// ```rust,should_panic
/// # use constrs::slice::range_inclusive_get;
/// range_inclusive_get(b"012345", 5, 3); // (start - 1) > end
/// ```
///
/// ```rust,should_panic
/// # use constrs::slice::range_inclusive_get;
/// range_inclusive_get(b"012345", 0, 6); // end >= len
/// ```
///
/// # Examples
///
/// ```rust
/// # use constrs::slice::range_inclusive_get;
/// let v = b"012345";
/// assert_eq!(range_inclusive_get(v, 2, 4), &v[2..=4]); // "234"
/// assert_eq!(range_inclusive_get(v, 0, 5), &v[0..=5]); // "012345": end == len - 1
/// assert_eq!(range_inclusive_get(v, 5, 5), &v[5..=5]); // "5":    start == end
/// assert_eq!(range_inclusive_get(v, 5, 4), &v[5..=4]); // "": start - 1 == end
/// ```
pub const fn range_inclusive_get<T>(s: &[T], start: usize, end: usize) -> &[T] {
    match range_inclusive_get_checked(s, start, end) {
        Some(v) => v,
        None => panic!("`0 <= start <= end + 1 <= len` violated"),
    }
}

/// [`range_inclusive_get`] but not panicking.
pub const fn range_inclusive_get_checked<T>(s: &[T], start: usize, end: usize) -> Option<&[T]> {
    // `end < s.len()` strictly smaller so `end + 1` can happen
    let Some(end) = end.checked_add(1) else {
        // TODO when the version is bumped past 1.95, add `core::hint::cold_path()` here
        //      this basically never happens naturally unless something is very wrong.
        //      indices are naturally below or equal [`isize::MAX`] even for [`u8`] which is half
        //      the [`usize::MAX`] that makes this return [`None`].
        // core::hint::cold_path();
        return None;
    };
    range_get_checked(s, start, end)
}

/// Get `&s[start..=end]` without any checks (see [`range_inclusive_get`]).
///
/// # Safety
///
/// The caller must guarantee the following: `0 <= start <= end + 1 <= len`.
pub const unsafe fn range_inclusive_get_unchecked<T>(s: &[T], start: usize, end: usize) -> &[T] {
    // SAFETY: caller already promised validity
    unsafe { range_get_unchecked(s, start, end.unchecked_add(1)) }
}
