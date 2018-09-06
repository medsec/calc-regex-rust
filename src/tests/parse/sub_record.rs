
/// Defines tests for a generic reader.
///
/// All tests are run for each reader that is given via an invocation of this
/// macro.
macro_rules! run_tests {
    ($name:ident, $get_reader:path) => {
        pub mod $name {
            use ::*;

// Start of macro-instantiated module.

#[test]
fn simple_regex() {
    let calc_regex = generate! {
        foo := "foo";
        bar := foo;
    };
    let mut reader = $get_reader("foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let sub_record = record.get_sub_record("foo").unwrap();
    assert_eq!(sub_record.get_all(), b"foo");
}

#[test]
fn regex_nested() {
    let calc_regex = generate! {
        foo := "foo";
        bar := foo;
        baz := bar;
    };
    let mut reader = $get_reader("foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let sub_record = record.get_sub_record("bar").unwrap();
    assert_eq!(sub_record.get_capture("foo").unwrap(), b"foo");
}

#[test]
fn regex_nested_twice() {
    let calc_regex = generate! {
        foo := "foo";
        bar := foo;
        baz := bar;
        bazz := baz;
    };
    let mut reader = $get_reader("foo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let sub_record = record.get_sub_record("baz").unwrap();
    let sub_record = sub_record.get_sub_record("bar").unwrap();
    assert_eq!(sub_record.get_capture("foo").unwrap(), b"foo");
}

#[test]
fn repeat() {
    let calc_regex = generate! {
        letter = "a" - "z";
        foo := letter ^ 3;
    };
    let mut reader = $get_reader("abc".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let mut sub_records_iter = record.get_sub_records("letter").unwrap();
    assert_eq!(sub_records_iter.next().unwrap().get_all(), b"a");
    assert_eq!(sub_records_iter.next().unwrap().get_all(), b"b");
    assert_eq!(sub_records_iter.next().unwrap().get_all(), b"c");
    assert!(sub_records_iter.next().is_none());
}

#[test]
fn repeat_concat() {
    let calc_regex = generate! {
        lhs = ("a" - "z")^3;
        rhs = "A" - "Z", ("a" - "z")^2;
        foo := lhs, rhs;
        bar := foo ^ 2;
    };
    let mut reader = $get_reader("fooBarbarFoo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let mut sub_records_iter = record.get_sub_records("foo").unwrap();
    let sub_record = sub_records_iter.next().unwrap();
    assert_eq!(sub_record.get_capture("lhs").unwrap(), b"foo");
    assert_eq!(sub_record.get_capture("rhs").unwrap(), b"Bar");
    let sub_record = sub_records_iter.next().unwrap();
    assert_eq!(sub_record.get_capture("lhs").unwrap(), b"bar");
    assert_eq!(sub_record.get_capture("rhs").unwrap(), b"Foo");
    assert!(sub_records_iter.next().is_none());
}

#[test]
fn repeat_concat_nested() {
    let calc_regex = generate! {
        lhs = ("a" - "z")^3;
        rhs = "A" - "Z", ("a" - "z")^2;
        foo := lhs, rhs;
        bar := foo ^ 2;
        baz := bar;
    };
    let mut reader = $get_reader("fooBarbarFoo".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let mut sub_records_iter = record.get_sub_records("bar.foo").unwrap();
    let sub_record = sub_records_iter.next().unwrap();
    assert_eq!(sub_record.get_capture("lhs").unwrap(), b"foo");
    assert_eq!(sub_record.get_capture("rhs").unwrap(), b"Bar");
    let sub_record = sub_records_iter.next().unwrap();
    assert_eq!(sub_record.get_capture("lhs").unwrap(), b"bar");
    assert_eq!(sub_record.get_capture("rhs").unwrap(), b"Foo");
    assert!(sub_records_iter.next().is_none());
}

#[test]
fn repeat_nested() {
    let calc_regex = generate! {
        lower = "a" - "z";
        inner := lower ^ 3;
        outer := inner ^ 2;
    };
    let mut reader = $get_reader("foobar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let mut sub_records_iter = record.get_sub_records("inner").unwrap();
    let sub_record = sub_records_iter.next().unwrap();
    let mut capture_iter = sub_record.get_captures("lower").unwrap();
    assert_eq!(capture_iter.next().unwrap(), b"f");
    assert_eq!(capture_iter.next().unwrap(), b"o");
    assert_eq!(capture_iter.next().unwrap(), b"o");
    assert!(capture_iter.next().is_none());
    let sub_record = sub_records_iter.next().unwrap();
    let mut capture_iter = sub_record.get_captures("lower").unwrap();
    assert_eq!(capture_iter.next().unwrap(), b"b");
    assert_eq!(capture_iter.next().unwrap(), b"a");
    assert_eq!(capture_iter.next().unwrap(), b"r");
    assert!(capture_iter.next().is_none());
    assert!(sub_records_iter.next().is_none());
}

#[test]
fn repeat_nested_twice() {
    let calc_regex = generate! {
        lower = "a" - "z";
        inner := lower ^ 3;
        outer := inner ^ 2;
    };
    let mut reader = $get_reader("foobar".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let mut sub_records_iter = record.get_sub_records("inner").unwrap();
    let sub_record = sub_records_iter.next().unwrap();
    let mut sub_sub_records_iter =
        sub_record.get_sub_records("lower").unwrap();
    assert_eq!(sub_sub_records_iter.next().unwrap().get_all(), b"f");
    assert_eq!(sub_sub_records_iter.next().unwrap().get_all(), b"o");
    assert_eq!(sub_sub_records_iter.next().unwrap().get_all(), b"o");
    assert!(sub_sub_records_iter.next().is_none());
    let sub_record = sub_records_iter.next().unwrap();
    let mut sub_sub_records_iter =
        sub_record.get_sub_records("lower").unwrap();
    assert_eq!(sub_sub_records_iter.next().unwrap().get_all(), b"b");
    assert_eq!(sub_sub_records_iter.next().unwrap().get_all(), b"a");
    assert_eq!(sub_sub_records_iter.next().unwrap().get_all(), b"r");
    assert!(sub_sub_records_iter.next().is_none());
    assert!(sub_records_iter.next().is_none());
}

///////////////////////////////////////////////////////////////////////////////
//      Erroneous Capture Access
///////////////////////////////////////////////////////////////////////////////

#[test]
fn non_existent_single_capture() {
    let calc_regex = generate! {
        letter = "a" - "z";
        foo := letter ^ 3;
    };
    let mut reader = $get_reader("abc".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_sub_record("foo").unwrap_err();
    if let NameError::NoSuchName { name } = err {
        assert_eq!(name, "foo");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn non_existent_repeat_capture() {
    let calc_regex = generate! {
        letter = "a" - "z";
        foo := letter ^ 3;
    };
    let mut reader = $get_reader("abc".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_sub_records("foo").unwrap_err();
    if let NameError::NoSuchName { name } = err {
        assert_eq!(name, "foo");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn get_single_from_repeated() {
    let calc_regex = generate! {
        letter = "a" - "z";
        foo := letter ^ 3;
    };
    let mut reader = $get_reader("abc".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_sub_record("letter").unwrap_err();
    if let NameError::MisplacedSingleAccess { name } = err {
        assert_eq!(name, "letter");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn get_repeated_from_single() {
    let calc_regex = generate! {
        letter = "a" - "z";
        foo := letter;
    };
    let mut reader = $get_reader("a".as_bytes());
    let record = reader.parse(&calc_regex).unwrap();
    let err = record.get_sub_records("letter").unwrap_err();
    if let NameError::MisplacedRepeatAccess { name } = err {
        assert_eq!(name, "letter");
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
