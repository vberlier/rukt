#![recursion_limit = "512"]

use rukt::rukt;

#[test]
fn expand() {
    let mut value = 1;
    rukt! {
        expand {
            value += 2;
        }
        expand {
            value *= 3;
        }
    }
    assert_eq!(value, 9);
}

#[test]
fn let_bool() {
    rukt! {
        let a = true;
        let b = false;
        expand {
            const A: bool = $a;
            const B: bool = $b;
        }
    }
    assert_eq!(A, true);
    assert_eq!(B, false);
}

#[test]
fn let_literal() {
    rukt! {
        let a = 123;
        let b = "hello";
        expand {
            const A: u32 = $a;
            const B: &str = $b;
        }
    }
    assert_eq!(A, 123);
    assert_eq!(B, "hello");
}

#[test]
fn let_token_tree() {
    rukt! {
        let a = (^here);
        let b = {
            [ARBITRARY SYNTAX] in $a
            where "nothing" => match
        };
        let {[$($c:ident)*] $($_:tt)*} = b;
        expand {
            $(const $c: &str = stringify!($b);)*
        }
    }
    assert_eq!(ARBITRARY, SYNTAX);
    assert_eq!(
        SYNTAX,
        "{ [ARBITRARY SYNTAX] in (^ here) where \"nothing\" => match }"
    );
}

#[test]
fn let_export() {
    rukt! {
        pub(self) let value = [1, 2, 3];
    }
    rukt! {
        let [$($number:tt),*] = value;
        expand {
            const VALUE: u32 = 0 $(+ $number)*;
        }
    }
    assert_eq!(VALUE, 6);
    assert_eq!(value!(), [1, 2, 3]);
}

#[test]
fn parse_regular_macro() {
    rukt! {
        let value = { 7 [arbitrary] stuff ... };
        pub(self) let string = rukt::builtins::parse::<expr>(stringify!($value));
    }
    assert_eq!(string!(), "{ 7 [arbitrary] stuff ... }");
}

#[test]
fn comparison() {
    rukt! {
        let a = "foo" == "bar";
        let b = false == a;
        let c = a != b;
        expand {
            assert_eq!($a, false);
            assert_eq!($b, true);
            assert_eq!($c, true);
        }
    }
}

#[test]
fn comparison_escape() {
    rukt! {
        let a = ($dummy:tt) == (anything);
        let b = (anything) == ($dummy:tt);
        let c = ($dummy:tt) == ($dummy:tt) ;
        expand {
            assert_eq!($a, false);
            assert_eq!($b, false);
            assert_eq!($c, true);
        }
    }
}

#[test]
fn boolean() {
    rukt! {
        let p0 = true && true && true;
        let p1 = true && true && false;
        let p2 = true && false && true;
        let p3 = true && false && false;
        let p4 = false && true && true;
        let p5 = false && true && false;
        let p6 = false && false && true;
        let p7 = false && false && false;
        expand {
            assert_eq!($p0, true);
            assert_eq!($p1, false);
            assert_eq!($p2, false);
            assert_eq!($p3, false);
            assert_eq!($p4, false);
            assert_eq!($p5, false);
            assert_eq!($p6, false);
            assert_eq!($p7, false);
        }
    };
    rukt! {
        let p0 = true || true || true;
        let p1 = true || true || false;
        let p2 = true || false || true;
        let p3 = true || false || false;
        let p4 = false || true || true;
        let p5 = false || true || false;
        let p6 = false || false || true;
        let p7 = false || false || false;
        expand {
            assert_eq!($p0, true);
            assert_eq!($p1, true);
            assert_eq!($p2, true);
            assert_eq!($p3, true);
            assert_eq!($p4, true);
            assert_eq!($p5, true);
            assert_eq!($p6, true);
            assert_eq!($p7, false);
        }
    }
    rukt! {
        let p1 = false && true || true;
        let p2 = true || true && false;
        expand {
            assert_eq!($p1, true);
            assert_eq!($p2, true);
        }
    }
}

#[test]
fn starts_with() {
    use rukt::builtins::starts_with;
    rukt! {
        let a = [1 2 3].starts_with(1 2);
        let b = [1 2 3].starts_with(2 2);
        let c = [1 2 3].starts_with(1 2 3 4);
        expand {
            assert_eq!($a, true);
            assert_eq!($b, false);
            assert_eq!($c, false);
        }
    }
}

#[test]
fn starts_with_escape() {
    use rukt::builtins::starts_with;
    rukt! {
        let D = $;
        let a = [1 2 $D($_:tt)*].starts_with($D($_:tt)*);
        let b = [1 2 $D($_:tt)*].starts_with(1 2);
        let c = [1 2 $D($_:tt)*].starts_with(2 2);
        let d = [1 2 $D($_:tt)*].starts_with(1 2 $D($T:tt)*);
        let e = [1 2 $D($_:tt)*].starts_with(1 2 $D($_:tt)*);
        let f = [1 2 $D($_:tt)*].starts_with(1 2 3 4);
        expand {
            assert_eq!($a, false);
            assert_eq!($b, true);
            assert_eq!($c, false);
            assert_eq!($d, false);
            assert_eq!($e, true);
            assert_eq!($f, false);
        }
    }
}

#[test]
fn user_function() {
    rukt! {
        fn foo($n:expr) {
            let inner = (123 * $n);
            inner
        }
        let value = foo(2);
        expand {
            assert_eq!($value, 246);
            assert_eq!(stringify!($inner), "$inner");
        }
    }
}

#[test]
fn user_function_value() {
    rukt! {
        fn double($($args:tt)*) {
            ($($args)* $($args)*)
        }
        fn apply($f:tt $($args:tt)*) {
            f($($args)*)
        }
        let d = apply($double mind blown);
        expand {
            assert_eq!(stringify!($d), "(mind blown mind blown)");
        }
    }
}

#[test]
fn user_function_expand() {
    rukt! {
        fn define($name:ident, $n:expr) {
            expand {
                const $name: u32 = $n;
            }
        }
        define(SEVEN, 7);
        let result = define(NINE, 9);
        expand {
            assert_eq!($result, ());
        }
    }
    assert_eq!(SEVEN, 7);
    assert_eq!(NINE, 9);
}

#[test]
fn manual_function() {
    rukt! {
        let ($name:ident) = (test);
        let manual_fn = {
            fn $name() {
                $name
            }
        };
        let a = manual_fn();
        let b = manual_fn()();
        let equal = a == b && b()()() == b;
        expand {
            assert_eq!(stringify!($a), "{ fn test () { test } }");
            assert_eq!(stringify!($b), "{ fn test () { test } }");
            assert_eq!($equal, true);
        }
    }
}

#[test]
fn condition() {
    use rukt::builtins::starts_with;
    rukt! {
        let value = if [1 2 3].starts_with(-1 2) {
            expand {
                compile_error!("invalid");
            }
        } else {
            42
        };

        let unit = if false {} else {};

        let result = if true == false {
            1
        } else if "something" == "other thing" {
            2
        } else if true == (42).starts_with($value) && value == 42 {
            if true {
                let inner = 3;
                inner
            } else {
                9999
            }
        } else {
            4
        };

        fn total() {
            if true {
                8
            } else {
                9
            }
        }
        let total_result = total();

        fn partial() {
            if true {
                8
            }
        }
        let partial_result = partial();

        fn with_semi() {
            if true {
                8
            } else {
                9
            };
        }
        let with_semi_result = with_semi();

        expand {
            assert_eq!($value, 42);
            assert_eq!($unit, ());
            assert_eq!($result, 3);
            assert_eq!(stringify!($inner), "$inner");
            assert_eq!($total_result, 8);
            assert_eq!($partial_result, ());
            assert_eq!($with_semi_result, ());
        }
    }
}

#[test]
fn condition_function() {
    rukt! {
        let result = true && if true {
            fn f($n:tt) {
                n
            }
            f
        } else {
            fn f($n:tt) {
            }
            f
        }(123) == 123;
        expand {
            assert_eq!($result, true);
        }
    }
}

#[test]
fn recursion() {
    use rukt::builtins::starts_with;
    rukt! {
        let [$($start:tt)*] = [
            123
            456
        ];
        fn f($a:tt $($remaining:tt)*) {
            if a == "stop" {
                [start $($start)*]
            } else {
                let [$($prefix:tt)*] = if [$($remaining)*].starts_with(2) {
                    [double it]
                } else if [$($remaining)*].starts_with(3) {
                    [just wait]
                } else {
                    []
                };
                let [$($result:tt)*] = f($($remaining)*);
                [$($result)* -> $($prefix)* $a]
            }
        }
        let result = f(1 2 3 "stop" ignored);
        expand {
            assert_eq!(stringify!($result), "[start 123 456 -> 3 -> just wait 2 -> double it 1]");
        }
    }
}
