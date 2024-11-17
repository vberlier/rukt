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
