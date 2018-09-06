/*!
This library introduces a type of expressions called “calc-regular”.
It provides a way to create such an expression via a custom meta-language and
parse input against it — either from a byte array in memory or from a stream.

# Calc-Regular Expressions

Calc-Regular Expressions are similar to regular expressions (Regex), but they
allow for some additional productions.
The goal of these productions is mainly to allow languages with length fields.
This technically makes calc-regular expressions context-free grammars in the
Chomsky hierarchy, but calc-regular expressions are quite restricted in these
terms, making them suitable to parse large amounts of data without too much
effort.

Calc-regular expressions are defined by a custom [meta-language] and can be
built using the provided [`generate!`] macro.

Parts of a calc-regular expression are required to be **prefix-free**.
See [here][prefix-free] for details.

# Reader, Record, and Capture

Once you have generated a [`CalcRegex`], you can parse input against it.
To do that, you need to initialize a [`Reader`] from your input.
A `Reader` can be created [from a byte array][`from_array`] or [from
an `io::Read` stream][`from_stream`].

With the `Reader`, you can parse against your `CalcRegex` into a [`Record`]
using [`parse`].
If you have multiple concatenated words you want to match against the same
calc-regular expression, you can use [`parse_many`], which returns an iterator
over `Record`s.

When using `parse` or `parse_many`, the `Reader` saves the bytes it read in the
process into the `Record`.
You can get a reference to all matched bytes with [`get_all`].
It also remembers what parts of you expression it parsed which parts of your
input against.
This is called capturing.

Captures can be accessed by the names of their sub-expressions that were used
when generating the expression using [`get_capture`] and some other methods of
[`Record`].

## Costs

When parsing from a stream, the `Reader` keeps a copy of all bytes it read and
gives it to the `Record` when finished.
The memory is freed when the `Record` goes out of scope.

For `Reader`s initialized from a byte array, no data is copied.
That means, you have to keep the byte array around for the `Reader` and any
`Record` to work, but additional memory is only used for meta data.

The above restrictions are enforced by the compiler; you don't get any unsafe
code.

# Examples

A minimal example, parsing a predefined string:

```
#[macro_use] extern crate calc_regex;

# fn main() {
let mut array_reader = calc_regex::Reader::from_array(b"foo");

let re = generate!(
    foo = "foo";
);

let record = array_reader.parse(&re).unwrap();
assert_eq!(record.get_all(), b"foo");
# }
```

Still a simple example, making use of the new length-count expression:

```
#[macro_use] extern crate calc_regex;
use std::str;

# fn main() {
/// Parses a decimal number from a byte array.
///
/// E.g. decimal(b"42") -> Some(42)
fn decimal(number: &[u8]) -> Option<usize> {
    let number = str::from_utf8(number).ok()?;
    number.parse::<usize>().ok()
}

let mut reader = calc_regex::Reader::from_array(b"5:fooo!");

let re = generate!(
    foo    = "f", "o"*, "!";
    digit  = "0" - "9";
    fooo  := digit.decimal, ":", foo#decimal;
);

let record = reader.parse(&re).unwrap();
assert_eq!(record.get_capture("digit").unwrap(), b"5");
assert_eq!(record.get_capture("foo").unwrap(), b"fooo!");
# }
```

A real-life example, reading a [Netstring]:

```
#[macro_use] extern crate calc_regex;
use std::str;

# fn main() {
/// Parses a bytestring containing a number and a trailing colon in ASCII
/// format to the respective number, discarding the colon.
///
/// E.g. decimal(b"42:") -> Some(42)
fn decimal(pf_number: &[u8]) -> Option<usize> {
    let (number, colon) = pf_number.split_at(pf_number.len() - 1);
    if colon != [b':'] {
        return None;
    }
    let number = str::from_utf8(number).ok()?;
    number.parse::<usize>().ok()
}

let netstring = generate! {
    byte          = %0 - %FF;
    nonzero_digit = "1" - "9";
    digit         = "0" | nonzero_digit;
    number        = "0" | (nonzero_digit, digit*);
    pf_number     = number, ":";
    netstring    := pf_number.decimal, (byte*)#decimal, ",";
};

let mut reader = calc_regex::Reader::from_array(b"3:foo,");
let record = reader.parse(&netstring).unwrap();

assert_eq!(record.get_capture("pf_number").unwrap(), b"3:");
assert_eq!(record.get_capture("$value").unwrap(), b"foo");
# }
```

# Limitations

## Kleene Star

Like the union operator, the Kleene star is generally only allowed in
unrestricted, i.e. regex productions.
This is not strictly a language-theoretic limitation, as this should
be OK as long as the total size is limited by a length-count production.
However, this would lead to cases, where the parser cannot know if the input
should still be matched against the repeated expression or another expression
following it, requiring backtracking or more sophisticated approaches.
This has some disadvantages:

- The need for implementing some more or less advanced approaches of handling
  non-deterministic processes.
- Potentially high run-time costs.
- Incompatibility to reading input from a stream byte-by-byte without going
  back.

In order to circumvent these problems, usage of the Kleene star on calc-regular
expressions is limited to the top-most level a length-counted production.
This way, the parser can know at any time whether to continue matching the
repeated expression.

## Anonymous Repeats

Repeats in restricted productions can only be applied to identifiers and not
general calc-regex productions.
This affects repeats with a hard-coded number of repetitions and
occurrence-counted productions
This limitation doesn't originate from problems when parsing such an
expression, but when accessing captures.
When allowing anonymous repeats, the same name could occur multiple times
inside a repeated expression or in different repeated expressions located in
the same scope.
Consider, for example, the following production:

```plain
foo := (bar, baz, bar)^2, bar^3;
```

This kind of production would cause two problems:

- Accessing the captures of this expression by names in a consistent and
  intuitive way doesn't seem possible.
- Traversal of the saved captures becomes more complicated (if the user asks
  for some repeated identifier, which of the repeats will it be in?).

In order to avoid these problems and unnecessary confusion, the user is asked
to explicitly assign names to any repeated expressions, so accessing captures
will be straight forward.

## Regex Captures

Captures can only be obtained from calc-regular expressions, i.e. productions
that are assigned via the `:=` operator or named expressions that are used
inside such productions in a way compliant with the restrictions imposed on
calc-regular expressions (otherwise, the sub-production containing them is
considered an anonymous regular production).

While the [`regex`] crate supports captures itself, its capture system differs
from the one used here.
Mainly, there is no concept of a name hierarchy, and only the last occurrence
of a repeated expression is kept.
This presents us with the following problems with our current approach of
parsing regular expressions using the [`regex`] crate.

- Capturing repeated expressions inside a regular expression is basically
  impossible.
- Introducing a system of assigning qualified names to regular sub-expressions
  would add severe complexity to the process of generating expressions from our
  meta-language.

[`regex`]: https://doc.rust-lang.org/regex/regex/index.html
[`generate!`]: macro.generate.html
[meta-language]: macro.generate.html#the-meta-language
[prefix-free]: macro.generate.html#requirement-for-prefix-free-expressions
[`CalcRegex`]: struct.CalcRegex.html
[`Reader`]: reader/struct.Reader.html
[`Record`]: reader/struct.Record.html
[`from_array`]: struct.Reader.html#method.from_array
[`from_stream`]: struct.Reader.html#method.from_stream
[`parse`]: struct.Reader.html#method.parse
[`parse_many`]: struct.Reader.html#method.parse_many
[`get_all`]: reader/struct.Record.html#method.get_all
[`get_capture`]: reader/struct.Record.html#method.get_capture
[Netstring]: https://cr.yp.to/proto/netstrings.txt
*/

#![deny(missing_docs)]
// #![feature(trace_macros)]
#![recursion_limit="128"]

extern crate regex;

#[macro_use]
#[doc(hidden)]
pub mod generate;

pub mod aux;

mod calc_regex;
pub use calc_regex::CalcRegex;

mod error;
pub use error::{NameError, NameResult, ParserError, ParserResult};

pub mod reader;
pub use reader::Reader;

#[cfg(test)]
mod tests;
