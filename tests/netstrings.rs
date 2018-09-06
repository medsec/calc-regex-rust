//! Test generating and using Netstrings with our library.

#![recursion_limit="128"]

#[macro_use(generate)]
extern crate calc_regex;

use std::str;

/// Parses a bytestring containing a number and a trailing colon in ASCII
/// format to the respective number, discarding the colon.
fn decimal(pf_number: &[u8]) -> Option<usize> {
    let (number, colon) = pf_number.split_at(pf_number.len() - 1);
    if colon != [b':'] {
        return None;
    }
    let number = match str::from_utf8(number) {
        Ok(n) => n,
        Err(_) => return None,
    };
    number.parse::<usize>().ok()
}

#[test]
fn netstring() {
    let netstring = generate! {
        byte          = %0 - %FF;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        netstring    := pf_number.decimal, (byte*)#decimal, ",";
    };

    // assert!(!netstring.is_bounded());

    let mut reader = calc_regex::Reader::from_array(b"3:foo,");
    let record = reader.parse(&netstring).unwrap();

    let expected = b"3:";
    let actual = record.get_capture("pf_number").unwrap();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("$value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"3:foo,";
    let actual = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn netstring_number_as_calc_regex() {
    let netstring = generate! {
        byte          = %0 - %FF;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number       := "0" | (nonzero_digit, digit*);
        pf_number    := number, ":";
        netstring    := pf_number.decimal, (byte*)#decimal, ",";
    };

    let mut reader = calc_regex::Reader::from_array(b"3:foo,");
    let record = reader.parse(&netstring).unwrap();

    let expected = b"3:";
    let actual = record.get_capture("pf_number").unwrap();
    assert_eq!(expected, actual);

    let expected = b"3";
    let actual = record.get_capture("pf_number.number").unwrap();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("$value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"3:foo,";
    let actual = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn bounded_netstring() {
    let mut netstring = generate! {
        byte          = %0 - %FF;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        value         = byte*;
        netstring    := pf_number.decimal, value#decimal, ",";
    };

    netstring.set_length_bound("pf_number", 2).unwrap();
    netstring.set_length_bound("value", 8).unwrap();
    // assert!(netstring.is_bounded());

    let mut reader = calc_regex::Reader::from_array(b"3:foo,");
    let record = reader.parse(&netstring).unwrap();

    let expected = b"3:";
    let actual = record.get_capture("pf_number").unwrap();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"3:foo,";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let mut reader = calc_regex::Reader::from_array(b"9:foofoofoo,");
    let err = reader.parse(&netstring).unwrap_err();
    if let calc_regex::ParserError::ConflictingBounds { old, new } = err {
        assert_eq!(old, 9);
        assert_eq!(new, 8);
    } else {
        panic!("Unexpected error: {:?}", err);
    }

    let mut reader = calc_regex::Reader::from_array(b"12:foofoofoofoo,");
    let err = reader.parse(&netstring).unwrap_err();
    if let calc_regex::ParserError::Regex { value, .. } = err {
        assert_eq!(value, b"12");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn n_netstring() {
    let n_netstring = generate! {
        byte          = %0 - %FF;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        netstring    := pf_number.decimal, (byte*)#decimal, ",";
        n_netstring  := pf_number.decimal, (netstring, byte*)#decimal, ",";
    };

    let mut reader = calc_regex::Reader::from_array(b"8:3:abc,XY,");
    let record = reader.parse(&n_netstring).unwrap();

    let expected = b"3:abc,";
    let actual = record.get_capture("netstring").unwrap();
    assert_eq!(expected, actual);

    let expected = b"abc";
    let actual = record.get_capture("netstring.$value").unwrap();
    assert_eq!(expected, actual);

    let error = calc_regex::Reader::from_array(b"5:9999:")
        .parse(&n_netstring).unwrap_err();
    if let calc_regex::ParserError::ConflictingBounds { old, new } = error {
        assert_eq!(old, 0);
        assert_eq!(new, 9999);
    } else { panic!("Unexpected error: {:?}", error) }
}

#[test]
fn nested_netstrings_by_length() {
    let n_netstring = generate! {
        byte          = %0 - %FF;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        netstring    := pf_number.decimal, (byte*)#decimal, ",";
        n_netstring  := pf_number.decimal, (netstring*)#decimal, ",";
    };

    let mut reader = calc_regex::Reader::from_array(b"11:3:abc,2:de,,");
    let record = reader.parse(&n_netstring).unwrap();

    let expected = b"3:abc,";
    let actual = record.get_capture("netstring[0]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"abc";
    let actual = record.get_capture("netstring[0].$value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"2:de,";
    let actual = record.get_capture("netstring[1]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"de";
    let actual = record.get_capture("netstring[1].$value").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn netstring_sequence_single() {
    let netstring = generate! {
        byte          = %0 - %FF;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        netstring    := pf_number.decimal, (byte*)#decimal, ",";
    };

    let mut reader = calc_regex::Reader::from_array(b"3:foo,");
    for result in reader.parse_many(&netstring) {
        let record = result.unwrap();

        let expected = b"3:";
        let actual = record.get_capture("pf_number").unwrap();
        assert_eq!(expected, actual);

        let expected = b"foo";
        let actual = record.get_capture("$value").unwrap();
        assert_eq!(expected, actual);

        let expected = b"3:foo,";
        let actual = record.get_all();
        assert_eq!(expected, actual);
    }
}

#[test]
fn netstring_sequence_single_stream() {
    let netstring = generate! {
        byte          = %0 - %FF;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        netstring    := pf_number.decimal, (byte*)#decimal, ",";
    };

    let mut reader = calc_regex::Reader::from_stream(b"3:foo,".as_ref());
    for result in reader.parse_many(&netstring) {
        let record = result.unwrap();

        let expected = b"3:";
        let actual = record.get_capture("pf_number").unwrap();
        assert_eq!(expected, actual);

        let expected = b"foo";
        let actual = record.get_capture("$value").unwrap();
        assert_eq!(expected, actual);

        let expected = b"3:foo,";
        let actual = record.get_all();
        assert_eq!(expected, actual);
    }
}

#[test]
fn netstring_sequence_multiple() {
    let netstring = generate! {
        byte          = %0 - %FF;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        netstring    := pf_number.decimal, (byte*)#decimal, ",";
    };

    let mut reader = calc_regex::Reader::from_array(b"3:foo,4:baar,");
    let mut iter = reader.parse_many(&netstring);

    let record = iter.next().unwrap().unwrap();

    let expected = b"3:";
    let actual = record.get_capture("pf_number").unwrap();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("$value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"3:foo,";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let record = iter.next().unwrap().unwrap();

    let expected = b"4:";
    let actual = record.get_capture("pf_number").unwrap();
    assert_eq!(expected, actual);

    let expected = b"baar";
    let actual = record.get_capture("$value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"4:baar,";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    assert!(iter.next().is_none());
}

#[test]
fn netstring_sequence_multiple_stream() {
    let netstring = generate! {
        byte          = %0 - %FF;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        netstring    := pf_number.decimal, (byte*)#decimal, ",";
    };

    let mut reader = calc_regex::Reader::from_stream(
        b"3:foo,4:baar,".as_ref()
    );
    let mut iter = reader.parse_many(&netstring);

    let record = iter.next().unwrap().unwrap();

    let expected = b"3:";
    let actual = record.get_capture("pf_number").unwrap();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("$value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"3:foo,";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let record = iter.next().unwrap().unwrap();

    let expected = b"4:";
    let actual = record.get_capture("pf_number").unwrap();
    assert_eq!(expected, actual);

    let expected = b"baar";
    let actual = record.get_capture("$value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"4:baar,";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    assert!(iter.next().is_none());
}
