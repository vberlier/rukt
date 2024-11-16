# rukt

[![Test](https://github.com/vberlier/rukt/actions/workflows/test.yml/badge.svg)](https://github.com/vberlier/rukt/actions/workflows/test.yml)
[![License](https://img.shields.io/crates/l/rukt)](https://github.com/vberlier/rukt/blob/main/LICENSE-MIT)
[![Crates.io](https://img.shields.io/crates/v/rukt.svg)](https://crates.io/crates/rukt)
[![Crates.io](https://img.shields.io/crates/d/rukt.svg)](https://crates.io/crates/rukt)
[![Documentation](https://docs.rs/rukt/badge.svg)](https://docs.rs/rukt)

Rust dialect for token-based compile-time scripting.

```rust
use rukt::rukt;

rukt! {
    pub(crate) let operations = {
        add: +,
        sub: -,
        mul: *,
        div: /,
    };
}

rukt! {
    let {$($name:ident: $operator:tt,)*} = operations;
    expand {
        $(
            fn $name(a: u32, b: u32) -> u32 {
                a $operator b
            }
        )*
    }
}
```

# Introduction

Rukt is a subset of Rust where you manipulate tokens instead of values.

It executes entirely at compile-time. It lets you store arbitrary token trees in variables, operate on these token trees using ordinary expressions and control flow, and substitute them anywhere in regular Rust code.

Rukt is designed to be as unsurprising as possible. It ports well-established Rust idioms to the realm of `macro_rules` using polished syntax you're already used to.

This is a lightweight, no-dependency crate, backed entirely by [declarative macros](https://doc.rust-lang.org/reference/macros-by-example.html). There's no procedural macro involved. No unstable features.

## Documentation

- [Statements](https://docs.rs/rukt/latest/rukt/eval/macro.block.html)
- [Expressions](https://docs.rs/rukt/latest/rukt/eval/macro.expression.html)
- [Builtins](https://docs.rs/rukt/latest/rukt/builtins/index.html)
- [Internals](https://docs.rs/rukt/latest/rukt/eval/index.html)

## License

Licensed under [MIT](https://github.com/vberlier/rukt/blob/main/LICENSE-MIT) or [Apache-2.0](https://github.com/vberlier/rukt/blob/main/LICENSE-APACHE).
