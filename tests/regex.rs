//! Tests parsing plain regexes that are generated with the `generate!` macro
//! from a reader, like an external crate would use this library.

#[macro_use(generate)]
extern crate calc_regex;

#[test]
fn simple_regex() {
    let re: calc_regex::CalcRegex = generate! {
        re = "foo";
    };
    // assert!(re.is_bounded());
    let mut reader = calc_regex::Reader::from_array(b"foo");
    let record = reader.parse(&re).unwrap();
    let expected = b"foo";
    let actual = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn repeat() {
    let re = generate! {
        re = "foo"^3;
    };
    // assert!(re.is_bounded());
    let mut reader = calc_regex::Reader::from_array(b"foofoofoo");
    let record = reader.parse(&re).unwrap();
    let expected = b"foofoofoo";
    let actual = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn length_bound_within() {
    let mut re = generate! {
        re = "foo";
    };
    re.set_root_length_bound(5);
    let mut reader = calc_regex::Reader::from_array(b"foo");
    let record = reader.parse(&re).unwrap();
    let expected = b"foo";
    let actual = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn length_bound_exact() {
    let mut re = generate! {
        re = "foo";
    };
    re.set_root_length_bound(3);
    let mut reader = calc_regex::Reader::from_array(b"foo");
    let record = reader.parse(&re).unwrap();
    let expected = b"foo";
    let actual = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn length_bound_exceeded() {
    let mut re = generate! {
        re = "foo";
    };
    re.set_root_length_bound(2);
    let mut reader = calc_regex::Reader::from_array(b"foo");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::Regex { regex, value } = err {
        assert_eq!(regex, "^(?-u:foo)$");
        assert_eq!(value, b"fo");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}
