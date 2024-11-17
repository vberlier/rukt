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
//!     ($T:tt $S:tt $N:tt $P:tt $V:tt $D:tt) => {
//!     };
//! }
//! ```
//!
//! For brevity, macros invoked by the evaluator use a conventional
//! single-letter metavariable name to bind each specific fragment.
//!
//! The `$D:tt` metavariable at the very end is not part of the evaluation
//! state. It's always bound to the dollar-sign token `$`, which can be useful
//! for generating intermediate `macro_rules` definitions.
//!
//! All macros expand to a call to a continuation. This can be a predetermined
//! continuation or the [next dynamic continuation](#next-continuation).
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt $N:tt $P:tt $V:tt $D:tt) => {
//!         continuation!($T $S $N $P $V $);
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
//!     ({ let $L:tt = $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
//!         continuation!({ $($T)* } () $N $P $V $);
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
//!     ($T:tt $S:tt $N:tt $P:tt $V:tt $D:tt) => {
//!         continuation!($T () $N $P $V $);
//!     };
//! }
//! ```
//!
//! # Next continuation
//!
//! The `$N:tt` metavariable matches the next dynamic continuation.
//!
//! After evaluating part of an expression, most macros will need to invoke the
//! next continuation to dispatch depending on the previous caller.
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
//!         $F!($T "hello" $($C)* $P $V $);
//!     };
//! }
//! ```
//!
//! The pattern for destructuring the continuation is `($F:path; $($C:tt)*)`,
//! where `$F:path` matches an arbitrary Rust path to a declarative macro that
//! follows the calling convention of the evaluator, and `$($C:tt)*` matches
//! additional context information that was saved when the continuation was
//! pushed.
//!
//! The context information `$C` includes the previous continuation. As such, it
//! must be forwarded after the current subject `$S` and before the execution
//! environment patterns `$P`, which is where the next macro will be expecting
//! to receive its next continuation.
//!
//! When expecting a sub-expression as part of a larger construct, pushing a
//! continuation makes it so that the evaluator can call you back once the
//! sub-expressions is evaluated.
//!
//! ```
//! macro_rules! example {
//!     ({ let $L:tt = $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
//!         expression!({ $($T)* } () (let_binding; $L $N) $P $V $);
//!     };
//! }
//! ```
//! ```
//! macro_rules! let_binding {
//!     ({ ; $($T:tt)* } $S:tt $I:ident $N:tt [$($P:tt)*] [$($V:tt)*] $D:tt) => {
//!         block!({ $($T)* } () $N [$($P)* $D$I:tt] [$($V)* $S] $);
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
//!     ($T:tt $S:tt $N:tt [$($P:tt)*] [$($V:tt)*] $D:tt) => {
//!         continuation!($T () $N [$($P)* $message:tt] [$($V)* "hello"] $);
//!     };
//! }
//! ```
//!
//! To substitute variables defined in the current scope, you can generate and
//! expand an intermediate macro.
//!
//! ```
//! macro_rules! example {
//!     ($T:tt $S:tt $N:tt$P:tt $V:tt  $D:tt) => {
//!         macro_rules! __transcribe {
//!             ($P $TT:tt $NN:tt $PP:tt $VV:tt) => {
//!                 continuation!($TT $S $NN $PP $VV $);
//!             };
//!         }
//!         __transcribe!($V $T $N $P $V);
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

#[doc(hidden)]
#[macro_export]
macro_rules! eval_block {
    ({} $S:tt $N:tt $P:tt $V:tt $D:tt) => {
    };
    ({ use $($I:ident)::+; $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        $($I)::*!({ $($T)* } () ($crate::eval_use_import; [$($I)::*] $N) $P $V $);
    };
    ({ use $($I:ident)::+ as $A:ident; $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        $($I)::*!({ $($T)* } () ($crate::eval_use_import; [$A] $N) $P $V $);
    };
    ({ let $L:tt = $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval::expression!({ $($T)* } () ($crate::eval::operator; [] ($crate::eval_let_binding; $L $N)) $P $V $);
    };
    ({ $(#[$A:meta])* pub $(($($E:tt)*))? let $L:ident = $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval::expression!({ $($T)* } () ($crate::eval::operator; [] ($crate::eval_let_binding_pub; $L [$(#[$A])*] [pub $(($($E)*))*] $N)) $P $V $);
    };
    ({ expand { $($B:tt)* } $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P) => {
                $($B)*
            };
        }
        __rukt_transcribe!($V);
        $crate::eval::block!({ $($T)* } () $N $P $V $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_let_binding {
    ({ ; $($T:tt)* } $S:tt _ $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval::block!({ $($T)* } () $N $P $V $);
    };
    ({ ; $($T:tt)* } $S:tt $I:ident $N:tt [$($P:tt)*] [$($V:tt)*] $D:tt) => {
        $crate::eval::block!({ $($T)* } () $N [$($P)* $D$I:tt] [$($V)* $S] $);
    };
    ({ ; $($T:tt)* } $S:tt $L:tt $N:tt [$($P:tt)*] [$($V:tt)*] $D:tt) => {
        $crate::eval::block!({ $($T)* } () $N [$($P)* $L] [$($V)* $S] $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_let_binding_pub {
    ({ ; $($T:tt)* } $S:tt $I:ident $A:tt $E:tt $N:tt [$($P:tt)*] [$($V:tt)*] $D:tt) => {
        $crate::utils::escape_repetitions!([$S] [] [$DD] ($crate::export_variable; $I $A $E [$DD:tt] $));
        $crate::eval::block!({ $($T)* } () $N [$($P)* $D$I:tt] [$($V)* $S] $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! export_variable {
    ([$S:tt] $I:ident [$($A:tt)*] [$($E:tt)+] [$($M:tt)+] $D:tt) => {
        $($A)*
        macro_rules! $I {
            () => {
                $I!{@internal $}
            };
            (@internal $($M)*) => {
                $S
            };
            ($TT:tt $SS:tt ($FF:path; $D($CC:tt)*) $PP:tt $VV:tt $($M)*) => {
                $FF!($TT $S $D($CC)* $PP $VV $);
            };
        }
        $($E)* use $I;
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_use_import {
    ($T:tt $S:tt [$I:ident] $N:tt [$($P:tt)*] [$($V:tt)*] $D:tt) => {
        $crate::eval::block!($T () $N [$($P)* $D$I:tt] [$($V)* $S] $);
    };
    ($T:tt $S:tt [$_:ident $(::$I:ident)+] $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval_use_import!($T $S [$($I)::*] $N $P $V $);
    };
}

/// Evaluate statements within blocks.
///
/// Rukt blocks can contain the following statements:
///
/// - [Let bindings](#let-bindings)
/// - [Expand statements](#expand-statements)
/// - [Exports](#exports)
/// - [Imports](#imports)
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
///
/// # Exports
///
/// By default, none of the variables created during the expansion of a
/// [`rukt`](crate::rukt) block will be visible to the outside.
///
/// You can use the `pub` keyword with the `#[macro_export]` attribute to export
/// variables and make them accessible from [`rukt`](crate::rukt) blocks in
/// other crates.
///
/// ```
/// # use rukt::rukt;
/// // my_crate/src/lib.rs
/// rukt! {
///     #[macro_export]
///     pub let values = {
///         A: 1,
///         B: 2,
///         C: 3,
///     };
/// }
/// ```
/// ```
/// # mod my_crate {
/// #     use rukt::rukt;
/// #     rukt! {
/// #         pub(crate) let values = {
/// #             A: 1,
/// #             B: 2,
/// #             C: 3,
/// #         };
/// #     }
/// # }
/// # use rukt::rukt;
/// rukt! {
///     let {$($name:ident: $value:expr,)*} = my_crate::values;
///     expand {
///         enum MyEnum {
///             $($name = $value,)*
///         }
///     }
/// }
/// assert_eq!(MyEnum::A as u32, 1);
/// assert_eq!(MyEnum::B as u32, 2);
/// assert_eq!(MyEnum::C as u32, 3);
/// ```
///
/// In addition to binding the variable in the current scope, the `let`
/// statement will generate a [`builtin`](crate::builtins) that resolves to the
/// assigned value.
///
/// You can make the variable accessible only to other [`rukt`](crate::rukt)
/// blocks in your own crate with the usual `pub(...)` variants. Of course when
/// the variable is not meant to be visible to other crates there's no need for
/// `#[macro_export]`.
///
/// In regular Rust, `pub(self)` is equivalent to not using `pub` in the first
/// place. In Rukt it can be used to signal that you want to export the variable
/// as a builtin without extending its visibility beyond the current Rust scope.
/// As mentioned earlier, Rukt variables are not exported by default, there's no
/// trace of them in the surrounding Rust code unless you use the `pub` keyword.
///
/// Exported variables can also be used directly as macros in the surrounding
/// Rust code.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     pub(self) let numbers = [1, 2, 3];
/// }
/// assert_eq!(numbers!(), [1, 2, 3]);
/// ```
///
/// # Imports
///
/// Rukt supports `use` statements as an alternative to `let` bindings for
/// bringing exported variables into scope.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     pub(crate) let numbers = [1, 2, 3];
/// }
/// rukt! {
///     use numbers;
///     expand {
///         assert_eq!($numbers, [1, 2, 3]);
///     }
/// }
/// ```
///
/// While you can refer to exported variables by path in
/// [expressions](expression), they must be brought into scope within the
/// [`rukt`](crate::rukt) code block to allow substitution in token trees.
///
/// ```compile_fail
/// # use rukt::rukt;
/// rukt! {
///     pub(crate) let numbers = [1, 2, 3];
/// }
/// rukt! {
///     expand {
///         assert_eq!($numbers, [1, 2, 3]); // error: no rules expected the token `$`
///     }
/// }
/// ```
///
/// Rukt `use` statements also support the `as` keyword for bringing exported
/// variables into scope under a different name.
///
/// ```
/// # mod path {
/// #     pub mod to {
/// #         use rukt::rukt;
/// #         rukt! {
/// #             pub(crate) let my_variable = 123;
/// #         }
/// #     }
/// # }
/// # use rukt::rukt;
/// rukt! {
///     use path::to::my_variable;
///     use path::to::my_variable as alias;
/// }
/// ```
///
/// Note that both variants of the `use` statement are nothing more than a
/// restricted version of `let` which only allow binding exported variables.
/// They're functionally completely equivalent. Rukt `use` statements simply
/// make it easier to identify imports at first glance.
///
/// ```
/// # mod path {
/// #     pub mod to {
/// #         use rukt::rukt;
/// #         rukt! {
/// #             pub(crate) let my_variable = 123;
/// #         }
/// #     }
/// # }
/// # use rukt::rukt;
/// rukt! {
///     let my_variable = path::to::my_variable;
///     let alias = path::to::my_variable;
/// }
/// ```
#[doc(inline)]
pub use eval_block as block;

#[doc(hidden)]
#[macro_export]
macro_rules! eval_expression {
    ({ true $($T:tt)* } $S:tt ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!({ $($T)* } true $($C)* $P $V $);
    };
    ({ false $($T:tt)* } $S:tt ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!({ $($T)* } false $($C)* $P $V $);
    };
    ({ $I:ident $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P $TT:tt $NN:tt $PP:tt $VV:tt) => {
                $crate::eval_identifier!($TT [$D$I] $NN $PP $VV $);
            };
        }
        __rukt_transcribe!($V { $($T)* } $N $P $V);
    };
    ({ ($($R:tt)*) $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P $TT:tt ($FF:path; $D($CC:tt)*) $PP:tt $VV:tt) => {
                $FF!($TT ($($R)*) $D($CC)* $PP $VV $);
            };
        }
        __rukt_transcribe!($V { $($T)* } $N $P $V);
    };
    ({ [$($R:tt)*] $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P $TT:tt ($FF:path; $D($CC:tt)*) $PP:tt $VV:tt) => {
                $FF!($TT [$($R)*] $D($CC)* $PP $VV $);
            };
        }
        __rukt_transcribe!($V { $($T)* } $N $P $V);
    };
    ({ {$($R:tt)*} $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        macro_rules! __rukt_transcribe {
            ($P $TT:tt ($FF:path; $D($CC:tt)*) $PP:tt $VV:tt) => {
                $FF!($TT {$($R)*} $D($CC)* $PP $VV $);
            };
        }
        __rukt_transcribe!($V { $($T)* } $N $P $V);
    };
    ({ ! $($T:tt)* } $S:tt $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval::expression!({ $($T)* } () ($crate::eval::operator; [!] $N) $P $V $);
    };
    ({ $R:tt $($T:tt)* } $S:tt ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!({ $($T)* } $R $($C)* $P $V $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_identifier {
    ($T:tt [$S:tt] ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T $S $($C)* $P $V $);
    };
    ($T:tt [$_:tt $S:tt] $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval_builtin!($T [$S] $N $P $V $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_builtin {
    ({ ::$I:ident $($T:tt)* } [$($S:tt)+] $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval_builtin!({ $($T)* } [$($S)*::$I] $N $P $V $);
    };
    ($T:tt [$($S:tt)+] $N:tt $P:tt $V:tt $D:tt) => {
        $($S)*!($T () $N $P $V $);
    };
}

/// Evaluate expression.
///
/// Rukt expressions support the following:
///
/// - [Literals](#literals)
/// - [Variables](#variables)
/// - [Builtins](crate::builtins)
/// - [Operators](operator)
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
macro_rules! eval_operator {
    // ! operator
    ($T:tt $S:tt [!] $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval_not!($T $S $N $P $V $);
    };

    // comparison operators
    ($T:tt $S:tt [== $R:tt] $N:tt $P:tt $V:tt $D:tt) => {
        // todo: escape dollar
        $crate::eval_select!($T $R [{ [$S] [true] } { [$_:tt] [false] }] $N $P $V $);
    };
    ($T:tt $S:tt [!= $R:tt] $N:tt $P:tt $V:tt $D:tt) => {
        // todo: escape dollar
        $crate::eval_select!($T $R [{ [$S] [false] } { [$_:tt] [true] }] $N $P $V $);
    };
    ({ == $($T:tt)* } $S:tt $O:tt $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval::expression!({ $($T)* } () ($crate::eval::operator; [== $S] ($crate::eval::operator; $O $N)) $P $V $);
    };
    ({ != $($T:tt)* } $S:tt $O:tt $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval::expression!({ $($T)* } () ($crate::eval::operator; [!= $S] ($crate::eval::operator; $O $N)) $P $V $);
    };

    // boolean operators
    ($T:tt $S:tt [&& $R:tt] $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval_and!($T $R $S $N $P $V $);
    };
    ({ && $($T:tt)* } $S:tt $O:tt $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval::expression!({ $($T)* } () ($crate::eval::operator; [&& $S] ($crate::eval::operator; $O $N)) $P $V $);
    };
    ($T:tt $S:tt [|| $R:tt] $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval_or!($T $R $S $N $P $V $);
    };
    ({ || $($T:tt)* } $S:tt $O:tt $N:tt $P:tt $V:tt $D:tt) => {
        $crate::eval::expression!({ $($T)* } () ($crate::eval::operator; [|| $S] ($crate::eval::operator; $O $N)) $P $V $);
    };

    // nothing
    ($T:tt $S:tt [] ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T $S $($C)* $P $V $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_not {
    ($T:tt true ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T false $($C)* $P $V $);
    };
    ($T:tt false ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T true $($C)* $P $V $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_select {
    ($T:tt $S:tt [$({ [$($R1:tt)*] [$($R2:tt)*] })+] $N:tt $P:tt $V:tt $D:tt) => {
        macro_rules! __rukt_dispatch {
            $(
                ($($R1)* $TT:tt ($FF:path; $D($CC:tt)*) $PP:tt $VV:tt) => {
                    $FF!($TT $($R2)* $D($CC)* $PP $VV $);
                };
            )*
        }
        __rukt_dispatch!($S $T $N $P $V);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_and {
    // explicit truth table to validate the rhs even when its value doesn't matter
    ($T:tt true true ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T true $($C)* $P $V $);
    };
    ($T:tt false false ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T false $($C)* $P $V $);
    };
    ($T:tt false true ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T false $($C)* $P $V $);
    };
    ($T:tt true false ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T false $($C)* $P $V $);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! eval_or {
    // explicit truth table to validate the rhs even when its value doesn't matter
    ($T:tt true true ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T true $($C)* $P $V $);
    };
    ($T:tt false false ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T false $($C)* $P $V $);
    };
    ($T:tt false true ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T true $($C)* $P $V $);
    };
    ($T:tt true false ($F:path; $($C:tt)*) $P:tt $V:tt $D:tt) => {
        $F!($T true $($C)* $P $V $);
    };
}

/// Evaluate operator.
///
/// Rukt supports the following operators:
///
/// - [Comparison operators](#comparison-operators)
/// - [Boolean operators](#boolean-operators)
///
/// # Comparison operators
///
/// You can use `==` and `!=` for comparing tokens.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let value = 42;
///     let equal = value == 42;
///     let not_equal = equal != false;
///     expand {
///         assert_eq!($equal, true);
///         assert_eq!($not_equal, true);
///     }
/// }
/// ```
///
/// # Boolean operators
///
/// You can use the typical `!`, `&&`, and `||` boolean operators.
///
/// ```
/// # use rukt::rukt;
/// rukt! {
///     let a = !true;
///     let b = !false;
///     expand {
///         assert_eq!([$a, $b], [false, true]);
///     }
/// }
/// rukt! {
///     let a = true && true;
///     let b = true && false;
///     let c = false && true;
///     let d = false && false;
///     expand {
///         assert_eq!([$a, $b, $c, $d], [true, false, false, false]);
///     }
/// }
/// rukt! {
///     let a = true || true;
///     let b = true || false;
///     let c = false || true;
///     let d = false || false;
///     expand {
///         assert_eq!([$a, $b, $c, $d], [true, true, true, false]);
///     }
/// }
/// ```
///
/// These operators will fail to compile when used with tokens other than `true`
/// and `false`.
///
/// ```compile_fail
/// # use rukt::rukt;
/// rukt! {
///     let value = 42;
///     let _ = true && value; // error: no rules expected the token `42`
/// }
/// ```
///
/// Note that unlike in regular Rust, the right-side of `&&` and `||` is not
/// lazy and will always be evaluated eagerly.
#[doc(inline)]
pub use eval_operator as operator;
