//! Tests correct application of length bounds.

/// Defines tests for a generic reader.
///
/// All tests are run for each reader that is given via an invocation of this
/// macro.
macro_rules! run_tests {
    ($name:ident, $get_reader:path) => {
        mod $name {
            use ::*;
            use aux::decimal;

// Start of macro-instantiated module.

#[test]
fn regex_unbounded() {
    let mut re = generate! {
        foo = ("a" - "z")^3;
    };
    re.get_root_mut().length_bound = None;
    let mut reader = $get_reader("bar".as_bytes());
    let record = reader.parse(&re).unwrap();
    let expected = b"bar";
    let actual: &[u8] = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn regex_unbounded_empty() {
    let re = generate! {
        foo = "foo"*;
    };
    assert_eq!(re.get_root().length_bound, None);
    let mut reader = $get_reader("".as_bytes());
    let record = reader.parse(&re).unwrap();
    let expected = b"";
    let actual: &[u8] = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn regex_empty_bounded() {
    let mut re = generate! {
        foo = "foo"*;
    };
    re.set_root_length_bound(0);
    let mut reader = $get_reader("".as_bytes());
    let record = reader.parse(&re).unwrap();
    let expected = b"";
    let actual: &[u8] = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn regex_bounded_exact() {
    let re = generate! {
        foo = ("a" - "z")^3;
    };
    assert_eq!(re.get_root().length_bound, Some(3));
    let mut reader = $get_reader("bar".as_bytes());
    let record = reader.parse(&re).unwrap();
    let expected = b"bar";
    let actual: &[u8] = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn regex_bounded_within() {
    let mut re = generate! {
        foo = ("a" - "z")^3;
    };
    re.set_root_length_bound(4);
    let mut reader = $get_reader("bar".as_bytes());
    let record = reader.parse(&re).unwrap();
    let expected = b"bar";
    let actual: &[u8] = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn regex_bounded_exceeded() {
    let mut re = generate! {
        foo = ("a" - "z")^3;
    };
    re.set_root_length_bound(2);
    let mut reader = $get_reader("bar".as_bytes());
    let err = reader.parse(&re).unwrap_err();
    if let ParserError::Regex { ref regex, ref value } = err {
        assert_eq!(regex, "^(?-u:([a-z]){3})$");
        assert_eq!(value, b"ba");
    } else {
        panic!("Unexpected error: {:?}", err)
    }
}

#[test]
fn calc_regex_bounded() {
    let mut re = generate! {
        foo := ("a" - "z")^3;
        bar := foo;
    };
    re.set_root_length_bound(3);
    let mut reader = $get_reader("bar".as_bytes());
    let record = reader.parse(&re).unwrap();
    let expected = b"bar";
    let actual: &[u8] = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn length_count_bounded() {
    let mut re = generate! {
        foo         = "f", "o"*;
        digit       = "0" - "9";
        calc_regex := digit.decimal, "bar", foo#decimal;
    };
    re.set_root_length_bound(7);
    let mut reader = $get_reader("3barfoo".as_bytes());
    let record = reader.parse(&re).unwrap();
    assert_eq!(b"3barfoo", record.get_all());
}

#[test]
fn length_count_bounded_exceeded() {
    let mut re = generate! {
        foo         = "f", "o"*;
        digit       = "0" - "9";
        calc_regex := digit.decimal, "bar", foo#decimal;
    };
    re.set_root_length_bound(6);
    let mut reader = $get_reader("3barfoo".as_bytes());
    let err = reader.parse(&re).unwrap_err();
    if let ParserError::ConflictingBounds { old, new } = err {
        assert_eq!(old, 2);
        assert_eq!(new, 3);
    } else {
        panic!("Unexpected error: {:?}", err)
    }
}

#[test]
fn occurrence_count_bounded() {
    let mut re = generate! {
        foo         = "foo";
        digit       = "0" - "9";
        calc_regex := digit.decimal, "bar", foo^decimal;
    };
    re.set_root_length_bound(10);
    let mut reader = $get_reader("2barfoofoo".as_bytes());
    let record = reader.parse(&re).unwrap();
    assert_eq!(b"2barfoofoo", record.get_all());
}

#[test]
fn occurrence_count_bounded_exceeded() {
    let mut re = generate! {
        foo         = "foo";
        digit       = "0" - "9";
        calc_regex := digit.decimal, "bar", foo^decimal;
    };
    re.set_root_length_bound(9);
    let mut reader = $get_reader("2barfoofoo".as_bytes());
    let err = reader.parse(&re).unwrap_err();
    if let ParserError::Regex { ref regex, ref value } = err {
        assert_eq!(regex, "^(?-u:foo)$");
        assert_eq!(value, b"fo");
    } else {
        panic!("Unexpected error: {:?}", err)
    }
}

// End of macro-instantiated module.
        }
    }
}

run_tests!(stream, Reader::from_stream);
run_tests!(array, Reader::from_array);
