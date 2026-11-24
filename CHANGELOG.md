# `0.1.5`

## Fix

- crate not being compiled on `1.85.0` (before `1.88.0`) which was the claimed
  `rust-version` (`E0658` without the [`let-chains`
  feature](https://github.com/rust-lang/rust/pull/132833)).  A zero `clippy`
  warning on both `1.85.0` and `stable` is now guaranteed.

## Add

- `slice::as_(mut_)?array` which does as its standard counterpart.
- `slice::range_(inclusive_)?get_(un)?checked_mut` functions.
- `slice::fill_copy` function.
- `num::*::min` and `num::*::max` for types implementing `core::cmp::Ord`
  (integers).
- `num::span` function as a peer for `num::prefix_span`.

# `0.1.4`

## Add

- `date` module for pasing WHATWG dates.
- `email` module for parsing WHATWG emails.
- `slice::range_(inclusive_)?_get_(un)?checked` functions.
- `slice::split_at_chunk` function.
- `slice::split_two_chunks` function.
- `num::*::[lb]e_postfix` functions.
- `num::*::[lb]e_at` functions.
- `seq_macro` to testing dependencies (already needed by some
  disabled/documenation only tests).

# `0.1.3`

## Fix

- `slice` functions with `*_mut` postfix not being mutable.

## Add

- enforments and hooks using `prek` to prevent unhappy clippy, `fmt` or other
  `mut` mistakes.

# `0.1.2`

## Add

- `num::*::le_prefix` for getting Little-Endian prefixes of strings.
- `num::*::be_prefix` for getting Big-Endian prefixes of strings.
- `num::i*` and `num::f*` modules with limited functionalities.
- `slice` module with item and range getters.

# `0.1.1`

## Fix

- `usize` decode functions returning `u16`
