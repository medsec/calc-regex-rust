#![feature(trace_macros)]
#![feature(log_syntax)]

#[macro_use(generate)]
extern crate calc_regex;

fn cnta(s: &[u8]) -> Option<usize> {
    let cnt = s.iter()
        .fold(0, |cnt, c| if *c == b'a' { cnt + 1 } else { cnt });
    Some(cnt)
}

fn cntd(s: &[u8]) -> Option<usize> {
    let cnt = s.iter()
        .fold(0, |cnt, c| if *c == b'd' { cnt + 1 } else { cnt });
    Some(cnt)
}

#[test]
fn language_a() {
    let expr = generate! {
        expr := ("a"*).cnta, ("b"*)#cnta;
    };
    let mut reader = calc_regex::Reader::from_array(b"aabb");
    let err = reader.parse(&expr).unwrap_err();
    if let calc_regex::ParserError::TrailingCharacters = err {
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn language_b1() {
    let expr = generate! {
        expr := ("a"*).cnta, "c", ("b"*)#cnta;
    };
    let mut reader = calc_regex::Reader::from_array(b"aacbb");
    let err = reader.parse(&expr).unwrap_err();
    if let calc_regex::ParserError::Regex { regex, value } = err {
        assert_eq!(regex, "^(?-u:c)$");
        assert_eq!(*value, [b'a']);
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn language_b2() {
    let expr = generate! {
        // expr := ("a"*, "c").cnta, ("b"*)#cnta;
        count = "a"*, "c";
        expr := count.cnta, ("b"*)#cnta;
        // bs = "b"*;
        // expr := count.cnta, bs#cnta;
    };
    let mut reader = calc_regex::Reader::from_array(b"aacbb");
    let record = reader.parse(&expr).unwrap();
    assert_eq!(record.get_capture("$value").unwrap(), b"bb");

    let mut reader = calc_regex::Reader::from_array(b"aacbc");
    let err = reader.parse(&expr).unwrap_err();
    if let calc_regex::ParserError::Regex { regex, value } = err {
        assert_eq!(regex, "^(?-u:b)$");
        assert_eq!(*value, [b'c']);
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn language_b3() {
    let expr = generate! {
        count = "a"*, "c";
        b = "b";
        expr := count.cnta, b^cnta;
    };
    let mut reader = calc_regex::Reader::from_array(b"aacbb");
    let record = reader.parse(&expr).unwrap();
    assert_eq!(record.get_capture("$value").unwrap(), b"bb");

    let mut reader = calc_regex::Reader::from_array(b"aacbc");
    let err = reader.parse(&expr).unwrap_err();
    if let calc_regex::ParserError::Regex { regex, value } = err {
        assert_eq!(regex, "^(?-u:b)$");
        assert_eq!(*value, [b'c']);
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn language_c() {
    let expr = generate! {
        count_a = "a"*, "c";
        count_d = "d"*, "f";
        e = "e";
        expr := count_a.cnta, (count_d.cntd, e^cntd)#cnta;
    };
    let mut reader = calc_regex::Reader::from_array(b"aaacdfe");
    let record = reader.parse(&expr).unwrap();
    assert_eq!(record.get_capture("$value").unwrap(), b"dfe");
    assert_eq!(
        record.get_captures("e").unwrap().collect::<Vec<&[u8]>>(),
        [b"e"]
    );

    let mut reader = calc_regex::Reader::from_array(b"aaaaacddfee");
    let record = reader.parse(&expr).unwrap();
    assert_eq!(record.get_capture("$value").unwrap(), b"ddfee");
    assert_eq!(
        record.get_captures("e").unwrap().collect::<Vec<&[u8]>>(),
        [b"e", b"e"]
    );

    let mut reader = calc_regex::Reader::from_array(b"aaaaaaacdddfeee");
    let _record = reader.parse(&expr).unwrap();
}
