//! Holds helpers for a more unconditionally `const` slice operations.

// NOTE macro exports are the last line of the module
//      for internal use and before the macros are public, use them instead

/// Get a slice element with a [`usize`] index, uncoditionally const (same as `&s[index]`).
///
/// This aims to be a replacement for the [slice primitive `get` method] until it's const
/// stable.
///
/// # Examples
///
/// ```rust
/// # use constrs::slice::get;
/// let s = b"1234";
/// assert_eq!(s.get(0), get(s, 0));
/// assert_eq!(s.get(1), get(s, 1));
/// assert_eq!(s.get(5), get(s, 5));
/// assert_eq!(s.get(usize::MAX), get(s, usize::MAX));
/// ```
///
/// [slice primitive `get` method]:
/// https://doc.rust-lang.org/core/primitive.slice.html#method.get
pub const fn get<T>(s: &[T], index: usize) -> Option<&T> {
    if index < s.len() {
        return Some(&s[index]);
    }
    None
}

/// [`get`] but mutable (same as `&mut s[index]`).
pub const fn get_mut<T>(s: &mut [T], index: usize) -> Option<&mut T> {
    if index < s.len() {
        return Some(&mut s[index]);
    }
    None
}

/// Get the first slice element, uncoditionally const (same as `&s[0]`).
pub const fn first<T>(s: &[T]) -> Option<&T> {
    get(s, 0)
}

/// [`first`] but mutable (same as `&mut s[0]`).
pub const fn first_mut<T>(s: &mut [T]) -> Option<&mut T> {
    get_mut(s, 0)
}

/// Get the last slice element, uncoditionally const (same as `&s[s.len() - 1]`).
pub const fn last<T>(s: &[T]) -> Option<&T> {
    if s.is_empty() {
        return None;
    }
    Some(&s[s.len() - 1])
}

/// [`last`] but mutable (same as `&mut s[s.len() - 1]`).
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

/// [`range_get`] but mutable (same as `&mut s[start..end]`).
pub const fn range_get_mut<T>(s: &mut [T], start: usize, end: usize) -> &mut [T] {
    match range_get_checked_mut(s, start, end) {
        Some(v) => v,
        None => panic!("`0 <= start <= end <= len` violated"),
    }
}

/// [`range_get`] but not panicking (same as `&s[start..end]`).
pub const fn range_get_checked<T>(s: &[T], start: usize, end: usize) -> Option<&[T]> {
    if start <= end && end <= s.len() {
        // SAFETY: Checked `0 <= start <= end <= len`
        return Some(unsafe { range_get_unchecked(s, start, end) });
    }
    None
}

/// [`range_get_checked`] but mutable (same as non-panicking `&mut s[start..end]`).
pub const fn range_get_checked_mut<T>(s: &mut [T], start: usize, end: usize) -> Option<&mut [T]> {
    if start <= end && end <= s.len() {
        // SAFETY: Checked `0 <= start <= end <= len`
        return Some(unsafe { range_get_unchecked_mut(s, start, end) });
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

/// Get `&mut s[start..end]` without any checks (see [`range_get_mut`]).
///
/// # Safety
///
/// The caller must guarantee the following: `0 <= start <= end <= len`.
pub const unsafe fn range_get_unchecked_mut<T>(s: &mut [T], start: usize, end: usize) -> &mut [T] {
    // SAFETY: caller already promised validity
    unsafe { core::slice::from_raw_parts_mut(s.as_mut_ptr().add(start), end.unchecked_sub(start)) }
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

/// [`range_inclusive_get`] but mutable (same as `&mut s[start..=end]`).
pub const fn range_inclusive_get_mut<T>(s: &mut [T], start: usize, end: usize) -> &mut [T] {
    match range_inclusive_get_checked_mut(s, start, end) {
        Some(v) => v,
        None => panic!("`0 <= start <= end + 1 <= len` violated"),
    }
}

/// [`range_inclusive_get`] but not panicking (same as `&mut s[start..=end]`).
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

/// [`range_inclusive_get`] but mutable (same as non-panicking `&mut s[start..=end]`).
pub const fn range_inclusive_get_checked_mut<T>(
    s: &mut [T],
    start: usize,
    end: usize,
) -> Option<&mut [T]> {
    // `end < s.len()` strictly smaller so `end + 1` can happen
    let Some(end) = end.checked_add(1) else {
        // TODO when the version is bumped past 1.95, add `core::hint::cold_path()` here
        //      this basically never happens naturally unless something is very wrong.
        //      indices are naturally below or equal [`isize::MAX`] even for [`u8`] which is half
        //      the [`usize::MAX`] that makes this return [`None`].
        // core::hint::cold_path();
        return None;
    };
    range_get_checked_mut(s, start, end)
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

/// Get `&s[start..=end]` without any checks (see [`range_inclusive_get`]).
///
/// # Safety
///
/// The caller must guarantee the following: `0 <= start <= end + 1 <= len`.
pub const unsafe fn range_inclusive_get_unchecked_mut<T>(
    s: &mut [T],
    start: usize,
    end: usize,
) -> &mut [T] {
    // SAFETY: caller already promised validity
    unsafe { range_get_unchecked_mut(s, start, end.unchecked_add(1)) }
}

/// Split a chunk at the given index.
///
/// If the given index is too late in the given array that taking a chunk is impossible returns
/// [`None`].
///
/// Alternatively, if the given index is out of bound, it returns [`None`].
///
/// # Examples
///
/// ```rust
/// # use constrs::slice::split_at_chunk;
/// assert_eq!(split_at_chunk::<_, 0>(b"012345", 2), Some(&[]));
/// assert_eq!(split_at_chunk::<_, 1>(b"012345", 0), Some(&[b'0']));
/// assert_eq!(split_at_chunk::<_, 3>(b"012345", 2), Some(&[b'2', b'3', b'4']));
/// assert_eq!(split_at_chunk::<_, 4>(b"012345", 2), Some(&[b'2', b'3', b'4', b'5']));
/// assert_eq!(split_at_chunk::<_, 5>(b"012345", 2), None); // cannot get 5 elements (no b'6')
/// assert_eq!(split_at_chunk::<_, 1>(b"012345", 5), Some(&[b'5']));
/// assert_eq!(split_at_chunk::<_, 1>(b"012345", 6), None); // invalid index
/// ```
pub const fn split_at_chunk<T, const N: usize>(s: &[T], index: usize) -> Option<&[T; N]> {
    // HACK(E0658, 1.88.0<): ifs can merge with `&&`
    if let Some(leftover) = s.len().checked_sub(index) {
        if N <= leftover {
            // SAFETY: checked for pointer validity, lifetime and count of elements.
            return Some(unsafe { &*(s.as_ptr().add(index) as *const _) });
        }
    }
    None
}

/// Split a slice into two array references.
///
/// # Examples
///
/// ```rust
/// # use constrs::slice::split_two_chunks;
/// assert_eq!(split_two_chunks::<_, 4, 0>(b"0123"), Some((&[b'0', b'1', b'2', b'3'], &[])));
/// assert_eq!(split_two_chunks::<_, 3, 1>(b"0123"), Some((&[b'0', b'1', b'2'], &[ b'3'])));
/// assert_eq!(split_two_chunks::<_, 3, 2>(b"0123"), None); // invalid split sizes
/// ```
pub const fn split_two_chunks<T, const N1: usize, const N2: usize>(
    s: &[T],
) -> Option<(&[T; N1], &[T; N2])> {
    let n = const {
        // compile time check
        match N1.checked_add(N2) {
            Some(v) => v,
            None => panic!("chunks are too large"),
        }
    };
    if s.len() == n {
        // SAFETY: checked for pointer validity, lifetime and count of elements.
        return Some(unsafe {
            (
                &*(s.as_ptr() as *const _),
                &*(s.as_ptr().add(N1) as *const _),
            )
        });
    }
    None
}

/// A copy of the standard `slice::as_array` (prefer the standardized, only in `1.93.0` and higher).
pub const fn as_array<T, const N: usize>(s: &[T]) -> Option<&[T; N]> {
    if s.len() == N {
        // SAFETY: cast is valid only if the lengths match
        return Some(unsafe { &*(s.as_ptr() as *const _) });
    }
    None
}

/// A copy of the `slice::as_mut_array` (prefer the standardized, only in `1.93.0` and higher).
pub const fn as_mut_array<T, const N: usize>(s: &mut [T]) -> Option<&mut [T; N]> {
    if s.len() == N {
        // SAFETY: cast is valid only if the lengths match
        return Some(unsafe { &mut *(s.as_mut_ptr() as *mut _) });
    }
    None
}

/// Copy a single item in all the element of a slice.
///
/// To fill with a value of [`Clone`], use the standard `slice::fill`.
#[doc(alias("copy_from_item"))]
pub const fn fill_copy<T: Copy>(s: &mut [T], v: T) {
    // TODO with different feature flags, optimize this
    let mut i = 0;
    while i < s.len() {
        s[i] = v;
        i += 1;
    }
}

/// Return the number of items for which the virtual closure over each item returns true.
///
/// See [`prefix_matching_len`] for example which is an alias for `prefix_matching_len($s, |i| i ==
/// key)` (count the number of items all the while the given key is not seen).
///
/// [`prefix_matching_len`]: crate::prefix_matching_len
///
/// This is roughly equal to something like the following without the traits and for const context:
///
// NOTE not public so the test must be ignored
/// ```rust,ignore
/// # use std::ops::Index;
/// # use constrs::prefix_matching_len;
/// /// An alternative to `Iterator::iter(x).find(|v| v == key)`.
/// type Idx = /* {integer} and if signed {integer}, only 0 and upwards are checked. */
/// # usize;
/// # trait HasLen { fn len(&self) -> usize; }
/// # impl<T> HasLen for [T] { fn len(&self) -> usize { <[T]>::len(self) } }
/// trait Prefix: Index<Idx> + HasLen
/// where
///     <Self as Index<Idx>>::Output: PartialEq + Sized
/// {
///     /// Increase the index while `f` is true (return the first index of first not matching).
///     ///
///     /// If `self.len()` is the same as the results, implies all the values are exactly the same
///     /// as `key`.
///     fn prefix_matching_len(&self, f: impl Fn(&<Self as Index<Idx>>::Output) -> bool) -> usize {
///         let mut i = 0;
///         while i < self.len() {
///             if !f(&self[i]) {
///                 return i;
///             }
///             i += 1;
///         }
///         i
///     }
/// }
///
/// // trivial implementation for everything can be managed to be implemented
/// impl<T: Index<Idx> + HasLen + ?Sized> Prefix for T
/// where
///     <T as Index<Idx>>::Output: PartialEq + Sized
/// {}
///
/// let s = &[1, 1, 2, 2, 3, 3];
///
/// // the number of elements at the start matching `3` (the starting items are not `3`)
/// assert_eq!(s.prefix_matching_len(|v| *v == 3), 0);
/// assert_eq!(s.prefix_matching_len(|v| *v == 3), prefix_matching_len!(s, |v| *v == 3));
/// // the number of elements at the start matching `1` (the start items are two `1`s)
/// assert_eq!(s.prefix_matching_len(|v| *v < 3), 4);
/// assert_eq!(s.prefix_matching_len(|v| *v < 3), prefix_matching_len!(s, |v| *v < 3));
/// ```
// TODO make the `Prefix` traits from the [`prefix_matching_len`] examples once `HasLen` is managed
// TODO move to its own macro crate alongside "index" macro to have them exported from here and
//      only here. This is to prevent an unecessary move.
macro_rules! prefix_matching_len {
    // TODO support |&&&&&...v| syntax
    ($s:expr, |$v:ident $(: &$ty:ty)?| $expr:expr $(,)?) => {
        $crate::slice::prefix_matching_len!($s, |$v $(: $ty)?| -> bool { $expr })
    };

    ($s:expr, |($i:tt, $v:ident) $(: ($ti:tt, &$ty:ty))?| $expr:expr $(,)?) => {
        $crate::slice::prefix_matching_len!($s, |($i, $v) $(: ($ti, $ty))?| -> bool { $expr })
    };

    ($s:expr, |$v:ident $(: &$ty:ty)?| $(-> bool)? { $expr:expr } $(,)?) => {{
        $crate::slice::prefix_matching_len!($s, |(_, $v) $(: (_, $ty))?| -> bool { $expr })
    }};

    ($s:expr, |($i:tt, $v:ident) $(: ($ti:tt, &$ty:ty))?| $(-> bool)? { $expr:expr } $(,)?) => {{
        let s = $s;
        // have the index be mutable and an unlikely identifier
        let mut ___zpqlxmi123456789___ $(: $ti)? = 0;
        while ___zpqlxmi123456789___ < s.len() {
            // pass the index as immutable to a variable accessable by the expr to prevent
            // modification
            let $i = ___zpqlxmi123456789___;
            let $v $(: &$ty)? = &s[___zpqlxmi123456789___];
            if !($expr) {
                break;
            }
            ___zpqlxmi123456789___ += 1;
        }
        ___zpqlxmi123456789___
    }};
}

/// Indexing helper just like `&s[I]`.
///
/// The start is limited to a single `TokenTree` which roughly means that the start of any range
/// must be a single token-like (`a` or `1` not `a.len()`) unless wrapped in braces or parenthesis.
///
/// Holds two types of indexing:
/// - `s[I]` like `s[1]`
/// - `s[RANGE]` like `s[1..]`
///
/// Implements the ranges below:
/// - `..` (RangeFull)
/// - `a..` (RangeFrom)
/// - `..b` (RangeTo)
/// - `a..b` (Range)
/// - `a..=b` (RangeInclusive)
/// - `..=b` (RangeToInclusive)
macro_rules! i {
    ($s:tt[..]) => {
        &$s
    };

    ($s:tt[$i:tt]) => {{
        #[allow(unused_parens, unused_braces)]
        {
            &$s[$i]
        }
    }};

    ($s:tt[$start:tt..]) => {{
        let end = $s.len();
        $crate::slice::i!($s[{ $start }..{ end }])
    }};

    ($s:tt[..$($end:tt)+]) => {
        $crate::slice::i!($s[0..{ $($end)+ }])
    };

    ($s:tt[..=$($end:tt)+]) => {
        $crate::slice::i!($s[0..={ $($end)+ }])
    };

    ($s:tt[$start:tt..$($end:tt)+]) => {{
        #[allow(unused_parens, unused_braces)]
        {
            let start = $start;
            let end = $($end)+;
            $crate::slice::range_get($s, start, end)
        }
    }};

    ($s:tt[$start:tt..=$($end:tt)+]) => {{
        {
            let start = $start;
            let end = $($end)+;
            $crate::slice::range_inclusive_get($s, start, end)
        }
    }};

    (mut $s:tt[..]) => {
        &mut $s
    };

    (mut $s:tt[$i:tt]) => {{
        #[allow(unused_parens, unused_braces)]
        {
            &mut $s[$i]
        }
    }};

    (mut $s:tt[$start:tt..]) => {{
        let end = $s.len();
        $crate::slice::i!(mut $s[{ $start }..{ end }])
    }};

    (mut $s:tt[..$($end:tt)+]) => {
        $crate::slice::i!(mut $s[0..{ $($end)+ }])
    };

    (mut $s:tt[..=$($end:tt)+]) => {
        $crate::slice::i!(mut $s[0..={ $($end)+ }])
    };

    (mut $s:tt[$start:tt..$($end:tt)+]) => {{
        #[allow(unused_parens, unused_braces)]
        {
            let start = $start;
            let end = $($end)+;
            $crate::slice::range_get_mut($s, start, end)
        }
    }};

    (mut $s:tt[$start:tt..=$($end:tt)+]) => {{
        #[allow(unused_parens, unused_braces)]
        {
            let start = $start;
            let end = $($end)+;
            $crate::slice::range_inclusive_get_mut($s, start, end)
        }
    }};
}

pub(crate) use {i, prefix_matching_len};
