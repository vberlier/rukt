//! The primary extension mechanism.
//!
//! Rukt builtins are a specific kind of declarative macros that can participate
//! in the evaluation of [`rukt`](crate::rukt) blocks.
//!
//! Rukt expressions can refer to builtins by name using simple paths comprised
//! of identifiers joined by `::`.
//!
//! This module provides some common utilities but you can create your own
//! builtins anywhere.
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
//!         $F!($T "hello world" $($C)* $P $V $);
//!     };
//! }
//! ```
//!
//! This is an example of a simple builtin that resolves to the token `"hello
//! world"`. The macro follows the [calling
//! convention](crate::eval#calling-convention) of the Rukt evaluator. As long
//! as the macro is accessible from the surrounding Rust code, the evaluator
//! will know how to invoke it during the expansion of [`rukt`](crate::rukt)
//! blocks.
//!
//! ```
//! # macro_rules! example {
//! #     ($T:tt $S:tt ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
//! #         $F!($T "hello world" $($C)* $P $V $);
//! #     };
//! # }
//! # use rukt::rukt;
//! rukt! {
//!     let message = example;
//!     expand {
//!         assert_eq!($message, "hello world");
//!     }
//! }
//! ```

#[doc(hidden)]
#[macro_export]
macro_rules! builtin_breakpoint {
    ($T:tt $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        compile_error!(concat!(
            "rukt: breakpoint\n",
            "tokens = ",
            stringify!($T),
            "\n",
            "subject = ",
            stringify!($S),
            "\n",
            "next = ",
            stringify!($N),
            "\n",
            "patterns = ",
            stringify!($P),
            "\n",
            "values = ",
            stringify!($V),
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
///        next = ($crate :: eval_let_binding; value /)
///        patterns = [$ message : tt]
///        values = ["hello"]
/// ```
#[doc(inline)]
pub use builtin_breakpoint as breakpoint;

#[doc(hidden)]
#[macro_export]
macro_rules! builtin_parse {
    ({ ::<$F:tt>($($R:tt)*) $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        macro_rules! __rukt_parse {
            ($TT:tt ($FF:path; $D($CC:tt)*) $PP:tt $VV:tt $SS:$F) => {
                $FF!($TT $SS $D($CC)* $PP $VV $);
            };
        }
        macro_rules! __rukt_transcribe {
            ($P $TT:tt $NN:tt $PP:tt $VV:tt) => {
                __rukt_parse!($TT $NN $PP $VV $($R)*);
            };
        }
        __rukt_transcribe!($V { $($T)* } $N $P $V);
    };
}

/// Parse tokens into a specific syntax fragment according to the given
/// specifier.
///
/// ```
/// # use rukt::rukt;
/// use rukt::builtins::parse;
/// rukt! {
///     let result = parse::<expr>(1 + 2 + 3);
/// }
/// ```
///
/// This is equivalent to destructuring a token tree using the same fragment
/// specifier.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let ($result:expr) = (1 + 2 + 3);
/// }
/// ```
///
/// Using the [`parse`] builtin can be more convenient if you're in the middle
/// of an expression or if you want to directly export a valid piece of Rust
/// syntax.
///
/// ```
/// # use rukt::rukt;
/// use rukt::builtins::parse;
/// rukt! {
///     pub(crate) let define_struct = parse::<item>(
///         struct MyStruct {
///             value: u32,
///         }
///     );
/// }
/// define_struct!();
/// assert_eq!(MyStruct { value: 42 }.value, 42);
/// ```
///
/// Another use case which can be covered by either [`parse`] or destructuring
/// is calling regular macros. Note that in most cases since the returned token
/// will be opaque you won't be able to inspect it further, but it can still be
/// useful.
///
/// ```
/// # use rukt::rukt;
/// use rukt::builtins::parse;
/// rukt! {
///     let a = "hello";
///     let b = "world";
///     pub(crate) let message = parse::<expr>(
///         concat!($a, " ", $b)
///     );
/// }
/// assert_eq!(message!(), "hello world");
/// ```
#[doc(inline)]
pub use builtin_parse as parse;
