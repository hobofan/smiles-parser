## smiles-parser - SMILES parser in Rust based on the OpenSMILES spec

<!-- Crates version -->
<a href="https://crates.io/crates/smiles-parser">
  <img src="https://img.shields.io/crates/v/smiles-parser.svg?style=flat-square"
  alt="Crates.io version" />
</a>
<!-- docs.rs docs -->
<a href="https://docs.rs/smiles-parser">
  <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
    alt="docs.rs docs" />
</a>

## Installation

Via [cargo-edit](https://github.com/killercup/cargo-edit):

```
cargo add smiles-parser
```

## Usage

Parse a chain (top-level object):

```rust
use smiles_parser::chain;

let chain = chain(b"C1CCC2(CC1)CO2");
assert!(chain.is_ok());
```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

