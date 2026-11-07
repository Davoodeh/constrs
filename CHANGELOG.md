# `0.1.4`

## Add

- `date` module for pasing WHATWG dates.
- `email` module for parsing WHATWG emails.
- `slice::range_(inclusive_)?_get_(un)?checked` functions.
- `slice::split_at_chunk` function.
- `num::*::[lb]e_postfix` functions.
- `num::*::[lb]e_at` functions.

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
