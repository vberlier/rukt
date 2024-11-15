//! Implementation of the evaluator.
//!
//! Source code inside [`rukt`](crate::rukt) blocks is evaluated and expanded
//! entirely using declarative macros.
//!
//! The evaluator is a simple [TT
//! muncher](https://veykril.github.io/tlborm/decl-macros/patterns/tt-muncher.html)
//! with a continuation stack that can handle a rich subset of Rust syntax.
//!
//! # Calling convention
//!
//! All macros invoked by the Rukt evaluator follow a unified calling convention
//! that encodes the complete evaluation state.
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt $P:tt $V:tt $N:tt $D:tt) => {
//!     };
//! }
//! ```
//!
//! For brevity, macros invoked by the evaluator use a conventional
//! single-letter metavariable name to bind each specific fragment.
//!
//! The `$D:tt` metavariable at the very end is not part of the state of the
//! interpreter. It's always bound to the dollar-sign token `$`, which can be
//! useful for generating intermediate `macro_rules` definitions.
//!
//! All macros expand to a call to the next continuation, which can be static or
//! popped from the [continuation stack](#next-continuation).
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt $P:tt $V:tt $N:tt $D:tt) => {
//!         continuation!($T $S $P $V $N $);
//!     };
//! }
//! ```
//!
//! ## Remaining tokens
//!
//! The `$T:tt` metavariable matches the source tokens that we still need to
//! evaluate, enclosed in braces `{}`.
//!
//! Macros responsible for parsing the input will consume tokens then pass the
//! remainder to the next continuation unmodified.
//!
//! ```
//! macro_rules! example {
//!     ({ let $L:tt = $($T:tt)* } $S:tt $P:tt $V:tt $N:tt $D:tt) => {
//!         continuation!({ $($T)* } () $P $V $N $);
//!     };
//! }
//! ```
//!
//! ## Current subject
//!
//! The `$S:tt` metavariable matches the token corresponding to the last
//! evaluated expression.
//!
//! It's essentially an accumulator that individual macros are free to consume
//! or ignore depending on the specific context.
//!
//! When a macro decides to discard the current subject it should invoke the
//! next continuation with the unit token `()`.
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt $P:tt $V:tt $N:tt $D:tt) => {
//!         continuation!($T () $P $V $N $);
//!     };
//! }
//! ```
//!
//! # Environment
//!
//! The `$P:tt` and `$V:tt` metavariables represent the current execution
//! environment. The execution environment defines the variables accessible in
//! the current scope and their respective values.
//!
//! The `$P:tt` metavariable matches every variable's corresponding pattern,
//! enclosed in brackets `[]`.
//!
//! The `$V:tt` metavariable matches every variable's associated value, enclosed
//! in brackets `[]`.
//!
//! You can define a new variable in the current scope by pushing a pattern and
//! its matching value when calling the next continuation.
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt [$($P:tt)*] [$($V:tt)*] $N:tt $D:tt) => {
//!         continuation!($T () [$($P)* $message:tt] [$($V)* "hello"] $N $);
//!     };
//! }
//! ```
//!
//! To substitute variables defined in the current scope, you can generate and
//! expand an intermediate macro.
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt $P:tt $V:tt $N:tt $D:tt) => {
//!         macro_rules! __transcribe {
//!             ($P $TT:tt $PP:tt $VV:tt $NN:tt) => {
//!                 continuation!($TT $S $PP $VV $NN $);
//!             };
//!         }
//!         __transcribe!($V $T $P $V $N);
//!     };
//! }
//! ```
//!
//! By pasting the environment patterns into the signature of the generated
//! macro and matching them with the associated environment values, the expanded
//! metavariables will bind all the accessible local variables.
//!
//! Make sure to forward the rest of the evaluation state through intermediate
//! metavariables passed to the generated macro. In this case for example,
//! variable substitution should only occur within the current subject `$S`
//! before passing it to the next continuation.
//!
//! # Next continuation
//!
//! The `$N:tt` metavariable matches the stack of next continuations, enclosed
//! in brackets `[]`.
//!
//! After evaluating part of an expression, most macros will need to pop the
//! next continuation from the stack to dispatch depending on the previous
//! caller.
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt $P:tt $V:tt [($F:path; $($C:tt)*) $($N:tt)*] $D:tt) => {
//!         $F!($T "hello" $($C)* $P $V [$($N)*] $);
//!     };
//! }
//! ```
//!
//! The pattern for destructuring continuations is `($F:path; $($C:tt)*)`, where
//! `$F:path` matches an arbitrary Rust path to a declarative macro that follows
//! the calling convention of the evaluator, and `$($C:tt)*` matches additional
//! context information that was saved when the continuation was pushed.
//!
//! Context information `$C` must be forwarded after the current subject `$S`
//! and before the execution environment patterns `$P`.
//!
//! When expecting a sub-expression as part of a larger construct, pushing a
//! continuation makes it so that the evaluator can call you back once the
//! sub-expressions is evaluated.
//!
//! ```
//! macro_rules! example {
//!     ({ let $L:tt = $($T:tt)* } $S:tt $P:tt $V:tt [$($N:tt)*] $D:tt) => {
//!         expression!({ $($T)* } () $P $V [(let_binding; $L) $($N)*] $);
//!     };
//! }
//! ```
//! ```
//! macro_rules! let_binding {
//!     ({ ; $($T:tt)* } $S:tt $I:ident [$($P:tt)*] [$($V:tt)*] $N:tt $D:tt) => {
//!         block!({ $($T)* } () [$($P)* $D$I:tt] [$($V)* $S] $N $);
//!     };
//! }
//! ```

#[doc(hidden)]
#[macro_export]
macro_rules! eval_block {
    ({} $S:tt $P:tt $V:tt $N:tt $D:tt) => {
    };
    ({ let $L:tt = $($T:tt)* } $S:tt $P:tt $V:tt [$($N:tt)*] $D:tt) => {
        $crate::eval::expression!({ $($T)* } () $P $V [($crate::eval_let_binding; $L) $($N)*] $);
    };
    ({ expand { $($B:tt)* } $($T:tt)* } $S:tt $P:tt $V:tt $N:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P) => {
                $($B)*
            };
        }
        __rukt_transcribe!($V);
        $crate::eval::block!({ $($T)* } () $P $V $N $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_let_binding {
    ({ ; $($T:tt)* } $S:tt _ $P:tt $V:tt $N:tt $D:tt) => {
        $crate::eval::block!({ $($T)* } () $P $V $N $);
    };
    ({ ; $($T:tt)* } $S:tt $I:ident [$($P:tt)*] [$($V:tt)*] $N:tt $D:tt) => {
        $crate::eval::block!({ $($T)* } () [$($P)* $D$I:tt] [$($V)* $S] $N $);
    };
    ({ ; $($T:tt)* } $S:tt $L:tt [$($P:tt)*] [$($V:tt)*] $N:tt $D:tt) => {
        $crate::eval::block!({ $($T)* } () [$($P)* $L] [$($V)* $S] $N $);
    };
}

/// Evaluate statements within blocks.
///
/// Rukt blocks can contain the following statements:
///
/// - [Let bindings](#let-bindings)
/// - [Expand statements](#expand-statements)
///
/// # Let bindings
///
/// They mirror Rust's own `let` bindings. They allow you bind the result of an
/// [`expression`] to a variable.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let message = "hello";
/// }
/// ```
///
/// Note that unlike in Rust, you can't implicitly shadow a previous variable
/// with the same name.
///
/// ```compile_fail
/// # use rukt::rukt;
/// rukt! {
///     let message = "hello";
///     let message = "world"; // error: duplicate matcher binding
///     let _ = message;
/// }
/// ```
///
/// There's also no `let mut`, all variables are immutable.
///
/// Using an underscore `_` as the variable name will explicitly discard the
/// result of the expression.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let _ = "hello";
/// }
/// ```
///
/// The left side can also be a pattern for destructuring the value specified as
/// a delimiter-enclosed
/// [`macro_rules`](https://doc.rust-lang.org/reference/macros-by-example.html)
/// matchers. This is particularly useful for binding [repeated
/// fragments](https://veykril.github.io/tlborm/decl-macros/macros-methodical.html#repetitions).
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let {$($name:ident: $operator:tt,)*} = {
///         add: +,
///         sub: -,
///         mul: *,
///         div: /,
///     };
/// }
/// ```
///
/// Note that depending on the fragment specifier you might not be able to
/// inspect the tokens further. You can usually stick to `tt` and `ident`. See
/// [forwarding a matched
/// fragment](https://doc.rust-lang.org/stable/reference/macros-by-example.html#forwarding-a-matched-fragment).
///
/// # Expand statements
///
/// The `expand` statement will substitute all variables accessible in the
/// current scope in the given code block, and paste the resulting Rust code as
/// part of the expansion of the [`rukt`](crate::rukt) macro. The expansion
/// doesn't include the braces `{}` used to delimit the code block.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let message = "Expanded from Rukt!";
///     expand {
///         fn example() -> &'static str {
///             $message
///         }
///     }
/// }
/// # assert_eq!(example(), "Expanded from Rukt!");
/// ```
///
/// Variable substitutions in the code block rely on the standard `$variable`
/// syntax handled by
/// [`macro_rules`](https://doc.rust-lang.org/reference/macros-by-example.html#metavariables).
#[doc(inline)]
pub use eval_block as block;

#[doc(hidden)]
#[macro_export]
macro_rules! eval_expression {
    ({ true $($T:tt)* } $S:tt $P:tt $V:tt [($F:path; $($C:tt)*) $($N:tt)*] $D:tt) => {
        $F!({ $($T)* } true $($C)* $P $V [$($N)*] $);
    };
    ({ false $($T:tt)* } $S:tt $P:tt $V:tt [($F:path; $($C:tt)*) $($N:tt)*] $D:tt) => {
        $F!({ $($T)* } false $($C)* $P $V [$($N)*] $);
    };
    ({ $I:ident $($T:tt)* } $S:tt $P:tt $V:tt $N:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P $TT:tt $PP:tt $VV:tt $NN:tt) => {
                $crate::eval_identifier!($TT [$D$I] $PP $VV $NN $);
            };
        }
        __rukt_transcribe!($V { $($T)* } $P $V $N);
    };
    ({ ($($R:tt)*) $($T:tt)* } $S:tt $P:tt $V:tt $N:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P $TT:tt $PP:tt $VV:tt [($FF:path; $D($CC:tt)*) $D($NN:tt)*]) => {
                $FF!($TT ($($R)*) $D($CC)* $PP $VV [$D($NN)*] $);
            };
        }
        __rukt_transcribe!($V { $($T)* } $P $V $N);
    };
    ({ [$($R:tt)*] $($T:tt)* } $S:tt $P:tt $V:tt $N:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P $TT:tt $PP:tt $VV:tt [($FF:path; $D($CC:tt)*) $D($NN:tt)*]) => {
                $FF!($TT [$($R)*] $D($CC)* $PP $VV [$D($NN)*] $);
            };
        }
        __rukt_transcribe!($V { $($T)* } $P $V $N);
    };
    ({ {$($R:tt)*} $($T:tt)* } $S:tt $P:tt $V:tt $N:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P $TT:tt $PP:tt $VV:tt [($FF:path; $D($CC:tt)*) $D($NN:tt)*]) => {
                $FF!($TT {$($R)*} $D($CC)* $PP $VV [$D($NN)*] $);
            };
        }
        __rukt_transcribe!($V { $($T)* } $P $V $N);
    };
    ({ $R:tt $($T:tt)* } $S:tt $P:tt $V:tt [($F:path; $($C:tt)*) $($N:tt)*] $D:tt) => {
        $F!({ $($T)* } $R $($C)* $P $V [$($N)*] $);
    };
}

/// Evaluate expression.
///
/// Rukt expressions can be one of the following:
///
/// - [Literals](#literals)
/// - [Variables](#variables)
/// - [Builtins](crate::builtins)
///
/// # Literals
///
/// With the exception of identifiers, every Rust token is a literal when used
/// as part of an expression in Rukt.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let number = 42;
///     let string = "hello";
///     let boolean = true;
///     let operator = +;
///     let separator = ::;
///     let punctuation = .;
/// }
/// ```
///
/// Note that this includes `true` and `false` which are normally tokenized as
/// identifiers by Rust.
///
/// Literals can also be entire token trees enclosed in parenthesis `()`,
/// brackets `[]`, or braces `{}`. Variables accessible in the current scope are
/// expanded inside the token tree.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let taco = {
///         chat => bouc,
///         cheese => pizza,
///     };
///     let arbitrary = {
///         [1 2 3]
///         $taco
///     };
///     expand {
///         assert_eq!(stringify!($arbitrary), "{ [1 2 3] { chat => bouc, cheese => pizza, } }");
///     }
/// }
/// ```
///
/// Just like in regular
/// [`macro_rules`](https://doc.rust-lang.org/reference/macros-by-example.html),
/// token trees can contain pretty much arbitrary syntax.
///
/// Variable substitutions in delimiter-enclosed token tree literals rely on the
/// standard `$variable` syntax handled by
/// [`macro_rules`](https://doc.rust-lang.org/reference/macros-by-example.html#metavariables).
///
/// # Variables
///
/// Identifiers inside Rukt expressions refer to previously defined variables.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let value = 123;
///     let number = value;
/// }
/// ```
///
/// If the identifier doesn't match any variable accessible in the current
/// scope, the evaluator will try to fall back to any available
/// [`builtins`](crate::builtins) before failing to compile.
///
/// ```compile_fail
/// # use rukt::rukt;
/// rukt! {
///     let number = value; // error: cannot find macro `value` in this scope
/// }
/// ```
///
/// If you want to store an identifier token in a variable you'll have to
/// extract it from a token tree, for example using `let` destructuring.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let ($name:ident) = (VALUE);
///     expand {
///         const $name: u32 = 123;
///     }
/// }
/// assert_eq!(VALUE, 123);
/// ```
#[doc(inline)]
pub use eval_expression as expression;

#[doc(hidden)]
#[macro_export]
macro_rules! eval_identifier {
    ($T:tt [$S:tt] $P:tt $V:tt [($F:path; $($C:tt)*) $($N:tt)*] $D:tt) => {
        $F!($T $S $($C)* $P $V [$($N)*] $);
    };
    ($T:tt [$_:tt $S:tt] $P:tt $V:tt $N:tt $D:tt) => {
        $crate::eval_builtin!($T [$S] $P $V $N $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_builtin {
    ({ ::$I:ident $($T:tt)* } [$($S:tt)+] $P:tt $V:tt $N:tt $D:tt) => {
        $crate::eval_builtin!({ $($T)* } [$($S)*::$I] $P $V $N $);
    };
    ($T:tt [$($S:tt)+] $P:tt $V:tt $N:tt $D:tt) => {
        $($S)*!($T () $P $V $N $);
    };
}
