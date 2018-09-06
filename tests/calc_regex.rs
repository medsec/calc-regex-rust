//! Tests parsing `CalcRegex`es that are generated with the `generate!` macro
//! from a reader, like an external crate would use this library.

use std::str;

#[macro_use(generate)]
extern crate calc_regex;

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
fn simple_regex() {
    let re: calc_regex::CalcRegex = generate! {
        re := "foo";
    };
    let mut reader = calc_regex::Reader::from_array(b"foo");
    let record = reader.parse(&re).unwrap();
    let expected = b"foo";
    let actual = record.get_all();
    assert_eq!(expected, actual);
}

#[test]
fn repeat_foo() {
    let re = generate! {
        foo = "foo";
        re := foo^3;
    };
    let mut reader = calc_regex::Reader::from_array(b"foofoofoo");
    let record = reader.parse(&re).unwrap();

    let expected = b"foofoofoo";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("foo[0]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("foo[1]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("foo[2]").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn repeat_regex() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        word      = character^3;
        re       := word^2;
    };
    let mut reader = calc_regex::Reader::from_array(b"FooBar");
    let record = reader.parse(&re).unwrap();

    let expected = b"FooBar";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let expected = b"Foo";
    let actual = record.get_capture("word[0]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"Bar";
    let actual = record.get_capture("word[1]").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn repeat_repeat() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        word     := character^3;
        re       := word^2;
    };
    let mut reader = calc_regex::Reader::from_array(b"FooBar");
    let record = reader.parse(&re).unwrap();

    let expected = b"FooBar";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let expected = b"Foo";
    let actual = record.get_capture("word[0]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"Bar";
    let actual = record.get_capture("word[1]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"F";
    let actual = record.get_capture("word[0].character[0]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"o";
    let actual = record.get_capture("word[0].character[1]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"o";
    let actual = record.get_capture("word[0].character[2]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"B";
    let actual = record.get_capture("word[1].character[0]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"a";
    let actual = record.get_capture("word[1].character[1]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"r";
    let actual = record.get_capture("word[1].character[2]").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn repeat_regex_oneline() {
    let re = generate! {
        re := (("a"-"z"|"A"-"Z")^3)^2;
    };
    let mut reader = calc_regex::Reader::from_array(b"FooBar");
    let record = reader.parse(&re).unwrap();

    let expected = b"FooBar";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    // let expected = b"Foo";
    // let actual = record.get_capture("0").unwrap();
    // assert_eq!(expected, actual);

    // let expected = b"Bar";
    // let actual = record.get_capture("1").unwrap();
    // assert_eq!(expected, actual);
}

#[test]
fn repeat_regex_bounded() {
    let re = generate! {
        byte      = %0 - %FF;
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := character^3;
        re       := number.decimal, (value, byte*)#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"5:Fooxy");
    let record = reader.parse(&re).unwrap();

    let expected = b"5:Fooxy";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let expected = b"Foo";
    let actual = record.get_capture("value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"F";
    let actual = record.get_capture("value.character[0]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"o";
    let actual = record.get_capture("value.character[1]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"o";
    let actual = record.get_capture("value.character[2]").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn repeat_regex_bounded_exeeded() {
    let re = generate! {
        byte      = %0 - %FF;
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := character^3;
        re       := number.decimal, (value, byte*)#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"2:Fooxy");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::Regex { .. } = err {
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn repeat_regex_exact() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := character^3;
        re       := number.decimal, value#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"3:Foo");
    let record = reader.parse(&re).unwrap();

    let expected = b"3:Foo";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let expected = b"Foo";
    let actual = record.get_capture("value").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn repeat_regex_exact_exceeded() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := character^3;
        re       := number.decimal, value#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"2:Foo");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::Regex { .. } = err {
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn repeat_regex_exact_too_short() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := character^3;
        re       := number.decimal, value#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"4:Foo");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::ConflictingBounds { old, new } = err {
        assert_eq!(old, 2);
        assert_eq!(new, 1);
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn occurrence_count_bounded() {
    let re = generate! {
        byte      = %0 - %FF;
        character = "a" - "z" | "A" - "Z";
        number    = ("0" - "9")*, ":";
        word      = character^3;
        value    := number.decimal, word^decimal;
        re       := number.decimal, (value, byte*)#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"10:2:FooBarXY");
    let record = reader.parse(&re).unwrap();

    let expected = b"10:2:FooBarXY";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let expected = b"2:FooBar";
    let actual = record.get_capture("value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"Foo";
    let actual = record.get_capture("value.word[0]").unwrap();
    assert_eq!(expected, actual);

    let expected = b"Bar";
    let actual = record.get_capture("value.word[1]").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn occurrence_count_bounded_too_long() {
    let re = generate! {
        byte      = %0 - %FF;
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        word      = character^3;
        value    := number.decimal, word^decimal;
        re       := number.decimal, (value, byte*)#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"9:2:FooBar");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::UnexpectedEof = err {
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn occurrence_count_bounded_exceeded() {
    let re = generate! {
        byte      = %0 - %FF;
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        word      = character^3;
        value    := number.decimal, word^decimal;
        re       := number.decimal, (value, byte*)#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"7:2:FooBar");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::Regex { .. } = err {
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn length_count_exact() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := number.decimal, (character*)#decimal;
        re       := number.decimal, value#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"5:3:Foo");
    let record = reader.parse(&re).unwrap();

    let expected = b"5:3:Foo";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let expected = b"3:Foo";
    let actual = record.get_capture("value").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn length_count_exact_exceeded() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := number.decimal, (character*)#decimal;
        re       := number.decimal, value#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"4:3:Foo");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::ConflictingBounds { old, new } = err {
        assert_eq!(old, 2);
        assert_eq!(new, 3);
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn length_count_exact_too_short() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := number.decimal, (character*)#decimal;
        re       := number.decimal, value#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"6:3:Foo");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::ConflictingBounds { old, new } = err {
        assert_eq!(old, 4);
        assert_eq!(new, 3);
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn occurrence_count_exact() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := number.decimal, character^decimal;
        re       := number.decimal, value#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"5:3:Foo");
    let record = reader.parse(&re).unwrap();

    let expected = b"5:3:Foo";
    let actual = record.get_all();
    assert_eq!(expected, actual);

    let expected = b"3:Foo";
    let actual = record.get_capture("value").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn occurrence_count_exact_exceeded() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := number.decimal, character^decimal;
        re       := number.decimal, value#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"4:3:Foo");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::Regex { .. } = err {
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn occurence_count_exact_too_short() {
    let re = generate! {
        character = "a" - "z" | "A" - "Z";
        number    = "0" - "9", ":";
        value    := number.decimal, character^decimal;
        re       := number.decimal, value#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"6:3:Foo");
    let err = reader.parse(&re).unwrap_err();
    if let calc_regex::ParserError::ConflictingBounds { old, new } = err {
        assert_eq!(old, 2);
        assert_eq!(new, 1);
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn concatenated_equal_names() {
    let re = generate! {
        foo = ("a" - "z")^3;
        re := foo, foo, foo;
    };
    let mut reader = calc_regex::Reader::from_array(b"foobarbaz");
    let record = reader.parse(&re).unwrap();

    let expected = b"foo";
    let actual = record.get_capture("foo").unwrap();
    assert_eq!(expected, actual);

    let expected = b"bar";
    let actual = record.get_capture("foo'").unwrap();
    assert_eq!(expected, actual);

    let expected = b"baz";
    let actual = record.get_capture("foo''").unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn anonymous_length_count() {
    let re = generate! {
        re := ("0"-"9", ":").decimal, (("a"-"z")*)#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"3:foo");
    let record = reader.parse(&re).unwrap();

    let expected = b"3:";
    let actual = record.get_capture("$count").unwrap();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("$value").unwrap();
    assert_eq!(expected, actual);
}

#[test]
#[should_panic]
fn anonymous_occurrence_count() {
    let _ = generate! {
        re := ("0"-"9", ":").decimal, ("a"-"z")^decimal;
    };
    // let mut reader = calc_regex::Reader::from_array(b"3:foo");
    // let record = reader.parse(&re).unwrap();
    //
    // let expected = b"3:";
    // let actual = record.get_capture("$count").unwrap();
    // assert_eq!(expected, actual);
    //
    // let expected = b"foo";
    // let actual = record.get_capture("$value").unwrap();
    // assert_eq!(expected, actual);
}

#[test]
fn multiple_anonymous_length_count() {
    let re = generate! {
        re := ("0"-"9", ":").decimal, (("a"-"z")*)#decimal,
              ("0"-"9", ":").decimal, (("a"-"z")*)#decimal;
    };
    let mut reader = calc_regex::Reader::from_array(b"3:foo4:baar");
    let record = reader.parse(&re).unwrap();

    let expected = b"3:";
    let actual = record.get_capture("$count").unwrap();
    assert_eq!(expected, actual);

    let expected = b"foo";
    let actual = record.get_capture("$value").unwrap();
    assert_eq!(expected, actual);

    let expected = b"4:";
    let actual = record.get_capture("$count'").unwrap();
    assert_eq!(expected, actual);

    let expected = b"baar";
    let actual = record.get_capture("$value'").unwrap();
    assert_eq!(expected, actual);
}
