# Constrs

A small collection of low-level, no alloc, `const`-time one-pass parsers,
encoders and formatters.

Most of the utilities in this crate are implemented better elsewhere (e.g.
`itoa`) or are trivial parts of the standard libraries. The only reason for the
existence of this crate is the niche of `const`-time usage.

This crate grew way larger than intented but the general goal almost always
remained the same:

- remain `const`-time (functions are `const`)
- do not require heap allocation
- do not require Rust's standard library (`no_std`)
- have fast parsers that fail fast and do not "look-ahead" if possible
- be verbose, explicit and have minimal dependencies
- do not use specialized types and Rust shenanigans when possible (to ease FFIs)
- expose fine-grained and specialized functions

If you do not have peformance critical needs, can afford the `std` or
computations not in `const` or any combination of the previous criteria, do not
use this library and prefer other mature implementations like Rust's standard
library. Hopefully, once the standard library has more stable `const`
functions and the trait resolver supports `const` traits, this crate can be
removed or at least greatly simplified.

Currently, there are a collection of different (but related parsers) including
number parsers, UTF-8 parsers, Percent-Encoding, URL, IP and other parsers which
may or may not be converted to separate crates.

NOTE this is a work-in-progress crate and its layout may change (often). It's
not certain what will remain/be added in the scope of the project and what will
be evicted.

## References

This crate aims to be compatible with the following WHATWG notes: 

- WHATWG URL:
  <https://url.spec.whatwg.org/commit-snapshots/8e14777cfa145b08a9fb735fe580ec0c366564c3/>
- WHATWG Encoding:
  <https://encoding.spec.whatwg.org/commit-snapshots/a985b62a9b45c17da3e17a9f0a0b4e30c34c4a8a/>
- WHATWG HTML:
  <https://html.spec.whatwg.org/commit-snapshots/cd8ac6f1bbf86dd0bd09ef75d27dacaebe7b4c1d/>
- WHATWG Infra:
  <https://infra.spec.whatwg.org/commit-snapshots/3f984adcd24a6d5c53cc26b3e737701808003f3e/>
  
As for the time of writing this, WHATWG notes comply with Unicode standards and
use the same implementations used other tools like the Rust's standard library.
Meaning the implementations are also Unicode compatible even though the
implementation is done through WHATWG notes.
  
## Versioning

Before `1.0.0`, every minor version (`0.THIS.0`) bump *may* be a breaking change
for the dependent code using this library.

See the [`CHANGELOG`](./CHANGELOG.md) file in the source for the news about each
release.

## License

The license of this crate and its components is exactly as described in the
`Cargo.toml`, [`MIT`] OR [`Apache-2.0`]. WHATWG documents mentioned in the
references have their own open licenses.

For the dependencies, consult their licenses but this crate aims to keep it at
[`MIT`] OR [`Apache-2.0`] dual license for the sake of simplicity.

[`MIT`]: https://opensource.org/license/MIT
[`Apache-2.0`]: https://opensource.org/license/apache-2-0
