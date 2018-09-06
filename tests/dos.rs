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
#[ignore]
fn netstring_invalid() {
    let netstring = generate! {
        bytes         = (%0 - %FF)*;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        netstring    := pf_number.decimal, bytes#decimal, ",";
    };
    let mut reader = calc_regex::Reader::from_array(&[b'0'; 10_000_000]);
    reader.parse(&netstring).unwrap_err(); // ~4.3s
}

#[test]
#[ignore]
fn netstring_partially_valid() {
    let netstring = generate! {
        bytes         = (%0 - %FF)*;
        nonzero_digit = "1" - "9";
        digit         = "0" | nonzero_digit;
        number        = "0" | (nonzero_digit, digit*);
        pf_number     = number, ":";
        netstring    := pf_number.decimal, bytes#decimal, ",";
    };
    let mut bytes = b"10000000:".to_vec();
    bytes.append(&mut [b'0'; 10_000_000].to_vec());
    bytes.append(&mut b",".to_vec());
    let mut reader = calc_regex::Reader::from_array(&bytes);
    reader.parse(&netstring).unwrap(); // ~1.5s
}
