# rukt

Simple Rust dialect for token-based compile-time scripting.

```rust
use rukt::rukt;

rukt!{
    let operations = {
        add: +,
        sub: -,
        mul: *,
        div: /,
    };
    let {$($name:ident: $operator:tt,)*} = operations;
    expand {
        $(
            fn $name(a: u32, b: u32) {
                a $operator b
            }
        )*
    }
}
```

## License

Licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
