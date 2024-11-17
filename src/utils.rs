//! Reusable macro utilities.

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape {
    ([($($G:tt)*) $($T:tt)*] $R:tt $E:tt $N:tt) => {
        $crate::utils::escape!([$($G)*] [] $E ($crate::utils_escape_collect_parens; [$($T)*] $R $E ($crate::utils::escape) $N));
    };
    ([[$($G:tt)*] $($T:tt)*] $R:tt $E:tt $N:tt) => {
        $crate::utils::escape!([$($G)*] [] $E ($crate::utils_escape_collect_brackets; [$($T)*] $R $E ($crate::utils::escape) $N));
    };
    ([{$($G:tt)*} $($T:tt)*] $R:tt $E:tt $N:tt) => {
        $crate::utils::escape!([$($G)*] [] $E ($crate::utils_escape_collect_braces; [$($T)*] $R $E ($crate::utils::escape) $N));
    };
    ([$H:tt $($T:tt)*] $R:tt $E:tt $N:tt) => {
        $crate::utils_escape_detect!([=$H=] [$($T)*] $R $E $N);
    };
    ([] [$($R:tt)*] $E:tt ($F:path; $($C:tt)*)) => {
        $F!([$($R)*] $($C)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_detect {
    ([$(=)$+] $T:tt [$($R:tt)*] [$($E:tt)*] $N:tt) => {
        $crate::utils::escape!($T [$($R)* $($E)*] [$($E)*] $N);
    };
    ([=$H:tt=] $T:tt [$($R:tt)*] $E:tt $N:tt) => {
        $crate::utils::escape!($T [$($R)* $H] $E $N);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_collect_parens {
    ([$($G:tt)*] $T:tt [$($R:tt)*] $E:tt ($F:path) $N:tt) => {
        $F!($T [$($R)* ($($G)*)] $E $N);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_collect_brackets {
    ([$($G:tt)*] $T:tt [$($R:tt)*] $E:tt ($F:path) $N:tt) => {
        $F!($T [$($R)* [$($G)*]] $E $N);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_collect_braces {
    ([$($G:tt)*] $T:tt [$($R:tt)*] $E:tt ($F:path) $N:tt) => {
        $F!($T [$($R)* {$($G)*}] $E $N);
    };
}

/// Replace dollar sign tokens `$` with the given tokens.
///
/// The macro accepts the source tokens, followed by the initial output tokens,
/// followed by the escape tokens, followed by a next continuation.
///
/// ```
/// # use rukt::utils::escape;
/// macro_rules! define {
///     ([$($T:tt)*] $I:ident) => {
///         const $I: &str = stringify!($($T)*);
///     }
/// }
/// escape!([$name:ident($($arg:expr),*)] [] [<dollar>] (define; CALL_PATTERN));
/// assert_eq!(CALL_PATTERN, "<dollar>name:ident(<dollar>(<dollar>arg:expr),*)");
/// ```
///
/// Notice how all dollar signs the input tokens got replaced with the escape
/// tokens.
///
/// This is useful when the input tokens are meant to be matched against
/// literally as a pattern in a generated macro.
#[doc(inline)]
pub use utils_escape as escape;

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_repetitions {
    ([($($G:tt)*) $($T:tt)*] $R:tt $E:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] $E ($crate::utils_escape_collect_parens; [$($T)*] $R $E ($crate::utils::escape_repetitions) $N));
    };
    ([[$($G:tt)*] $($T:tt)*] $R:tt $E:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] $E ($crate::utils_escape_collect_brackets; [$($T)*] $R $E ($crate::utils::escape_repetitions) $N));
    };
    ([{$($G:tt)*} $($T:tt)*] $R:tt $E:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] $E ($crate::utils_escape_collect_braces; [$($T)*] $R $E ($crate::utils::escape_repetitions) $N));
    };
    ([$H:tt ($($G:tt)*) $($T:tt)*] $R:tt $E:tt $N:tt) => {
        $crate::utils_escape_repetitions_detect!([=$H=] ($($G)*) [$($T)*] $R $E $N);
    };
    ([$H:tt $($T:tt)*] [$($R:tt)*] $E:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($T)*] [$($R)* $H] $E $N);
    };
    ([] [$($R:tt)*] $E:tt ($F:path; $($C:tt)*)) => {
        $F!([$($R)*] $($C)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_repetitions_detect {
    ([$(=)$+] ($($G:tt)*) $T:tt [$($R:tt)*] [$($E:tt)*] $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] [$($E)*] ($crate::utils_escape_collect_parens; $T [$($R)* $($E)*] [$($E)*] ($crate::utils::escape_repetitions) $N));
    };
    ([=$H:tt=] ($($G:tt)*) $T:tt [$($R:tt)*] $E:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] $E ($crate::utils_escape_collect_parens; $T [$($R)* $H] $E ($crate::utils::escape_repetitions) $N));
    };
}

/// Replace the dollar sign `$` prefixing `macro_rules` repetitions with the
/// given tokens.
///
/// The macro accepts the source tokens, followed by the initial output tokens,
/// followed by the escape tokens, followed by a next continuation.
///
/// ```
/// # use rukt::utils::escape_repetitions;
/// macro_rules! define {
///     ([$($T:tt)*] $I:ident) => {
///         const $I: &str = stringify!($($T)*);
///     }
/// }
/// escape_repetitions!([$name:ident($($arg:expr),*)] [] [<dollar>] (define; CALL_PATTERN));
/// assert_eq!(CALL_PATTERN.replace(" ", ""), "$name:ident(<dollar>($arg:expr),*)");
/// ```
///
/// Notice how the repetition `$($arg:expr),*` turned into
/// `<dollar>($arg:expr),*`. The dollar sign `$` that would normally cause
/// `macro_rules` to identify a potential [fragment
/// repetition](https://doc.rust-lang.org/reference/macros-by-example.html#repetitions)
/// in the input tokens got replaced with the escape tokens.
///
/// This is useful when tokens are meant to be pasted into a generated macro,
/// but the expanded result should not be interpreted by `macro_rules`. (See
/// [`rust/35853`](https://github.com/rust-lang/rust/issues/35853))
///
/// The following example won't compile because the repetition `$($arg:expr),*`
/// pasted in the generated macro will be interpreted by `macro_rules` upon
/// expansion.
///
/// ```compile_fail
/// macro_rules! define {
///     ([$($T:tt)*] $I:ident) => {
///         macro_rules! $I {
///             () => {
///                 stringify!($($T)*)
///             };
///         }
///     }
/// }
/// define!([$name:ident($($arg:expr),*)] call_pattern);
/// assert_eq!(call_pattern!().replace(" ", ""), "$name:ident($($arg:expr),*)");
/// ```
/// ```text
/// error: attempted to repeat an expression containing no syntax variables matched as repeating at this depth
/// ```
///
/// By using [`escape_repetitions`], you can replace all the dollar sign `$`
/// tokens that would normally cause `macro_rules` to identify a potential
/// [fragment
/// repetition](https://doc.rust-lang.org/reference/macros-by-example.html#repetitions)
/// with a metavariable you'll substitute back for a dollar sign `$` when
/// expanding the generated macro.
///
/// ```
/// # use rukt::utils::escape_repetitions;
/// macro_rules! define {
///     ([$($T:tt)*] $I:ident $($E:tt)+) => {
///         macro_rules! $I {
///             ($($E)*) => {
///                 stringify!($($T)*)
///             };
///         }
///     }
/// }
/// escape_repetitions!([$name:ident($($arg:expr),*)] [] [$D] (define; call_pattern $D:tt));
/// assert_eq!(call_pattern!($).replace(" ", ""), "$name:ident($($arg:expr),*)");
/// ```
///
/// Note that due to
/// [hygiene](https://doc.rust-lang.org/reference/macros-by-example.html#hygiene),
/// the final substitution to unescape the dollar sign tokens only works if the
/// metavariable used for escaping and its corresponding pattern appear in
/// compatible lexical scopes. Passing them both at the call site ensures that
/// they match.
#[doc(inline)]
pub use utils_escape_repetitions as escape_repetitions;

#[doc(hidden)]
#[macro_export]
macro_rules! utils_select {
    ([$($T:tt)*] [$([[$($R1:tt)*] [$($R2:tt)*]])+] $N:tt $D:tt) => {
        macro_rules! __rukt_dispatch {
            $(
                ([$($R1)*] ($FF:path; $D($CC:tt)*)) => {
                    $FF!([$($R2)*] $D($CC)*);
                };
            )*
        }
        __rukt_dispatch!([$($T)*] $N);
    };
}

/// Select tokens associated with the first matching pattern.
#[doc(inline)]
pub use utils_select as select;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_repetitions() {
        macro_rules! check {
            ($T:tt $expected:expr) => {
                assert_eq!(stringify!($T), $expected);
            };
        }

        escape_repetitions!([] [] [$REP] (check; "[]"));
        escape_repetitions!([hello world] [] [$REP] (check; "[hello world]"));
        escape_repetitions!([hello(world)] [] [$REP] (check; "[hello(world)]"));
        escape_repetitions!([{ hello }(world)] [] [$REP] (check; "[{ hello }(world)]"));
        escape_repetitions!([$($hello)* world] [] [$REP] (check; "[$REP($hello)* world]"));
        escape_repetitions!([$($hello)*(world)] [] [$REP] (check; "[$REP($hello)*(world)]"));
        escape_repetitions!([{ $($hello)* }(world)] [] [$REP] (check; "[{ $REP($hello)* }(world)]"));
        escape_repetitions!([$($hello)* $($world:tt, 42)+] [] [$REP] (check; "[$REP($hello)* $REP($world:tt, 42)+]"));
        escape_repetitions!([$($hello)*($($world:tt, 42)+)] [] [$REP] (check; "[$REP($hello)*($REP($world:tt, 42)+)]"));
        escape_repetitions!([{ $($hello)* }($($world:tt, 42)+)] [] [$REP] (check; "[{ $REP($hello)* }($REP($world:tt, 42)+)]"));
        escape_repetitions!([$($hello $(;)?)* $($world:tt, 42)+] [] [$REP] (check; "[$REP($hello $REP(;)?)* $REP($world:tt, 42)+]"));
        escape_repetitions!([$($hello $(;)?)*($($world:tt, 42)+)] [] [$REP] (check; "[$REP($hello $REP(;)?)*($REP($world:tt, 42)+)]"));
        escape_repetitions!([{ $($hello $(;)?)* }($($world:tt, 42)+)] [] [$REP] (check; "[{ $REP($hello $REP(;)?)* }($REP($world:tt, 42)+)]"));
    }
}
