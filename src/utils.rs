//! Reusable macro utilities.

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_repetitions {
    ([($($G:tt)*) $($T:tt)*] $R:tt $M:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] $M ($crate::utils_escape_repetitions_parens; [$($T)*] $R $M $N));
    };
    ([[$($G:tt)*] $($T:tt)*] $R:tt $M:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] $M ($crate::utils_escape_repetitions_brackets; [$($T)*] $R $M $N));
    };
    ([{$($G:tt)*} $($T:tt)*] $R:tt $M:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] $M ($crate::utils_escape_repetitions_braces; [$($T)*] $R $M $N));
    };
    ([$H:tt ($($G:tt)*) $($T:tt)*] $R:tt $M:tt $N:tt) => {
        $crate::utils_escape_repetitions_detect!([=$H=] ($($G)*) [$($T)*] $R $M $N);
    };
    ([$H:tt $($T:tt)*] [$($R:tt)*] $M:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($T)*] [$($R)* $H] $M $N);
    };
    ([] [$($R:tt)*] $M:tt ($F:path; $($C:tt)*)) => {
        $F!([$($R)*] $($C)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_repetitions_detect {
    ([$(=)$+] ($($G:tt)*) $T:tt [$($R:tt)*] [$($M:tt)*] $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] [$($M)*] ($crate::utils_escape_repetitions_parens; $T [$($R)* $($M)*] [$($M)*] $N));
    };
    ([=$H:tt=] ($($G:tt)*) $T:tt [$($R:tt)*] $M:tt $N:tt) => {
        $crate::utils::escape_repetitions!([$($G)*] [] $M ($crate::utils_escape_repetitions_parens; $T [$($R)* $H] $M $N));
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_repetitions_parens {
    ([$($G:tt)*] $T:tt [$($R:tt)*] $M:tt $N:tt) => {
        $crate::utils::escape_repetitions!($T [$($R)* ($($G)*)] $M $N);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_repetitions_brackets {
    ([$($G:tt)*] $T:tt [$($R:tt)*] $M:tt $N:tt) => {
        $crate::utils::escape_repetitions!($T [$($R)* [$($G)*]] $M $N);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! utils_escape_repetitions_braces {
    ([$($G:tt)*] $T:tt [$($R:tt)*] $M:tt $N:tt) => {
        $crate::utils::escape_repetitions!($T [$($R)* {$($G)*}] $M $N);
    };
}

/// Escape the dollar sign `$` prefixing `macro_rules` repetitions with the
/// given metavariable.
///
/// The macro accepts the source tokens, followed by the initial output tokens,
/// followed by the metavariable, followed by a next continuation.
///
/// ```
/// # use rukt::utils::escape_repetitions;
/// macro_rules! define {
///     ([$($T:tt)*] $I:ident) => {
///         const $I: &str = stringify!($($T)*);
///     }
/// }
/// escape_repetitions!([$name:ident($($arg:expr),*)] [] [$D] (define; CALL_PATTERN));
/// assert_eq!(CALL_PATTERN.replace(" ", ""), "$name:ident($D($arg:expr),*)");
/// ```
///
/// Notice how the repetition `$($arg:expr),*` turned into `$D($arg:expr),*`.
/// The dollar sign `$` that would normally cause `macro_rules` to identify a
/// potential [fragment
/// repetition](https://doc.rust-lang.org/reference/macros-by-example.html#repetitions)
/// in the input tokens got replaced with the specified metavariable `$D`.
///
/// This is useful when tokens are meant to be pasted into a generated macro,
/// but expanded result should not be interpreted by `macro_rules`.
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
///     ([$($T:tt)*] $I:ident $($M:tt)+) => {
///         macro_rules! $I {
///             ($($M)*) => {
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! check {
        ($T:tt $expected:expr) => {
            assert_eq!(stringify!($T), $expected);
        };
    }

    #[test]
    fn basic() {
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
