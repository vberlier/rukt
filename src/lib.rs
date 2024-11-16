#![doc = include_str!("../README.md")]

pub mod builtins;
pub mod eval;

pub mod utils;

/// Rukt code block.
///
/// The primary entry point to evaluate and expand Rukt
/// [statements](crate::eval::block).
#[macro_export]
macro_rules! rukt {
    ($($T:tt)*) => {
        $crate::eval::block!({ $($T)* } () / [] [] $);
    };
}
