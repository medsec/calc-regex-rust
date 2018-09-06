//! Tests some possible real-life examples.

#[macro_use(generate)]
extern crate calc_regex;

fn g(l1: &[u8]) -> Option<usize> {
    if l1.len() != 1 {
        None
    } else if l1[0] < 128 {
        Some(0)
    } else {
        Some((l1[0] - 128) as usize)
    }
}

fn f(l2: &[u8]) -> Option<usize> {
    if l2.len() == 1 && l2[0] < 128 {
        Some(l2[0] as usize)
    } else if l2.len() > 1 &&
        l2.len() == l2[0] as usize - 127 &&
        l2.len() <= 9
    {
        calc_regex::aux::big_endian(&l2[1..])
    } else {
        None
    }
}

#[test]
fn nested_number() {
    let re = generate! {
        l1  = %0 - %FF;
        l2  = (%0 - %FF)*;
        l  := l1.g, l2#g;
        lv := l.f, ((%0 - %FF)*)#f;
    };
    let mut reader = calc_regex::Reader::from_array(b"\x03foo");
    let record = reader.parse(&re).unwrap();
    let v = record.get_capture("$value").unwrap();
    assert_eq!(v, b"foo");
    assert_eq!(record.get_capture("l.l1").unwrap(), b"\x03");
    assert_eq!(record.get_capture("l.l2").unwrap(), b"");

    let mut reader = calc_regex::Reader::from_array(b"\x81\x03bar");
    let record = reader.parse(&re).unwrap();
    let v = record.get_capture("$value").unwrap();
    assert_eq!(v, b"bar");

    let mut reader = calc_regex::Reader::from_array(b"\x82\x00\x03baz");
    let record = reader.parse(&re).unwrap();
    let v = record.get_capture("$value").unwrap();
    assert_eq!(v, b"baz");
}
