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
fn let_group() {
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
