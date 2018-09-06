//! Tests parsing of nested `CalcRegex`es.

use std::str;

fn decimal(number: &[u8]) -> Option<usize> {
    let number = match str::from_utf8(number) {
        Ok(n) => n,
        Err(_) => return None,
    };
    number.parse::<usize>().ok()
}

/// Defines tests for a generic reader.
///
/// All tests are run for each reader that is given via an invocation of this
/// macro.
macro_rules! run_tests {
    ($name:ident, $get_reader:path) => {
        pub mod $name {
            use ::*;
            use super::*;

// Start of macro-instantiated module.

///////////////////////////////////////////////////////////////////////////////
//      Identifier, Regex, Concatenate, Parentheses
///////////////////////////////////////////////////////////////////////////////

#[test]
fn simple_regex() {
    let calc_regex = generate! {
        foo := "foo";
    };
    let mut reader = $get_reader("foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foo", record.get_all());
    // Top-level name is skipped.
    if let Err(NameError::NoSuchName { ref name }) =
        record.get_capture("foo")
    {
        assert_eq!(name, "foo")
    } else {
        panic!("Unexpected error.")
    }
    if let Err(NameError::NoSuchName { ref name }) =
        record.get_capture("bar")
    {
        assert_eq!(name, "bar")
    } else {
        panic!("Unexpected error.")
    }
}

#[test]
fn simple_regex_invalid() {
    let calc_regex = generate! {
        foo := "foo";
    };
    let mut reader = $get_reader("bar".as_bytes());
    let err = reader.parse(&calc_regex).unwrap_err();
    if let ParserError::Regex { ref regex, ref value } = err {
        assert_eq!(regex, "^(?-u:foo)$");
        assert_eq!(value, b"bar");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn simple_regex_invalid_suffix() {
    let calc_regex = generate! {
        foo := "foo";
    };
    let mut reader = $get_reader("oo".as_bytes());
    let err = reader.parse(&calc_regex).unwrap_err();
    if let ParserError::UnexpectedEof = err {
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn simple_regex_trailing() {
    let calc_regex = generate! {
        foo := "foo";
    };
    let mut reader = $get_reader("foobar".as_bytes());
    let err = reader.parse(&calc_regex).unwrap_err();
    if let ParserError::TrailingCharacters = err {
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
#[should_panic]
fn empty_regex() {
    let _ = generate! {
        foo := "";
    };
}

#[test]
fn identifier() {
    let calc_regex = generate! {
        foo  = "foo";
        bar := foo;
    };
    let mut reader = $get_reader("foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foo", record.get_all());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    record.get_capture("bar").unwrap_err();
}

#[test]
fn identifier_regex() {
    let calc_regex = generate! {
        foo = "foo";
        bar = foo; // Plain regex assignment, so no capture is created.
    };
    let mut reader = $get_reader("foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foo", record.get_all());
    record.get_capture("foo").unwrap_err();
    record.get_capture("bar").unwrap_err();
}

#[test]
fn identifier_two_times() {
    let calc_regex = generate! {
        foo  = "foo";
        bar := foo;
        baz := bar;
    };
    let mut reader = $get_reader("foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foo", record.get_all());
    assert_eq!(b"foo", record.get_capture("bar").unwrap());
    assert_eq!(b"foo", record.get_capture("bar.foo").unwrap());
    record.get_capture("foo").unwrap_err();
    record.get_capture("baz").unwrap_err();
}

#[test]
fn parentheses() {
    let calc_regex = generate! {
        foo := "foo";
        bar := (foo);
    };
    let mut reader = $get_reader("foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foo", record.get_all());
    assert_eq!(b"foo", record.get_all());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    record.get_capture("bar").unwrap_err();
}

#[test]
fn concatenate_regex() {
    let calc_regex = generate! {
        foo         = "foo";
        bar         = "bar";
        calc_regex := foo, bar;
    };
    let mut reader = $get_reader("foobar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foobar", record.get_all());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    assert_eq!(b"bar", record.get_capture("bar").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn concatenate_regex_same() {
    let calc_regex = generate! {
        foo         = "foo" | "bar";
        calc_regex := foo, foo;
    };
    let mut reader = $get_reader("foobar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foobar", record.get_all());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    assert_eq!(b"bar", record.get_capture("foo'").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn concatenate_three_different() {
    let calc_regex = generate! {
        foo        := "foo";
        bar        := "bar";
        baz        := "baz";
        calc_regex := foo, bar, baz;
    };
    let mut reader = $get_reader("foobarbaz".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foobarbaz", record.get_all());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    assert_eq!(b"bar", record.get_capture("bar").unwrap());
    assert_eq!(b"baz", record.get_capture("baz").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn concatenate_regex_mixed_anonymous() {
    let calc_regex = generate! {
        foo         = "foo";
        baz         = "baz";
        calc_regex := foo, "bar", baz;
    };
    let mut reader = $get_reader("foobarbaz".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foobarbaz", record.get_all());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    record.get_capture("bar").unwrap_err();
    assert_eq!(b"baz", record.get_capture("baz").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

///////////////////////////////////////////////////////////////////////////////
//      Repeat
///////////////////////////////////////////////////////////////////////////////

#[test]
fn repeat_regex() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte^3;
    };
    let mut reader = $get_reader(&[0u8, 42u8, 255u8][..]);
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(&[0u8, 42u8, 255u8][..], record.get_all());
    assert_eq!(&[0u8][..], record.get_capture("byte[0]").unwrap());
    assert_eq!(&[42u8][..], record.get_capture("byte[1]").unwrap());
    assert_eq!(&[255u8][..], record.get_capture("byte[2]").unwrap());
}

#[test]
fn repeat_regex_get_captures() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte^3;
    };
    let mut reader = $get_reader(&[0u8, 42u8, 255u8][..]);
    let record = reader.parse(&calc_regex).unwrap();
    let mut captures_iter = record.get_captures("byte").unwrap();
    assert_eq!(&[0u8][..], captures_iter.next().unwrap());
    assert_eq!(&[42u8][..], captures_iter.next().unwrap());
    assert_eq!(&[255u8][..], captures_iter.next().unwrap());
    assert!(captures_iter.next().is_none());
}

#[test]
fn repeat_regex_anonymous() {
    let calc_regex = generate! {
        calc_regex := "foo"^3;
    };
    let mut reader = $get_reader("foofoofoo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foofoofoo", record.get_all());
    assert!(record.capture_is_empty());
}

#[test]
fn repeat_concatenate_anonymous() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := (byte, ",")^3;
    };
    let mut reader = $get_reader("a,b,c,".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"a,b,c,", record.get_all());
    // assert_eq!(b"a", record.get_capture("byte[0]").unwrap());
    // assert_eq!(b"b", record.get_capture("byte[1]").unwrap());
    // assert_eq!(b"c", record.get_capture("byte[2]").unwrap());
    assert!(record.capture_is_empty());
}

#[test]
fn repeat_concatenate_identifier() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        repeat     := byte, ",";
        calc_regex := repeat^3;
    };
    let mut reader = $get_reader("a,b,c,".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"a,b,c,", record.get_all());
    assert_eq!(b"a,", record.get_capture("repeat[0]").unwrap());
    assert_eq!(b"b,", record.get_capture("repeat[1]").unwrap());
    assert_eq!(b"c,", record.get_capture("repeat[2]").unwrap());
    assert_eq!(b"a", record.get_capture("repeat[0].byte").unwrap());
    assert_eq!(b"b", record.get_capture("repeat[1].byte").unwrap());
    assert_eq!(b"c", record.get_capture("repeat[2].byte").unwrap());
}

#[test]
fn repeat_multiple() {
    let calc_regex = generate! {
        lower       = "a" - "z";
        upper       = "A" - "Z";
        calc_regex := lower^2, upper^3;
    };
    let mut reader = $get_reader("abCDE".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abCDE", record.get_all());
    assert_eq!(b"a", record.get_capture("lower[0]").unwrap());
    assert_eq!(b"b", record.get_capture("lower[1]").unwrap());
    assert_eq!(b"C", record.get_capture("upper[0]").unwrap());
    assert_eq!(b"D", record.get_capture("upper[1]").unwrap());
    assert_eq!(b"E", record.get_capture("upper[2]").unwrap());
}

#[test]
fn repeat_multiple_same() {
    let calc_regex = generate! {
        lower       = "a" - "z";
        calc_regex := lower^2, lower^3;
    };
    let mut reader = $get_reader("abcde".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abcde", record.get_all());
    assert_eq!(b"a", record.get_capture("lower[0]").unwrap());
    assert_eq!(b"b", record.get_capture("lower[1]").unwrap());
    assert_eq!(b"c", record.get_capture("lower'[0]").unwrap());
    assert_eq!(b"d", record.get_capture("lower'[1]").unwrap());
    assert_eq!(b"e", record.get_capture("lower'[2]").unwrap());
}

#[test]
fn repeat_multiple_mixed() {
    let calc_regex = generate! {
        lower       = "a" - "z";
        upper       = "A" - "Z";
        calc_regex := lower^2, upper^2, lower^3;
    };
    let mut reader = $get_reader("abCDefg".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abCDefg", record.get_all());
    assert_eq!(b"a", record.get_capture("lower[0]").unwrap());
    assert_eq!(b"b", record.get_capture("lower[1]").unwrap());
    assert_eq!(b"C", record.get_capture("upper[0]").unwrap());
    assert_eq!(b"D", record.get_capture("upper[1]").unwrap());
    assert_eq!(b"e", record.get_capture("lower'[0]").unwrap());
    assert_eq!(b"f", record.get_capture("lower'[1]").unwrap());
    assert_eq!(b"g", record.get_capture("lower'[2]").unwrap());
}

#[test]
fn repeat_multiple_anonymous() {
    let calc_regex = generate! {
        calc_regex := ("a" - "z")^2, ("A" - "Z")^3;
    };
    let mut reader = $get_reader("abCDE".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abCDE", record.get_all());
    assert!(record.capture_is_empty());
}

#[test]
fn repeat_multiple_same_parentheses() {
    let calc_regex = generate! {
        lower       = "a" - "z";
        calc_regex := (lower)^2, (lower)^3;
    };
    let mut reader = $get_reader("abcde".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abcde", record.get_all());
    // assert_eq!(b"a", record.get_capture("lower[0]").unwrap());
    // assert_eq!(b"b", record.get_capture("lower[1]").unwrap());
    // assert_eq!(b"c", record.get_capture("lower'[0]").unwrap());
    // assert_eq!(b"d", record.get_capture("lower'[1]").unwrap());
    // assert_eq!(b"e", record.get_capture("lower'[2]").unwrap());
    assert!(record.capture_is_empty());
}

#[test]
fn repeat_multiple_nested() {
    let calc_regex = generate! {
        lower       = "a" - "z";
        upper       = "A" - "Z";
        calc_regex := (lower^3)^2, (upper^2)^3;
    };
    let mut reader = $get_reader("abcdefABCDEF".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abcdefABCDEF", record.get_all());
    // assert_eq!(b"a", record.get_capture("lower[0][0]").unwrap());
    // assert_eq!(b"b", record.get_capture("lower[0][1]").unwrap());
    // assert_eq!(b"c", record.get_capture("lower[0][2]").unwrap());
    // assert_eq!(b"d", record.get_capture("lower[1][0]").unwrap());
    // assert_eq!(b"e", record.get_capture("lower[1][1]").unwrap());
    // assert_eq!(b"f", record.get_capture("lower[1][2]").unwrap());
    // assert_eq!(b"A", record.get_capture("upper[0][0]").unwrap());
    // assert_eq!(b"B", record.get_capture("upper[0][1]").unwrap());
    // assert_eq!(b"C", record.get_capture("upper[1][0]").unwrap());
    // assert_eq!(b"D", record.get_capture("upper[1][1]").unwrap());
    // assert_eq!(b"E", record.get_capture("upper[2][0]").unwrap());
    // assert_eq!(b"F", record.get_capture("upper[2][1]").unwrap());
    assert!(record.capture_is_empty());
}

#[test]
fn repeat_multiple_nested_same() {
    let calc_regex = generate! {
        lower       = "a" - "z";
        calc_regex := (lower^3)^2, (lower^2)^3;
    };
    let mut reader = $get_reader("abcdefabcdef".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abcdefabcdef", record.get_all());
    // assert_eq!(b"a", record.get_capture("lower[0][0]").unwrap());
    // assert_eq!(b"b", record.get_capture("lower[0][1]").unwrap());
    // assert_eq!(b"c", record.get_capture("lower[0][2]").unwrap());
    // assert_eq!(b"d", record.get_capture("lower[1][0]").unwrap());
    // assert_eq!(b"e", record.get_capture("lower[1][1]").unwrap());
    // assert_eq!(b"f", record.get_capture("lower[1][2]").unwrap());
    // assert_eq!(b"a", record.get_capture("lower'[0][0]").unwrap());
    // assert_eq!(b"b", record.get_capture("lower'[0][1]").unwrap());
    // assert_eq!(b"c", record.get_capture("lower'[1][0]").unwrap());
    // assert_eq!(b"d", record.get_capture("lower'[1][1]").unwrap());
    // assert_eq!(b"e", record.get_capture("lower'[2][0]").unwrap());
    // assert_eq!(b"f", record.get_capture("lower'[2][1]").unwrap());
    assert!(record.capture_is_empty());
}

#[test]
fn repeat_nested_multiple_same() {
    let calc_regex = generate! {
        lower       = "a" - "z";
        calc_regex := (lower^2, lower^3)^2;
    };
    let mut reader = $get_reader("abcdeabcde".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abcdeabcde", record.get_all());
    // assert_eq!(b"a", record.get_capture("lower[0][0]").unwrap());
    // assert_eq!(b"b", record.get_capture("lower[0][1]").unwrap());
    // assert_eq!(b"c", record.get_capture("lower'[0][0]").unwrap());
    // assert_eq!(b"d", record.get_capture("lower'[0][1]").unwrap());
    // assert_eq!(b"e", record.get_capture("lower'[0][2]").unwrap());
    // assert_eq!(b"a", record.get_capture("lower[1][0]").unwrap());
    // assert_eq!(b"b", record.get_capture("lower[1][1]").unwrap());
    // assert_eq!(b"c", record.get_capture("lower'[1][0]").unwrap());
    // assert_eq!(b"d", record.get_capture("lower'[1][1]").unwrap());
    // assert_eq!(b"e", record.get_capture("lower'[1][2]").unwrap());
    assert!(record.capture_is_empty());
}

#[test]
fn repeat_nested_multiple_mixed() {
    let calc_regex = generate! {
        lower       = "a" - "z";
        upper       = "A" - "Z";
        calc_regex := (lower^2, upper^2, lower^3)^2;
    };
    let mut reader = $get_reader("abCDefgabCDefg".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abCDefgabCDefg", record.get_all());
    // assert_eq!(b"a", record.get_capture("lower[0][0]").unwrap());
    // assert_eq!(b"b", record.get_capture("lower[0][1]").unwrap());
    // assert_eq!(b"C", record.get_capture("upper[0][0]").unwrap());
    // assert_eq!(b"D", record.get_capture("upper[0][1]").unwrap());
    // assert_eq!(b"e", record.get_capture("lower'[0][0]").unwrap());
    // assert_eq!(b"f", record.get_capture("lower'[0][1]").unwrap());
    // assert_eq!(b"g", record.get_capture("lower'[0][2]").unwrap());
    // assert_eq!(b"a", record.get_capture("lower[1][0]").unwrap());
    // assert_eq!(b"b", record.get_capture("lower[1][1]").unwrap());
    // assert_eq!(b"C", record.get_capture("upper[1][0]").unwrap());
    // assert_eq!(b"D", record.get_capture("upper[1][1]").unwrap());
    // assert_eq!(b"e", record.get_capture("lower'[1][0]").unwrap());
    // assert_eq!(b"f", record.get_capture("lower'[1][1]").unwrap());
    // assert_eq!(b"g", record.get_capture("lower'[1][2]").unwrap());
    assert!(record.capture_is_empty());
}

#[test]
fn repeat_multiple_anonymous_nested() {
    let calc_regex = generate! {
        calc_regex := (("a" - "z")^3)^2, (("A" - "Z")^2)^3;
    };
    let mut reader = $get_reader("abcdefABCDEF".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"abcdefABCDEF", record.get_all());
    record.get_capture("calc_regex").unwrap_err();
    assert!(record.capture_is_empty());
}

///////////////////////////////////////////////////////////////////////////////
//      Length Count
///////////////////////////////////////////////////////////////////////////////

#[test]
fn length_count() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        digit       = "0" - "9";
        calc_regex := digit.decimal, foo#decimal;
    };
    let mut reader = $get_reader("3foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"3foo", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    assert_eq!(b"foo", record.get_capture("$value").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn length_count_empty() {
    let calc_regex = generate! {
        foo         = "o"*;
        digit       = "0" - "9";
        calc_regex := digit.decimal, foo#decimal;
    };
    let mut reader = $get_reader("0".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"0", record.get_all());
    assert_eq!(b"0", record.get_capture("digit").unwrap());
    assert_eq!(b"0", record.get_capture("$count").unwrap());
    assert_eq!(b"", record.get_capture("foo").unwrap());
    assert_eq!(b"", record.get_capture("$value").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn length_count_invalid_count() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        digit       = "0" - "9";
        calc_regex := digit.decimal, foo#decimal;
    };
    let mut reader = $get_reader("afoo".as_bytes());
    let err = reader.parse(&calc_regex).unwrap_err();
    if let ParserError::Regex { ref regex, ref value } = err {
        assert_eq!(regex, "^(?-u:[0-9])$");
        assert_eq!(value, b"a");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn length_count_invalid_count_match() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        digit       = "a";
        calc_regex := digit.decimal, foo#decimal;
    };
    let mut reader = $get_reader("afoo".as_bytes());
    let err = reader.parse(&calc_regex).unwrap_err();
    if let ParserError::CannotReadCount { ref raw_count } = err {
        assert_eq!(raw_count, b"a");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn length_count_s() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        bar         = "bar";
        digit       = "0" - "9";
        calc_regex := digit.decimal, bar, foo#decimal;
    };
    let mut reader = $get_reader("3barfoo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"3barfoo", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    assert_eq!(b"foo", record.get_capture("$value").unwrap());
    assert_eq!(b"bar", record.get_capture("bar").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn length_count_calc_regex() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        bar        := foo;
        digit       = "0" - "9";
        calc_regex := digit.decimal, bar#decimal;
    };
    let mut reader = $get_reader("3foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"3foo", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"foo", record.get_capture("bar").unwrap());
    assert_eq!(b"foo", record.get_capture("$value").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn length_count_kleene_star() {
    let calc_regex = generate! {
        foo         = "foo";
        digit       = "0" - "9";
        calc_regex := digit.decimal, (foo*)#decimal;
    };
    let mut reader = $get_reader("9foofoofoo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"9foofoofoo", record.get_all());
    assert_eq!(b"9", record.get_capture("digit").unwrap());
    assert_eq!(b"9", record.get_capture("$count").unwrap());
    assert_eq!(b"foo", record.get_capture("foo[0]").unwrap());
    assert_eq!(b"foo", record.get_capture("foo[1]").unwrap());
    assert_eq!(b"foo", record.get_capture("foo[2]").unwrap());
    assert_eq!(b"foofoofoo", record.get_capture("$value").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn length_count_anonymous_regex() {
    let calc_regex = generate! {
        calc_regex := ("0" - "9").decimal, "foo" | "bar", ("o"*)#decimal;
    };
    let mut reader = $get_reader("3fooooo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"3fooooo", record.get_all());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"ooo", record.get_capture("$value").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn length_count_anonymous_calc_regex() {
    let calc_regex = generate! {
        calc_regex := (("0" - "9")^3).decimal,
                      "foo" | "bar" , "baz",
                      ("f", "o"*)#decimal;
    };
    let mut reader = $get_reader("003barbazfoo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"003barbazfoo", record.get_all());
    assert_eq!(b"003", record.get_capture("$count").unwrap());
    assert_eq!(b"foo", record.get_capture("$value").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn concatenate_length_count() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        digit       = "0" - "9";
        calc_regex := "foo", digit.decimal, foo#decimal, "bar";
    };
    let mut reader = $get_reader("foo3foobar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foo3foobar", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    assert_eq!(b"foo", record.get_capture("$value").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn concatenate_length_count_s() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        bar         = "bar";
        digit       = "0" - "9";
        calc_regex := "foo", digit.decimal, bar, foo#decimal, "bar";
    };
    let mut reader = $get_reader("foo3barfoobar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foo3barfoobar", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"bar", record.get_capture("bar").unwrap());
    assert_eq!(b"foo", record.get_capture("foo").unwrap());
    assert_eq!(b"foo", record.get_capture("$value").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

///////////////////////////////////////////////////////////////////////////////
//      Occurrence Count
///////////////////////////////////////////////////////////////////////////////

#[test]
fn occurrence_count() {
    let calc_regex = generate! {
        foo         = ("a" - "z")^3;
        digit       = "0" - "9";
        calc_regex := digit.decimal, foo^decimal;
    };
    let mut reader = $get_reader("3foobarbaz".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"3foobarbaz", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"foobarbaz", record.get_capture("$value").unwrap());
    assert_eq!(b"foo", record.get_capture("foo[0]").unwrap());
    assert_eq!(b"bar", record.get_capture("foo[1]").unwrap());
    assert_eq!(b"baz", record.get_capture("foo[2]").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn occurrence_count_empty() {
    let calc_regex = generate! {
        foo         = ("a" - "z")^3;
        digit       = "0" - "9";
        calc_regex := digit.decimal, foo^decimal;
    };
    let mut reader = $get_reader("0".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"0", record.get_all());
    assert_eq!(b"0", record.get_capture("digit").unwrap());
    assert_eq!(b"0", record.get_capture("$count").unwrap());
    assert_eq!(b"", record.get_capture("$value").unwrap());
    let err = record.get_capture("foo[0]").unwrap_err();
    if let NameError::NoSuchName { ref name } = err {
        assert_eq!(name, "foo");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn occurrence_count_s() {
    let calc_regex = generate! {
        foo         = "f" | "o";
        bar         = "bar";
        digit       = "0" - "9";
        calc_regex := digit.decimal, bar, foo^decimal;
    };
    let mut reader = $get_reader("3barfoo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"3barfoo", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"foo", record.get_capture("$value").unwrap());
    assert_eq!(b"bar", record.get_capture("bar").unwrap());
    assert_eq!(b"f", record.get_capture("foo[0]").unwrap());
    assert_eq!(b"o", record.get_capture("foo[1]").unwrap());
    assert_eq!(b"o", record.get_capture("foo[2]").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn concatenate_occurrence_count() {
    let calc_regex = generate! {
        foo         = "f" | "o";
        digit       = "0" - "9";
        calc_regex := "foo", digit.decimal, foo^decimal, "bar";
    };
    let mut reader = $get_reader("foo3foobar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foo3foobar", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"foo", record.get_capture("$value").unwrap());
    assert_eq!(b"f", record.get_capture("foo[0]").unwrap());
    assert_eq!(b"o", record.get_capture("foo[1]").unwrap());
    assert_eq!(b"o", record.get_capture("foo[2]").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn concatenate_occurrence_count_s() {
    let calc_regex = generate! {
        foo         = "f" | "o";
        bar         = "bar";
        digit       = "0" - "9";
        calc_regex := "foo", digit.decimal, bar, foo^decimal, "bar";
    };
    let mut reader = $get_reader("foo3barfoobar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"foo3barfoobar", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"bar", record.get_capture("bar").unwrap());
    assert_eq!(b"foo", record.get_capture("$value").unwrap());
    assert_eq!(b"f", record.get_capture("foo[0]").unwrap());
    assert_eq!(b"o", record.get_capture("foo[1]").unwrap());
    assert_eq!(b"o", record.get_capture("foo[2]").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

#[test]
fn occurrence_count_calc_regex() {
    let calc_regex = generate! {
        foo         = ("a" - "z")^3;
        bar        := foo;
        digit       = "0" - "9";
        calc_regex := digit.decimal, bar^decimal;
    };
    let mut reader = $get_reader("3foobarbaz".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"3foobarbaz", record.get_all());
    assert_eq!(b"3", record.get_capture("digit").unwrap());
    assert_eq!(b"3", record.get_capture("$count").unwrap());
    assert_eq!(b"foobarbaz", record.get_capture("$value").unwrap());
    assert_eq!(b"foo", record.get_capture("bar[0]").unwrap());
    assert_eq!(b"bar", record.get_capture("bar[1]").unwrap());
    assert_eq!(b"baz", record.get_capture("bar[2]").unwrap());
    record.get_capture("calc_regex").unwrap_err();
}

///////////////////////////////////////////////////////////////////////////////
//      Nested
///////////////////////////////////////////////////////////////////////////////

#[test]
fn length_count_in_occurrence_count() {
    let calc_regex = generate! {
        digit       = "0" - "9";
        chars       = ("a" - "z")*;
        inner      := digit.decimal, chars#decimal;
        calc_regex := digit.decimal, inner^decimal;
    };
    let mut reader = $get_reader("23foo4baar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"2", record.get_capture("digit").unwrap());
    assert_eq!(b"2", record.get_capture("$count").unwrap());
    assert_eq!(b"3foo4baar", record.get_capture("$value").unwrap());
    assert_eq!(b"3foo", record.get_capture("inner[0]").unwrap());
    assert_eq!(b"4baar", record.get_capture("inner[1]").unwrap());
    assert_eq!(b"3", record.get_capture("inner[0].$count").unwrap());
    assert_eq!(b"3", record.get_capture("inner[0].digit").unwrap());
    assert_eq!(b"foo", record.get_capture("inner[0].chars").unwrap());
    assert_eq!(b"foo", record.get_capture("inner[0].$value").unwrap());
    assert_eq!(b"4", record.get_capture("inner[1].$count").unwrap());
    assert_eq!(b"4", record.get_capture("inner[1].digit").unwrap());
    assert_eq!(b"baar", record.get_capture("inner[1].chars").unwrap());
    assert_eq!(b"baar", record.get_capture("inner[1].$value").unwrap());
}

#[test]
fn length_count_s_in_occurrence_count() {
    let calc_regex = generate! {
        digit       = "0" - "9";
        chars       = ("a" - "z")*;
        inner      := digit.decimal, "foo", chars#decimal;
        calc_regex := digit.decimal, inner^decimal;
    };
    let mut reader = $get_reader("23foofoo4foobaar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"2", record.get_capture("digit").unwrap());
    assert_eq!(b"2", record.get_capture("$count").unwrap());
    assert_eq!(b"3foofoo4foobaar", record.get_capture("$value").unwrap());
    assert_eq!(b"3foofoo", record.get_capture("inner[0]").unwrap());
    assert_eq!(b"4foobaar", record.get_capture("inner[1]").unwrap());
    assert_eq!(b"3", record.get_capture("inner[0].$count").unwrap());
    assert_eq!(b"3", record.get_capture("inner[0].digit").unwrap());
    assert_eq!(b"foo", record.get_capture("inner[0].chars").unwrap());
    assert_eq!(b"foo", record.get_capture("inner[0].$value").unwrap());
    assert_eq!(b"4", record.get_capture("inner[1].$count").unwrap());
    assert_eq!(b"4", record.get_capture("inner[1].digit").unwrap());
    assert_eq!(b"baar", record.get_capture("inner[1].chars").unwrap());
    assert_eq!(b"baar", record.get_capture("inner[1].$value").unwrap());
}

#[test]
fn nested_length_count_s() {
    let calc_regex = generate! {
        digit       = "0" - "9";
        chars       = ("a" - "z")*;
        inner      := digit.decimal, "foo", chars#decimal;
        calc_regex := digit.decimal, "bar", inner#decimal;
    };
    let mut reader = $get_reader("6bar2foofo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"6", record.get_capture("digit").unwrap());
    assert_eq!(b"6", record.get_capture("$count").unwrap());
    assert_eq!(b"2foofo", record.get_capture("$value").unwrap());
    assert_eq!(b"2foofo", record.get_capture("inner").unwrap());
    assert_eq!(b"2", record.get_capture("inner.$count").unwrap());
    assert_eq!(b"2", record.get_capture("inner.digit").unwrap());
    assert_eq!(b"fo", record.get_capture("inner.chars").unwrap());
    assert_eq!(b"fo", record.get_capture("inner.$value").unwrap());
}

#[test]
fn occurrence_count_in_length_count() {
    let calc_regex = generate! {
        digit       = "0" - "9";
        lower_char  = "a" - "z";
        inner      := digit.decimal, lower_char^decimal;
        calc_regex := digit.decimal, inner#decimal;
    };
    let mut reader = $get_reader("43foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"4", record.get_capture("digit").unwrap());
    assert_eq!(b"4", record.get_capture("$count").unwrap());
    assert_eq!(b"3foo", record.get_capture("$value").unwrap());
    assert_eq!(b"3foo", record.get_capture("$value").unwrap());
    assert_eq!(b"3", record.get_capture("inner.$count").unwrap());
    assert_eq!(b"foo", record.get_capture("inner.$value").unwrap());
    assert_eq!(b"f", record.get_capture("inner.lower_char[0]").unwrap());
    assert_eq!(b"o", record.get_capture("inner.lower_char[1]").unwrap());
    assert_eq!(b"o", record.get_capture("inner.lower_char[2]").unwrap());
}

#[test]
fn occurrence_count_s_in_length_count() {
    let calc_regex = generate! {
        digit       = "0" - "9";
        lower_char  = "a" - "z";
        inner      := digit.decimal, "bar", lower_char^decimal;
        calc_regex := digit.decimal, inner#decimal;
    };
    let mut reader = $get_reader("73barfoo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"7", record.get_capture("digit").unwrap());
    assert_eq!(b"7", record.get_capture("$count").unwrap());
    assert_eq!(b"3barfoo", record.get_capture("$value").unwrap());
    assert_eq!(b"3barfoo", record.get_capture("$value").unwrap());
    assert_eq!(b"3", record.get_capture("inner.$count").unwrap());
    assert_eq!(b"foo", record.get_capture("inner.$value").unwrap());
    assert_eq!(b"f", record.get_capture("inner.lower_char[0]").unwrap());
    assert_eq!(b"o", record.get_capture("inner.lower_char[1]").unwrap());
    assert_eq!(b"o", record.get_capture("inner.lower_char[2]").unwrap());
}

#[test]
fn repeated_occurrence_count_in_length_count() {
    let calc_regex = generate! {
        digit       = "0" - "9";
        lower_char  = "a" - "z";
        inner      := digit.decimal, lower_char^decimal;
        two_inner  := inner^2;
        calc_regex := digit.decimal, two_inner#decimal;
    };
    let mut reader = $get_reader("93foo4baar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    assert_eq!(b"9", record.get_capture("digit").unwrap());
    assert_eq!(b"9", record.get_capture("$count").unwrap());
    assert_eq!(b"3foo4baar", record.get_capture("$value").unwrap());
    assert_eq!(b"3foo4baar", record.get_capture("$value").unwrap());
    assert_eq!(b"3", record.get_capture("two_inner.inner[0].$count").unwrap());
    assert_eq!(
        b"foo",
        record.get_capture("two_inner.inner[0].$value").unwrap()
    );
    assert_eq!(
        b"f",
        record.get_capture("two_inner.inner[0].lower_char[0]").unwrap()
    );
    assert_eq!(
        b"o",
        record.get_capture("two_inner.inner[0].lower_char[1]").unwrap()
    );
    assert_eq!(
        b"o",
        record.get_capture("two_inner.inner[0].lower_char[2]").unwrap()
    );
    assert_eq!(b"4", record.get_capture("two_inner.inner[1].$count").unwrap());
    assert_eq!(
        b"baar",
        record.get_capture("two_inner.inner[1].$value").unwrap()
    );
    assert_eq!(
        b"b",
        record.get_capture("two_inner.inner[1].lower_char[0]").unwrap()
    );
    assert_eq!(
        b"a",
        record.get_capture("two_inner.inner[1].lower_char[1]").unwrap()
    );
    assert_eq!(
        b"a",
        record.get_capture("two_inner.inner[1].lower_char[2]").unwrap()
    );
    assert_eq!(
        b"r",
        record.get_capture("two_inner.inner[1].lower_char[3]").unwrap()
    );
}

///////////////////////////////////////////////////////////////////////////////
//      Erroneous Capture Access
///////////////////////////////////////////////////////////////////////////////

#[test]
fn non_existent_single_capture() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte;
    };
    let mut reader = $get_reader(&[0u8][..]);
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_capture("foo").unwrap_err();
    assert_eq!(
        format!("{}", err),
        "No node named \"foo\" exists."
    );
    if let NameError::NoSuchName{ ref name } = err {
        assert_eq!(name, "foo");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn non_existent_repeat_capture() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte^3;
    };
    let mut reader = $get_reader(&[0u8, 42u8, 255u8][..]);
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_capture("foo[0]").unwrap_err();
    assert_eq!(
        format!("{}", err),
        "No node named \"foo\" exists."
    );
    if let NameError::NoSuchName{ ref name } = err {
        assert_eq!(name, "foo");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn repeat_out_of_bounds() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte^3;
    };
    let mut reader = $get_reader(&[0u8, 42u8, 255u8][..]);
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_capture("byte[3]").unwrap_err();
    assert_eq!(
        format!("{}", err),
        "Tried to get element number 3 of \"byte\", but only 3 elements exist."
    );
    if let NameError::OutOfBounds{ ref name, index, len } = err {
        assert_eq!(name, "byte");
        assert_eq!(index, 3);
        assert_eq!(len, 3);
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn repeat_not_indexed() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte^3;
    };
    let mut reader = $get_reader(&[0u8, 42u8, 255u8][..]);
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_capture("byte").unwrap_err();
    assert_eq!(
        format!("{}", err),
        "Tried to access single capture on repeat capture \"byte\"."
    );
    if let NameError::MisplacedSingleAccess { ref name } = err {
        assert_eq!(name, "byte");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn indexed_single_node() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte;
    };
    let mut reader = $get_reader(&[0u8][..]);
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_capture("byte[0]").unwrap_err();
    assert_eq!(
        format!("{}", err),
        "Tried to access repeat capture on single capture \"byte\"."
    );
    if let NameError::MisplacedRepeatAccess { ref name } = err {
        assert_eq!(name, "byte");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn missing_closing_bracket() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte^3;
    };
    let mut reader = $get_reader(&[0u8, 42u8, 255u8][..]);
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_capture("byte[2").unwrap_err();
    assert_eq!(
        format!("{}", err),
        "The given capture name is invalid: missing closing ']'."
    );
    if let NameError::InvalidCaptureName{ message } = err {
        assert_eq!(message, "missing closing ']'");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn non_numeric_index() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte^3;
    };
    let mut reader = $get_reader(&[0u8, 42u8, 255u8][..]);
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_capture("byte[a]").unwrap_err();
    assert_eq!(
        format!("{}", err),
        "The given capture name is invalid: non-numeric index."
    );
    if let NameError::InvalidCaptureName{ message } = err {
        assert_eq!(message, "non-numeric index");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

// End of macro-instantiated module.
        }
    }
}

run_tests!(stream, Reader::from_stream);
run_tests!(array, Reader::from_array);
