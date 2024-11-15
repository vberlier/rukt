//! Macros usable as builtins inside [`rukt`](crate::rukt) blocks.
//!
//! Rukt builtins are declarative macros which follow the [calling
//! convention](crate::eval#calling-convention) of the evaluator.
//!
//! You can write your own builtins. As long as the macros are accessible from
//! normal Rust code, the evaluator will know how to invoke them within Rukt.
//!
//! Rukt expressions can refer to builtins using simple paths comprised of
//! identifiers joined by `::`.

#[doc(hidden)]
#[macro_export]
macro_rules! builtin_breakpoint {
    ($T:tt $S:tt $P:tt $V:tt $N:tt $D:tt) => {
        compile_error!(concat!(
            "rukt: breakpoint\n",
            "tokens = ",
            stringify!($T),
            "\n",
            "subject = ",
            stringify!($S),
            "\n",
            "patterns = ",
            stringify!($P),
            "\n",
            "values = ",
            stringify!($V),
            "\n",
            "next = ",
            stringify!($N),
        ));
    };
}

/// Dump evaluation state using [`compile_error`].
///
/// Each field shows the corresponding fragment matched by the evaluator's
/// [calling convention](crate::eval#calling-convention).
///
/// ```compile_fail
/// # use rukt::rukt;
/// rukt! {
///     let message = "hello";
///     let value = rukt::builtins::breakpoint;
///     expand {
///         println!($message);
///     }
/// }
/// ```
/// ```text
/// error: rukt: breakpoint
///        tokens = { ; expand { println!($message); } }
///        subject = ()
///        patterns = [$ message : tt]
///        values = ["hello"]
///        next = [($crate :: eval_let_binding; value)]
/// ```
#[doc(inline)]
pub use builtin_breakpoint as breakpoint;
