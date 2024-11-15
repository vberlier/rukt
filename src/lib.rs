#![doc = include_str!("../README.md")]

pub mod builtins;
pub mod eval;

/// Rukt code block.
#[macro_export]
macro_rules! rukt {
    ($($T:tt)*) => {
        $crate::eval::block!({ $($T)* } () [] [] [] $);
    };
}
