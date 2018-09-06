#[macro_use(generate)]
extern crate calc_regex;

use std::str;

fn decimal(number: &[u8]) -> Option<usize> {
    let number = match str::from_utf8(number) {
        Ok(n) => n,
        Err(_) => return None,
    };
    number.parse::<usize>().ok()
}

#[test]
fn non_prefix_free() {
    let netstring = generate! {
        digit       = "0" - "9";
        calc_regex := digit.decimal, ("a"*, "b"*)#decimal;
    };

    let mut reader = calc_regex::Reader::from_array(b"3bbb");
    let _record = reader.parse(&netstring).unwrap();

    let mut reader = calc_regex::Reader::from_array(b"3abb");
    let _record = reader.parse(&netstring).unwrap_err();

    let mut reader = calc_regex::Reader::from_array(b"3aab");
    let _record = reader.parse(&netstring).unwrap_err();
}

#[test]
fn non_prefix_free_fixed() {
    let netstring = generate! {
        digit       = "0" - "9";
        value       = "a"*, "b"*;
        calc_regex := digit.decimal, (value)#decimal;
    };

    let mut reader = calc_regex::Reader::from_array(b"3bbb");
    let _record = reader.parse(&netstring).unwrap();

    let mut reader = calc_regex::Reader::from_array(b"3abb");
    let _record = reader.parse(&netstring).unwrap();

    let mut reader = calc_regex::Reader::from_array(b"3aab");
    let _record = reader.parse(&netstring).unwrap();
}

#[test]
fn kleene_star() {
    let netstring = generate! {
        digit       = "0" - "9";
        calc_regex := digit.decimal, (("a"^2 | "a"^3)*)#decimal;
    };

    let mut reader = calc_regex::Reader::from_array(b"2aa");
    let _record = reader.parse(&netstring).unwrap();

    let mut reader = calc_regex::Reader::from_array(b"3aaa");
    let _record = reader.parse(&netstring).unwrap_err();
}

#[test]
fn kleene_star_fixed() {
    let netstring = generate! {
        digit       = "0" - "9";
        value       = ("a"^2 | "a"^3)*;
        calc_regex := digit.decimal, (value)#decimal;
    };

    let mut reader = calc_regex::Reader::from_array(b"2aa");
    let _record = reader.parse(&netstring).unwrap();

    let mut reader = calc_regex::Reader::from_array(b"3aaa");
    let _record = reader.parse(&netstring).unwrap();
}
