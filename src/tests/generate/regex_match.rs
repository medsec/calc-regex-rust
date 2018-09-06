//! Generates plain regexes wrapped in `CalcRegex`es and calls them via the
//! external `regex` library. This tests on a higher level whether `generate!`
//! gets things right like operator precedence, Unicode flags, or matching the
//! entire input, but doesn't touch our own parsing routines yet.

use calc_regex::Inner;

#[test]
fn pf_number() {
    let calc_regex = generate! {
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
    };
    let root = calc_regex.get_root();
    if let Inner::Regex(ref re) = root.inner {
        assert!(re.is_match(b"0:"));
        assert!(!re.is_match(b"0"));
        assert!(!re.is_match(b"0 "));
        assert!(!re.is_match(b"00:"));
        assert!(!re.is_match(b"01:"));
        assert!(re.is_match(b"3:"));
        assert!(re.is_match(b"30:"));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_beginning_end() {
    let calc_regex = generate! {
        foo = "foo" | "bar";
    };
    let root = calc_regex.get_root();
    if let Inner::Regex(ref re) = root.inner {
        assert!(re.is_match(b"foo"));
        assert!(re.is_match(b"bar"));
        assert!(!re.is_match(b"foo "));
        assert!(!re.is_match(b" bar"));
        assert!(!re.is_match(b"foo|bar"));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_concat() {
    let calc_regex = generate! {
        foo = "foo" | "bar";
        bar = "a", foo, "z";
    };
    let root = calc_regex.get_root();
    if let Inner::Regex(ref re) = root.inner {
        assert!(re.is_match(b"afooz"));
        assert!(re.is_match(b"abarz"));
        assert!(!re.is_match(b"afoo"));
        assert!(!re.is_match(b"barz"));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn byte() {
    let calc_regex = generate! {
        byte = %0 - %FE;
    };
    let root = calc_regex.get_root();
    if let Inner::Regex(ref re) = root.inner {
        assert!(re.is_match(&[42]));
        assert!(re.is_match(&[0]));
        assert!(re.is_match(&[254]));
        assert!(!re.is_match(&[255]));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn repeat() {
    let calc_regex = generate! {
        foo = "foo" ^ 3;
    };
    let root = calc_regex.get_root();
    if let Inner::Regex(ref re) = root.inner {
        assert!(re.is_match(b"foofoofoo"));
        assert!(!re.is_match(b"foo"));
        assert!(!re.is_match(b""));
        assert!(!re.is_match(b"foofoofoofoo"));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}
