/*!
Auxiliary functions to read length fields.

# Examples

```
#[macro_use] extern crate calc_regex;
use calc_regex::aux::big_endian;

# fn main() {
let re = generate! {
    byte  = %0 - %FF;
    re   := (byte^2).big_endian, (byte*)#big_endian;
};

let mut reader = calc_regex::Reader::from_array(&[0, 3, b'f', b'o', b'o']);
let record = reader.parse(&re).unwrap();

assert_eq!(record.get_capture("$count").unwrap(), &[0, 3]);
assert_eq!(record.get_capture("$value").unwrap(), b"foo");
# }
```
*/

use std::mem;
use std::str;
use std::usize;

/// Parses a decimal number from a byte array.
///
/// # Examples
/// ```
/// # use calc_regex::aux::decimal;
/// assert_eq!(decimal(b"42"), Some(42));
/// ```
pub fn decimal(bytes: &[u8]) -> Option<usize> {
    let string = str::from_utf8(bytes).ok()?;
    string.parse::<usize>().ok()
}

/// Parses a hexadecimal number from a byte array.
///
/// # Examples
/// ```
/// # use calc_regex::aux::hex;
/// assert_eq!(hex(b"2A"), Some(42));
/// ```
pub fn hex(bytes: &[u8]) -> Option<usize> {
    let string = str::from_utf8(bytes).ok()?;
    usize::from_str_radix(string, 16).ok()
}

/// Reads raw value from byte array in little-endian format.
///
/// # Examples
/// ```
/// # use calc_regex::aux::little_endian;
/// assert_eq!(little_endian(&[0x0a, 0x0b, 0x00]), Some(0x0b0a));
/// ```
pub fn little_endian(bytes: &[u8]) -> Option<usize> {
    if bytes.len() > mem::size_of::<usize>() {
        return None;
    }
    let mut number = 0;
    for i in 0..bytes.len() {
        number += (bytes[i] as usize) * 256usize.pow(i as u32);
    }
    Some(number)
}

/// Reads raw value from byte array in big-endian format.
///
/// # Examples
/// ```
/// # use calc_regex::aux::big_endian;
/// assert_eq!(big_endian(&[0x00, 0x0a, 0x0b]), Some(0x0a0b));
/// ```
pub fn big_endian(bytes: &[u8]) -> Option<usize> {
    if bytes.len() > mem::size_of::<usize>() {
        return None;
    }
    let mut number = 0;
    for i in 0..bytes.len() {
        let exp = (bytes.len() - 1 - i) as u32;
        number += (bytes[i] as usize) * 256usize.pow(exp);
    }
    Some(number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal() {
        assert_eq!(decimal(b"42"), Some(42));
        assert_eq!(decimal(b"ab"), None);
    }

    #[test]
    fn test_hex() {
        assert_eq!(hex(b"2A"), Some(42));
        assert_eq!(hex(b"2a"), Some(42));
        assert_eq!(hex(b"a"), Some(10));
        assert_eq!(hex(b"g"), None);
        assert_eq!(hex(b"0x2a"), None);
    }

    #[test]
    fn test_little_endian() {
        assert_eq!(little_endian(&[0x0a, 0x0b]), Some(0x0b0a));
        assert_eq!(little_endian(&[0x0a, 0x0b, 0x00, 0x00]), Some(0x0b0a));
        assert_eq!(little_endian(&[0x0a, 0x0b, 0x0c, 0x0d]), Some(0x0d0c0b0a));
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_little_endian_64() {
        assert_eq!(
            little_endian(&[0x0d, 0x0c, 0x0b, 0x0a, 0x04, 0x03, 0x02, 0x01]),
            Some(0x010203040a0b0c0d)
        );
        assert_eq!(
            little_endian(&[
                0x0d, 0x0c, 0x0b, 0x0a, 0x04, 0x03, 0x02, 0x01, 0x00
            ]),
            None
        )
    }

    #[test]
    fn test_big_endian() {
        assert_eq!(big_endian(&[0x0a, 0x0b]), Some(0x0a0b));
        assert_eq!(big_endian(&[0x00, 0x00, 0x0a, 0x0b]), Some(0x0a0b));
        assert_eq!(big_endian(&[0x0a, 0x0b, 0x00, 0x00]), Some(0x0a0b0000));
        assert_eq!(big_endian(&[0x0a, 0x0b, 0x0c, 0x0d]), Some(0x0a0b0c0d));
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_big_endian_64() {
        assert_eq!(
            big_endian(&[0x01, 0x02, 0x03, 0x04, 0x0a, 0x0b, 0x0c, 0x0d]),
            Some(0x010203040a0b0c0d)
        );
        assert_eq!(
            big_endian(&[
                0x01, 0x02, 0x03, 0x04, 0x0a, 0x0b, 0x0c, 0x0d, 0x00,
            ]),
            None
        )
    }
}
