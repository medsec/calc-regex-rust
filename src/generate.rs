//! A macro to generate `CalcRegex` objects.
//!
//! This module contains the `generate!` macro and some types used by it.
//!
//! It is public, so the macro can access its types, but everything in here but
//! `generate!` should be considered internal.

use std::cell::RefCell;
use std::cmp;

use regex;

use calc_regex::{CalcRegex, Node, Inner, NodeIndex};

// Public types are used by `generate!` and are not meant to be part of the
// public interface.
// Hence their documentation is hidden.

/// Interim regular expressions.
///
/// This type is used for regexes that are used as parts of other regexes or
/// calc-regexes.
/// Combination with other regexes is done by string manipulation.
/// In order to use a regex in a calc-regex, the regex is compiled and
/// incorporated into a `CalcRegex`.
pub struct Regex {
    /// The uncompiled regex.
    re: String,
    /// Some attributes that are needed for construction.
    attributes: RegexAttributes,
    /// A cache for a compiled version of the regex.
    compiled: RefCell<Option<NodeIndex>>,
}

/// A type to keep track of the maximum length and other attributes of a regex
/// while it is being constructed.
struct RegexAttributes {
    /// Whether the current regex has a `|` operator on its highest level.
    ///
    /// The `|` operator has even lower precedence than concatenation, thus we
    /// need to wrap the regex in parentheses when concatenating if this is
    /// `true`.
    is_choice: bool,
    /// The total maxium length a matching value could have.
    total_length: Option<usize>,
    /// If the regex is a choice, keep track of the current length of the
    /// right-hand side of that choice, which can still be extended. Is is only
    /// useful if `total_length` is not `None` and `is_choice` is `true`.
    current_choice_length: Option<usize>,
}

impl RegexAttributes {
    /// Joins the attributes of an existing partial regex with a new element.
    ///
    /// Determines, whether the new regex is a choice, i.e. '|' operator at top
    /// level, and calculates maximum lengths.
    ///
    /// - `el_len` -- The maximum length of the new element, if any.
    fn join(&self, el_len: Option<usize>) -> Self {
        match (
            self.total_length,
            self.current_choice_length,
            self.is_choice,
            el_len,
        ) {
            // Both sides have length information. We can compute the total
            // maximum length.
            (Some(total_length),
             Some(current_choice_length),
             true,
             Some(el_len)) => {
                // Length of the new element is added to the choice currently
                // expanded. Afterwards, we check whether the total maximum
                // length needs to be updated.
                let current_choice_length = el_len + current_choice_length;
                RegexAttributes {
                 is_choice: true,
                    total_length: Some(cmp::max(
                        total_length,
                        current_choice_length
                    )),
                    current_choice_length: Some(current_choice_length),
                }
            }
            (Some(total_length), None, false, Some(el_len)) => {
                RegexAttributes {
                    is_choice: false,
                    total_length: Some(el_len + total_length),
                    current_choice_length: None,
                }
            }
            // At least one side doesn't have length information. We don't have
            // a total maximum length.
            (None, None, true, _) |
            (Some(_), Some(_), true, None) => {
                RegexAttributes {
                    is_choice: true,
                    total_length: None,
                    current_choice_length: None,
                }
            }
            (_, None, false, None) |
            (None, None, false, _) => {
                RegexAttributes {
                    is_choice: false,
                    total_length: None,
                    current_choice_length: None,
                }
            }
            // Invalid patterns.
            (Some(_), None, true, _) |
            (_, Some(_), false, _) |
            (None, Some(_), true, _) => {
                // Something went wrong with our production. This should never
                // be reached.
                panic!(
                    "Expected current_choice_length to be some value iff \
                     total_length is not None and is_choice is true!"
                );
            }
        }
    }
}

impl Default for Regex {
    fn default() -> Regex {
        Regex {
            re: "".to_owned(),
            attributes: RegexAttributes {
                is_choice: false,
                total_length: Some(0),
                current_choice_length: None,
            },
            compiled: RefCell::new(None),
        }
    }
}

impl Regex {
    /// Instantiates an empty regex.
    pub fn new() -> Self {
        Default::default()
    }

    /// The maximum length a matching value could have.
    fn max_length(&self) -> Option<usize> {
        self.attributes.total_length
    }

    /// Whether the regex is immune to separation by strongly binding
    /// operators.
    fn is_atomic(&self) -> bool {
        // Single character is fine...
        if self.re.len() == 1 {
            return true;
        }
        // ...so is an expression wrapped in parentheses...
        let mut chars = self.re.chars();
        if (Some('('), Some(')')) == (chars.next(), chars.last()) {
            return true;
        }
        // ...and a range.
        let mut chars = self.re.chars();
        (Some('['), Some(']')) == (chars.next(), chars.last())
    }

    /// Compiles the regex using `regex::bytes`.
    ///
    /// The first time this function is called, the compilation is done and the
    /// result is cached. On further calls, We can just reuse the cached value.
    /// We use a `RefCell` for interior mutability to provide an immutable
    /// interface despite of mutating the cache.
    fn compile(
        &self,
        calc_regex: &mut CalcRegex,
        name: Option<String>
    ) -> NodeIndex {
        if let Some(node_index) = *self.compiled.borrow() {
            // `name` is expected here to always be the stringified identifier.
            // `compile()` might be called multiple times, but the identifier
            // should never change.
            debug_assert_eq!(name, calc_regex.get_node(node_index).name);
            return node_index;
        }
        let inner = Inner::Regex(
            // Wrap regex in `^()$`. `^$`, so only complete matches are
            // considered and `()` so the `|` operator won't separate the `^$`
            // marks from the actual regex. Also disable Unicode support, so
            // non-unicode bytes can be matched.
            regex::bytes::Regex::new(
                &("^(?-u:".to_owned() + &self.re + ")$")
            ).unwrap()
        );
        let node = Node {
            name,
            length_bound: self.max_length(),
            inner,
        };
        let node_index = calc_regex.push_node(node);
        *self.compiled.borrow_mut() = Some(node_index);
        node_index
    }
}

/// Interim values for (calc-)regex productions.
///
/// Variables in production definitions can either hold regexes or
/// calc-regexes. They use this type.
pub enum Interim {
    Regex(Regex),
    CalcRegex(NodeIndex),
}

/// Non-restricted production rules for regexes.
///
/// These are generated and called `apply()` on within the `generate!` macro.
pub enum RegexProduction<'a> {
    Identifier(&'a Interim),
    Literal(&'a str),
    ByteLiteral(&'a str),
    Parentheses(&'a Regex),
    Choice,
    KleeneStar(&'a Regex),
    KleenePlus(&'a Regex),
    Repeat(&'a Regex, usize),
    CharRange(&'a str, &'a str),
    HexRange(&'a str, &'a str),
}

impl<'a> RegexProduction<'a> {
    /// Generates a partial regex to be compiled into a `Regex` by the `regex`
    /// crate.
    ///
    /// Processes a new part of the regex and joins it with the parts that are
    /// already processed.
    pub fn apply(self, prev: Regex) -> Regex {
        match self {
            RegexProduction::Identifier(interim) => {
                if let Interim::Regex(ref el) = *interim {
                    Regex {
                        // Conditionally wrap new element in parentheses. We
                        // need to do this because the user expects an
                        // identifier to be implicitly encapsulated.
                        re: if el.attributes.is_choice {
                            prev.re + "(" + &el.re + ")"
                        } else {
                            prev.re + &el.re
                        },
                        attributes: prev.attributes.join(el.max_length()),
                        compiled: RefCell::new(None),
                    }
                } else {
                    panic!("Found CalcRegex in regular production rule!")
                }
            }
            RegexProduction::Literal(s) => {
                Regex {
                    re: prev.re + &regex::escape(s),
                    attributes: prev.attributes.join(Some(s.len())),
                    compiled: RefCell::new(None),
                }
            }
            RegexProduction::ByteLiteral(v) => {
                if let Ok(v) = u8::from_str_radix(v, 16)
                {
                    Regex {
                        // Format `v` to be exactly two upper-case hex
                        // characters.
                        re: prev.re + &format!("\\x{:02X}", v),
                        attributes: prev.attributes.join(Some(1)),
                        compiled: RefCell::new(None),
                    }
                } else {
                    panic!("Found non-hex values in byte literal!");
                }
            }
            RegexProduction::Parentheses(el) => {
                Regex {
                    re: prev.re + "(" + &el.re + ")",
                    attributes: prev.attributes.join(el.max_length()),
                    compiled: RefCell::new(None),
                }
            }
            RegexProduction::Choice => {
                Regex {
                    re: prev.re + "|",
                    attributes: RegexAttributes {
                        is_choice: true,
                        total_length: prev.attributes.total_length,
                        current_choice_length: prev.attributes
                            .total_length
                            .and(Some(0)),
                    },
                    compiled: RefCell::new(None),
                }
            }
            RegexProduction::KleeneStar(el) => {
                Regex {
                    // Most of the time, the operand must be put into
                    // parentheses as the Kleene star binds very strongly. E.g.
                    // `"foo"*` would otherwise generate `foo*`, which has
                    // operator precedence like `fo(o)*`.
                    re: if el.is_atomic() {
                        prev.re + &el.re + "*"
                    } else {
                        prev.re + "(" + &el.re + ")*"
                    },
                    attributes: RegexAttributes {
                        is_choice: prev.attributes.is_choice,
                        // We cannot bound the length anymore.
                        total_length: None,
                        current_choice_length: None,
                    },
                    compiled: RefCell::new(None),
                }
            }
            RegexProduction::KleenePlus(el) => {
                Regex {
                    re: if el.is_atomic() {
                        prev.re + &el.re + "+"
                    } else {
                        prev.re + "(" + &el.re + ")+"
                    },
                    attributes: RegexAttributes {
                        is_choice: prev.attributes.is_choice,
                        total_length: None,
                        current_choice_length: None,
                    },
                    compiled: RefCell::new(None),
                }
            }
            RegexProduction::Repeat(el, n) => {
                Regex {
                    re: if el.is_atomic() {
                        // "[a-z]", 3 will become "[a-z]{3}".
                        prev.re + &format!("{}{{{}}}", el.re, n)
                    } else {
                        // "foo", 3 will become "(foo){3}".
                        prev.re + &format!("({}){{{}}}", el.re, n)
                    },
                    attributes: prev.attributes.join(
                        el.max_length().map(|l| l * n)
                    ),
                    compiled: RefCell::new(None),
                }
            }
            RegexProduction::CharRange(min, max) => {
                assert!(min.len() == 1 && max.len() == 1,
                        "Ranges must be between two single characters!");
                assert!(min <= max,
                        "Lower range value is grater then upper value!");
                Regex {
                    re: prev.re + "[" + min + "-" + max + "]",
                    attributes: prev.attributes.join(Some(1)),
                    compiled: RefCell::new(None),
                }

            }
            RegexProduction::HexRange(min, max) => {
                if let (Ok(min), Ok(max)) = (
                    u8::from_str_radix(min, 16),
                    u8::from_str_radix(max, 16)
                ) {
                    assert!(min <= max,
                            "Lower range value is grater then upper value!");
                    // Format ranges to be exactly two upper-case hex
                    // characters.
                    Regex {
                        re: prev.re +
                            &format!("[\\x{:02X}-\\x{:02X}]", min, max),
                        attributes: prev.attributes.join(Some(1)),
                        compiled: RefCell::new(None),
                    }
                } else {
                    panic!("Found non-hex values in hex range!");
                }
            }
        }
    }
}

/// Restricted production rules for calc-regexes.
///
/// These are generated and called `apply()` on within the `generate!` macro.
pub enum CalcRegexProduction<'a> {
    Identifier(&'a Interim, String),
    Regex(&'a Regex),
    Concat(NodeIndex, NodeIndex),
    Repeat(NodeIndex, usize),
    KleeneStar(NodeIndex),
    LengthCount {
        r: NodeIndex,
        s: Option<NodeIndex>,
        t: NodeIndex,
        f: Box<fn(&[u8]) -> Option<usize>>,
    },
    OccurrenceCount {
        r: NodeIndex,
        s: Option<NodeIndex>,
        t: NodeIndex,
        f: Box<fn(&[u8]) -> Option<usize>>,
    },
}

impl<'a> CalcRegexProduction<'a> {
    /// Generates `CalcRegex`es, that can be used directly or be compiled into
    /// other `CalcRegex`es.
    pub fn apply(
        self,
        calc_regex: &mut CalcRegex,
        name: Option<String>,
    ) -> NodeIndex {
        match self {
            CalcRegexProduction::Identifier(interim, identifier) => {
                let node_index = match *interim {
                    Interim::Regex(ref regex) => {
                        regex.compile(calc_regex, Some(identifier))
                    }
                    Interim::CalcRegex(node_index) => {
                        node_index
                    }
                };
                match name {
                    // We are assigning this identifier. Explicitly
                    // encapsulate its calc-regex.
                    Some(name) => {
                        let node = Node {
                            name: Some(name),
                            length_bound: None,
                            inner: Inner::CalcRegex(node_index),
                        };
                        calc_regex.push_node(node)
                    }
                    // The calc-regex is used anonymously. Use as is.
                    None => {
                        node_index
                    }
                }
            }
            CalcRegexProduction::Regex(regex) => {
                regex.compile(calc_regex, name)
            }
            CalcRegexProduction::Concat(lhs, rhs) => {
                let node = Node {
                    name,
                    length_bound: None,
                    inner: Inner::Concat(lhs, rhs),
                };
                calc_regex.push_node(node)
            }
            CalcRegexProduction::Repeat(node_index, n) => {
                let node = Node {
                    name,
                    length_bound: None,
                    inner: Inner::Repeat(node_index, n),
                };
                calc_regex.push_node(node)
            }
            CalcRegexProduction::KleeneStar(node_index) => {
                let node = Node {
                    name,
                    length_bound: None,
                    inner: Inner::KleeneStar(node_index),
                };
                calc_regex.push_node(node)
            }
            CalcRegexProduction::LengthCount { r, s, t, f } => {
                let node = Node {
                    name,
                    length_bound: None,
                    inner: Inner::LengthCount { r, s, t, f },
                };
                calc_regex.push_node(node)
            }
            CalcRegexProduction::OccurrenceCount { r, s, t, f } => {
                if calc_regex.get_node(t).name.is_none() {
                    panic!("Anonymous repeat patterns are not supported. \
                            Please assign a name to the repeated \
                            expressions.");
                }
                let node = Node {
                    name,
                    length_bound: None,
                    inner: Inner::OccurrenceCount { r, s, t, f },
                };
                calc_regex.push_node(node)
            }
        }
    }
}

/// Generates a `CalcRegex` by production rules.
///
/// The `generate!` macro compiles a custom meta-language into a [`CalcRegex`]
/// object for parsing input via an instance of [`Reader`].
///
/// # The Meta-Language
///
/// The meta-language is a sequence of productions assigned to an identifier.
/// It distinguishes between productions for **regular** and productions for
/// **calc-regular** expressions.
/// Regular (or *unrestricted*) productions are assigned to an identifier using
/// `=`.
/// Calc-regular (or *restricted*) productions use `:=`.
///
/// Production assignments look like this:
///
/// ```plain
/// REGEX_IDENTIFIER = REGEX_PRODUCTION ;
/// CALC_REGEX_IDENTIFIER := CALC_REGEX_PRODUCTION ;
/// ```
///
/// where `REGEX_PRODUCTION` can be any of the following expressions with the
/// traditional meanings:
///
/// - `"STRING"` (literal)
/// - `%XX`, with `XX` between 0 and FF (byte literal)
/// - `REGEX_IDENTIFIER`
/// - `( REGEX_PRODUCTION )` (parentheses)
/// - `REGEX_PRODUCTION , REGEX_PRODUCTION` (concatenation)
/// - `REGEX_PRODUCTION | REGEX_PRODUCTION` (choice)
/// - `REGEX_PRODUCTION *` (Kleene star)
/// - `REGEX_PRODUCTION +` (Kleene plus)
/// - `REGEX_PRODUCTION ^ NUMBER` with `NUMBER`  &#x2265; 0 (repetition)
/// - `"A" - "B"`, with `A` and `B` being single characters (char range)
/// - `%AA - %BB`, with `%AA` and `%BB` being byte literals (byte range)
///
/// and `CALC_REGEX_PRODUCTION` can be any of the following expressions with
/// the traditional meanings:
///
/// - `REGEX_PRODUCTION` (regex)
/// - `CALC_REGEX_IDENTIFIER`
/// - `( CALC_REGEX_PRODUCTION )` (parentheses)
/// - `CALC_REGEX_PRODUCTION , CALC_REGEX_PRODUCTION` (concatenation)
/// - `CALC_REGEX_IDENTIFIER ^ NUMBER`, with `NUMBER`  &#x2265; 0 (repetition)
///
/// or the following novel expressions:
///
/// - `r . f , t # f` (length count)
/// - `r . f , s , t # f` (length count)
/// - `r . f , (t*) # f` (length count with Kleene star)
/// - `r . f , s , (t*) # f` (length count with Kleene star)
///
/// with
///
/// - `r`, `s` and `t` being `CALC_REGEX_PRODUCTION`s, and
/// - `f` being a function or closure of type `fn(&[u8]) -> Option<usize>`
///
/// and
///
/// - `r . f , t ^ f` (occurrence count)
/// - `r . f , s , t ^ f` (occurrence count)
///
/// with
///
/// - `r` and `s` being `CALC_REGEX_PRODUCTION`s,
/// - `t` being a `CALC_REGEX_IDENTIFIER`, and
/// - `f` being a function or closure of type `fn(&[u8]) -> Option<usize>`
///
/// and the following operator meanings:
///
/// - `,`: common concatenation.
/// - `r . f`: read a word `x` that matches `r` and compute `f(x)`.
/// - `t # f`: read a word that matches `t` and has a length of exactly`f(x)`
///   bytes.
/// - `(t*) # f`: read a word that matches any number of occurrences of `t` and
///   has a length of exactly`f(x)` bytes.
/// - `t ^ f`: read exactly `f(x)` words matching `t`.
///
/// If `f` returns `None`, the parser aborts with an error.
///
/// ## Requirement for Prefix-Free Expressions
///
/// In general, calc-regular expressions need to be prefix-free with one
/// exception:
/// the expression given for `t` in length-count productions may be
/// non-prefix-free.
/// If this expression is a concatenation, only the right-hand side my be
/// non-prefix-free (going down to the right-most part if further nested).
///
/// Strictly regular sub-expressions that are *not* used inside restricted
/// productions do not need to be prefix-free.
/// See [The Meta-Language] for the difference between restricted and
/// unrestricted productions.
///
/// Regex patterns are matched on as few bytes as possible with no attempt to
/// correctly match ambiguous expressions, whatsoever.
/// It is the responsibility of the user of this library to account for that by
/// respecting the above rules.
///
/// ### Warning
///
/// While unrestricted productions are generally allowed inside restricted
/// productions as syntactic sugar, if there is a restricted form of that
/// production, it will be used regardless of whether it would also be a valid
/// unrestricted production or not.
/// In that case, its components need to be prefix-free, even if the whole
/// sub-production is prefix-free.
///
/// E.g. consider the following production:
///
/// ```plain
/// inner = "a"*, "b"*, ".";
/// outer := inner;
/// ```
///
/// It is valid because `inner` is assigned using the `=` operator.
/// This way it is encapsulated in a single prefix-free regular expression,
/// which can safely be used in a calc-regex.
///
/// Now consider this production:
///
/// ```plain
/// outer := "a"*, "b"*, ".";
/// ```
///
/// Here, the individual components *could* be assembled to a regular
/// expression, but because they are inside a restricted production using the
/// `:=` operator, restricted production rules take precedence.
/// Since concatenation is a valid restricted production rule, only the
/// individual parts are treated as regular expressions.
/// But since they are not prefix-free, parsing against this expression will
/// generally **not** work.
///
/// ## Length Bounds
///
/// In general, the calc-regex parser tries to match its input against some
/// regex without knowing if the input is still valid at most points.
/// This could be exploited by sending it input that will never match the
/// expression.
/// To avoid this, expressions and sub-expressions can be length-bounded with
/// the [`set_root_length_bound`] and [`set_length_bound`] methods.
/// Additionally, regexes that can by their expression only match a limited
/// number of bytes are bounded automatically.
///
/// If unsure, which expressions are bounded, you can check the debug output of
/// your `CalcRegex`:
///
/// ```
/// # #[macro_use] extern crate calc_regex;
/// # fn main() {
/// let re = generate!(
///     // ...
///     # foo = "foo!";
/// );
/// println!("{:#?}", re);
/// # }
/// ```
///
/// # Examples
///
/// ## Plain Regex
///
/// ```
/// #[macro_use] extern crate calc_regex;
///
/// # fn main() {
/// let re = generate!(
///     foo = "foo";
///     bar = "bar";
///     foobar = foo | bar;
///     re = (foobar, " ")+, foobar, "!";
/// );
///
/// let mut reader = calc_regex::Reader::from_array(b"bar foo bar bar!");
/// let record = reader.parse(&re).unwrap();
/// # }
/// ```
///
/// ## Length Count
///
/// ```
/// #[macro_use] extern crate calc_regex;
/// use std::str;
///
/// # fn main() {
/// fn decimal(number: &[u8]) -> Option<usize> {
///     let number = match str::from_utf8(number) {
///         Ok(n) => n,
///         Err(_) => return None,
///     };
///     number.parse::<usize>().ok()
/// }
///
/// let re = generate!(
///     foo = "f", "o"*, "!";
///     digit = "0" - "9";
///     fooo := digit.decimal, ":", foo#decimal;
/// );
///
/// let mut reader = calc_regex::Reader::from_array(b"5:fooo!");
/// let record = reader.parse(&re).unwrap();
/// # }
/// ```
///
/// ## Occurrence Count
///
/// ```
/// #[macro_use] extern crate calc_regex;
/// use std::str;
///
/// # fn main() {
/// fn decimal(number: &[u8]) -> Option<usize> {
///     let number = match str::from_utf8(number) {
///         Ok(n) => n,
///         Err(_) => return None,
///     };
///     number.parse::<usize>().ok()
/// }
///
/// let re = generate!(
///     foo = "foo!";
///     digit = "0" - "9";
///     n_foos := digit.decimal, ":", foo^decimal;
/// );
///
/// let mut reader = calc_regex::Reader::from_array(b"3:foo!foo!foo!");
/// let record = reader.parse(&re).unwrap();
///
/// # }
/// ```
///
/// [`CalcRegex`]: struct.CalcRegex.html
/// [`Reader`]: reader/struct.Reader.html
/// [`set_root_length_bound`]:
///     struct.CalcRegex.html#method.set_root_length_bound
/// [`set_length_bound`]: struct.CalcRegex.html#method.set_length_bound
/// [The Meta-Language]: #the-meta-language
#[macro_export]
macro_rules! generate {
    // This macro makes heavy use of recursion for different purposes:
    //  - Encapsulation:
    //    Many patterns start with `@something`. These are basically
    //    independent sub-macros, which are included into the main macro by
    //    distinguishing their calls with the `@something` parameter.
    //  - Accumulators:
    //    Rust macros cannot match ambiguous patters. Thus, something like "Any
    //    number of arbitrary symbols until a semicolon" cannot trivially be
    //    matched. However, symbols grouped in parentheses are matched as a
    //    single token tree (`tt`). The different accumulators
    //    (`@accum_something`) are called with an initial empty set of
    //    parentheses and then call themselves recursively, gradually moving
    //    symbols from outside the parentheses inside. Eventually, the wanted
    //    pattern can be matched.
    //  - Partial parsing:
    //    Parts of regexes that are linked by operators of low precedence,
    //    like `,` and `|` can be parsed partially and just be concatenated
    //    later. Respective patterns match with an arbitrary *tail* and the
    //    end, which is a valid regular expression itself and can thus be
    //    parsed by recursively calling the sub-macro again.

    ///////////////////////////////////////////////////////////////////////////
    //      Regex
    ///////////////////////////////////////////////////////////////////////////

    // Parse Regex
    //
    // Parses the right-hand side of an assignment.
    //
    // A `Regex` is recursively built up: With the first parameter the regex so
    // far is passed through. Processed productions are added to it one by one.
    //
    // Most of these macro patterns create a new `Regex` `el` from the previous
    // `Regex` `prev`, and then call this sub-macro recursively with `el` and
    // the remaining input.

    // Start from an empty regex.
    (@parse_regex None , $($tail:tt)*) => ({
        generate!(@parse_regex $crate::generate::Regex::new(), $($tail)*)
    });

    // The empty statement.
    // We are done parsing.
    (@parse_regex $re:expr , ) => ({
        $re
    });

    // Matches concatenated statements. Only the comma and right-hand side is
    // matched; the left-hand side was matched with previous calls (recursion)
    // to this sub-macro. Resulting values are concatenated, so just parse the
    // remaining elements.
    // Mind the double comma: the first one is part of the syntax for calling
    // this sub-macro, the second one is input.
    (@parse_regex $prev:expr , , $($tail:tt)*) => ({
        generate!(@parse_regex $prev, $($tail)*)
    });

    // Matches choice statement. Like above, only the `|` and the right-hand
    // side is matched. It gets concatenated elsewhere.
    (@parse_regex $prev:expr , | $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::Choice.apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Matches the Kleene Star.
    (@parse_regex $prev:expr , $el:tt * $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::KleeneStar(
            &generate!(@parse_regex None, $el)
        ).apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Matches the Kleene Plus.
    (@parse_regex $prev:expr , $el:tt + $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::KleenePlus(
            &generate!(@parse_regex None, $el)
        ).apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Matches constant repeat.
    (@parse_regex $prev:expr , $el:tt ^ $n:tt $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::Repeat(
            &generate!(@parse_regex None, $el),
            $n
        ).apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Matches an identifier, i.e. a variable holding some previously generated
    // regex.
    (@parse_regex $prev:expr , $interim:ident $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::Identifier(
            &$interim
        ).apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Matches any statement in parentheses.
    (@parse_regex $prev:expr , ($($el:tt)*) $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::Parentheses(
            &generate!(@parse_regex None, $($el)*)
        ).apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Matches a range given by two characters.
    (@parse_regex $prev:expr , $min:tt - $max:tt $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::CharRange(
            $min, $max
        ).apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Matches a range given by two hex values.
    (@parse_regex $prev:expr , % $min:tt - % $max:tt $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::HexRange(
            stringify!($min), stringify!($max)
        ).apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Matches a single hex value.
    (@parse_regex $prev:expr , % $v:tt $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::ByteLiteral(
            stringify!($v)
        ).apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Matches a literal. Needs to be last matching rule, because otherwise the
    // compiler would try to apply the different operators directly onto the
    // components. The literal has to be escaped in order to not mess with the
    // regex syntax.
    (@parse_regex $prev:expr , $literal:tt $($tail:tt)*) => ({
        let el = $crate::generate::RegexProduction::Literal(
            &$literal
        ).apply($prev);
        generate!(@parse_regex el, $($tail)*)
    });

    // Accum Regex
    //
    // Accumulate the right-hand side of a non-restricted production until the
    // line-ending semicolon.

    // We have reached the semicolon and end of file. Parse the accumulated
    // value and return it as CalcRegex.
    (@accum_regex $calc_regex:ident $name:ident ($($accum:tt)*) ;) => ({
        let re = generate!(@parse_regex None, $($accum)*);
        let name = Some(stringify!($name).to_owned());
        $crate::generate::CalcRegexProduction::Regex(&re)
            .apply(&mut $calc_regex, name)
    });

    // We have reached the semicolon. Parse the accumulated value.
    (@accum_regex $calc_regex:ident $name:ident
     ($($accum:tt)*) ;
     $($tail:tt)*
    ) => ({
        let $name = $crate::generate::Interim::Regex(
            generate!(@parse_regex None, $($accum)*));
        generate!(@read_lines $calc_regex $($tail)*)
    });

    // We have not reached the semicolon yet. Add one more symbol.
    (@accum_regex $calc_regex:ident $name:ident
     ($($accum:tt)*) $next:tt $($tail:tt)*
    ) => ({
        generate!(@accum_regex $calc_regex $name ($($accum)* $next) $($tail)*)
    });

    // "=" Production

    // A new assignment. Introduce new pair of parentheses and start
    // accumulation.
    (@read_lines $calc_regex:ident $name:ident = $($tail:tt)*) => ({
        generate!(@accum_regex $calc_regex $name () $($tail)*)
    });

    ///////////////////////////////////////////////////////////////////////////
    //      Calc Regex
    ///////////////////////////////////////////////////////////////////////////

    // Parse Calc Regex
    //
    // Calc regexes are parsed very similarly to regexes, but the macro call
    // has an additional parameter, that is either 0 or 1. Initially,
    // @parse_calc_regex is invoked with 0. If we can't match with some basic
    // rule, we start looking for a ',' by accumulating components inside
    // parentheses like we do to find the semicolon at the end of a line.
    // @parse_calc_regex is called again either on the part to the comma if one
    // is found, or on the whole list. However, this time with a 1 for our
    // parameter to indicate that the same process is not to be repeated. If we
    // don't get a match this time, we try to match non-restricted production
    // rules.

    // Matches an interim value, i.e. a variable. An interim value can either
    // already be a CalcRegex or still a String representing a regex. This
    // either uses the existing CalcRegex (giving it a new name), or generates
    // a new one.
    (@parse_calc_regex
     $calc_regex:ident
     $_c:tt
     $name:expr,
     $interim:ident
    ) => ({
        $crate::generate::CalcRegexProduction::Identifier(
            &$interim, stringify!($interim).to_owned()
        ).apply(&mut $calc_regex, $name)
    });

    // Parentheses. CalcRegexes provide operator precedence through their
    // graph structure. No further grouping required.
    (@parse_calc_regex
     $calc_regex:ident
     $_c:tt
     $name:expr,
     ($($el:tt)*)
    ) => ({
        generate!(@parse_calc_regex $calc_regex 0 $name, $($el)*)
    });

    // Repeat.
    (@parse_calc_regex
     $calc_regex:ident
     $_c:tt
     $name:expr,
     $el:ident ^ $n:expr
    ) => ({
        $crate::generate::CalcRegexProduction::Repeat(
            generate!(@parse_calc_regex $calc_regex 0 None, $el),
            $n
        ).apply(&mut $calc_regex, $name)
    });

    // Matches any counted value. Leaves further handling to `@accum_counted`.
    (@parse_calc_regex
     $calc_regex:ident
     $_c:tt
     $name:expr,
     $r:tt . $f:ident , $($tail:tt)*
    ) => ({
        generate!(@accum_counted $calc_regex $name, $r $f () $($tail)*)
    });

    // No basic production matches. Try to find comma-separated parts that can
    // be matched.
    (@parse_calc_regex
     $calc_regex:ident
     0
     $name:expr,
     $($tail:tt)*
    ) => ({
        generate!(@accum_partial $calc_regex $name, () $($tail)*)
    });

    // No restricted production matches. Match against regular productions,
    // allowing only (non-calc) regular expressions.
    (@parse_calc_regex
     $calc_regex:ident
     1
     $name:expr,
     $($re:tt)*
    ) => ({
        let re = generate!(@parse_regex None, $($re)*);
        $crate::generate::CalcRegexProduction::Regex(&re)
            .apply(&mut $calc_regex, $name)
    });

    // Accum Partial
    //
    // Accumulate left-hand side of concatenated calc-regex.

    // Found a comma. Parse the left-hand side and the right-hand side
    // separately, concatenating the resulting `CalcRegex`es.
    (@accum_partial
     $calc_regex:ident
     $name:expr,
     ($($accum:tt)*) , $($tail:tt)*
    ) => ({
        $crate::generate::CalcRegexProduction::Concat(
            generate!(@parse_calc_regex $calc_regex 1 None, $($accum)*),
            generate!(@parse_calc_regex $calc_regex 0 None, $($tail)*),
        ).apply(&mut $calc_regex, $name)
    });

    // Went through the entire tail without finding a comma. Try parsing as
    // (non-calc) regex.
    (@accum_partial
     $calc_regex:ident
     $name:expr,
     ($($accum:tt)*)
    ) => ({
        generate!(@parse_calc_regex $calc_regex 1 $name, $($accum)*)
    });

    // Didn't match anything yet. Add one more element.
    (@accum_partial
     $calc_regex:ident
     $name:expr,
     ($($accum:tt)*) $next:tt $($tail:tt)*
    ) => ({
        generate!(
            @accum_partial $calc_regex
            $name, ($($accum)* $next) $($tail)*
        )
    });

    // Accum Counted
    //
    // Accumulate the in-between value `s` of counted productions, i.e. `r.f,
    // s, t#f` and `r.f, s, t^f`. Also used when no `s` exists.

    // LengthCount without in-between value. If there is an additional value
    // following, the respective pattern below matches and calls @accum_counted
    // again matching this one.
    //
    // Version with Kleene Star.
    // A Kleene Star on a calc-regex is only allowed at this exact point, so
    // match it here instead of always.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident () ($t:tt *) # $f_:ident
    ) => ({
        assert_eq!(stringify!($f), stringify!($f_));
        $crate::generate::CalcRegexProduction::LengthCount {
            r: generate!(@parse_calc_regex $calc_regex 0 None, $r),
            s: None,
            t: $crate::generate::CalcRegexProduction::KleeneStar(
                generate!(@parse_calc_regex $calc_regex 0 None, $t)
            ).apply(&mut $calc_regex, None),
            f: Box::new($f),
        }.apply(&mut $calc_regex, $name)
    });

    // LengthCount without in-between value.
    //
    // Version without Kleene Star.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident () $t:tt # $f_:ident
    ) => ({
        assert_eq!(stringify!($f), stringify!($f_));
        $crate::generate::CalcRegexProduction::LengthCount {
            r: generate!(@parse_calc_regex $calc_regex 0 None, $r),
            s: None,
            t: generate!(@parse_calc_regex $calc_regex 0 None, $t),
            f: Box::new($f),
        }.apply(&mut $calc_regex, $name)
    });

    // OccurrenceCount without in-between value.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident () $t:tt ^ $f_:ident
    ) => ({
        assert_eq!(stringify!($f), stringify!($f_));
        $crate::generate::CalcRegexProduction::OccurrenceCount {
            r: generate!(@parse_calc_regex $calc_regex 0 None, $r),
            s: None,
            t: generate!(@parse_calc_regex $calc_regex 0 None, $t),
            f: Box::new($f),
        }.apply(&mut $calc_regex, $name)
    });

    // LengthCount with in-between value. If there is an additional value
    // following, the respective pattern below matches and calls @accum_counted
    // again matching this one.
    //
    // Version with Kleene Star.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident ($($accum:tt)*) , ($t:tt *) # $f_:ident
    ) => ({
        assert_eq!(stringify!($f), stringify!($f_));
        $crate::generate::CalcRegexProduction::LengthCount {
            r: generate!(@parse_calc_regex $calc_regex 0 None, $r),
            s: Some(
               generate!(@parse_calc_regex $calc_regex 0 None, $($accum)*)
            ),
            t: $crate::generate::CalcRegexProduction::KleeneStar(
                generate!(@parse_calc_regex $calc_regex 0 None, $t)
            ).apply(&mut $calc_regex, None),
            f: Box::new($f),
        }.apply(&mut $calc_regex, $name)
    });

    // LengthCount with in-between value.
    //
    // Version without Kleene Star.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident ($($accum:tt)*) , $t:tt # $f_:ident
    ) => ({
        assert_eq!(stringify!($f), stringify!($f_));
        $crate::generate::CalcRegexProduction::LengthCount {
            r: generate!(@parse_calc_regex $calc_regex 0 None, $r),
            s: Some(
               generate!(@parse_calc_regex $calc_regex 0 None, $($accum)*)
            ),
            t: generate!(@parse_calc_regex $calc_regex 0 None, $t),
            f: Box::new($f),
        }.apply(&mut $calc_regex, $name)
    });
    // OccurrenceCount with in-between value.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident ($($accum:tt)*) , $t:tt ^ $f_:ident
    ) => ({
        assert_eq!(stringify!($f), stringify!($f_));
        $crate::generate::CalcRegexProduction::OccurrenceCount {
            r: generate!(@parse_calc_regex $calc_regex 0 None, $r),
            s: Some(
               generate!(@parse_calc_regex $calc_regex 0 None, $($accum)*)
            ),
            t: generate!(@parse_calc_regex $calc_regex 0 None, $t),
            f: Box::new($f),
        }.apply(&mut $calc_regex, $name)
    });

    // `LengthCount` without in-between value and following value.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident () $t:tt # $f_:ident , $($tail:tt)*
    ) => ({
        $crate::generate::CalcRegexProduction::Concat(
            generate!(@accum_counted $calc_regex None, $r $f () $t # $f_),
            generate!(@parse_calc_regex $calc_regex 0 None, $($tail)*),
        ).apply(&mut $calc_regex, $name)
    });

    // `OccurrenceCount` without in-between value and following value.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident () $t:tt ^ $f_:ident , $($tail:tt)*
    ) => ({
        $crate::generate::CalcRegexProduction::Concat(
            generate!(@accum_counted $calc_regex None, $r $f () $t ^ $f_),
            generate!(@parse_calc_regex $calc_regex 0 None, $($tail)*),
        ).apply(&mut $calc_regex, $name)
    });

    // `LengthCount` with in-between value and following value.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident ($($accum:tt)*) , $t:tt # $f_:ident , $($tail:tt)*
    ) => ({
        $crate::generate::CalcRegexProduction::Concat(
            generate!(
                @accum_counted
                $calc_regex
                None,
                $r $f ($($accum)*) , $t # $f_
            ),
            generate!(
                @parse_calc_regex
                $calc_regex
                0
                None,
                $($tail)*
            ),
        ).apply(&mut $calc_regex, $name)
    });

    // `OccurrenceCount` with in-between value and following value.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident ($($accum:tt)*) , $t:tt ^ $f_:ident , $($tail:tt)*
    ) => ({
        $crate::generate::CalcRegexProduction::Concat(
            generate!(
                @accum_counted
                $calc_regex
                None,
                $r $f ($($accum)*) , $t ^ $f_
            ),
            generate!(
                @parse_calc_regex
                $calc_regex
                0
                None,
                $($tail)*
            ),
        ).apply(&mut $calc_regex, $name)
    });

    // No match found yet. Add one more element.
    (@accum_counted
     $calc_regex:ident
     $name:expr,
     $r:tt $f:ident ($($accum:tt)*) $next:tt $($tail:tt)*
    ) => ({
        generate!(
            @accum_counted
            $calc_regex
            $name,
            $r $f ($($accum)* $next) $($tail)*
        )
    });

    // Accum Calc Regex
    //
    // Accumulate the right-hand side of a restricted production until the
    // line-ending semicolon.

    // We have reached the semicolon and end of file. Parse the accumulated
    // value and return it.
    (@accum_calc_regex $calc_regex:ident $name:ident
     ($($accum:tt)*) ;
    ) => ({
        generate!(
            @parse_calc_regex
            $calc_regex
            0
            Some(stringify!($name).to_owned()),
            $($accum)*
        )
    });

    // We have reached the semicolon. Parse the accumulated value and save for
    // later use.
    (@accum_calc_regex $calc_regex:ident $name:ident
     ($($accum:tt)*) ;
     $($tail:tt)*
    ) => ({
        let $name = $crate::generate::Interim::CalcRegex(
            generate!(
                @parse_calc_regex $calc_regex
                0
                Some(stringify!($name).to_owned()),
                $($accum)*
            )
        );
        generate!(@read_lines $calc_regex $($tail)*)
    });

    // We have not reached the semicolon yet. Add one more symbol.
    (@accum_calc_regex $calc_regex:ident $name:ident
     ($($accum:tt)*) $next:tt $($tail:tt)*
    ) => ({
        generate!(
            @accum_calc_regex
            $calc_regex
            $name
            ($($accum)* $next) $($tail)*
        )
    });

    // ":=" Production

    // A new assignment. Introduce new pair of parentheses and start
    // accumulation.
    (@read_lines $calc_regex:ident $name:ident := $($tail:tt)*) => ({
        generate!(@accum_calc_regex $calc_regex $name () $($tail)*)
    });

    ($($lines:tt)*) => ({
        let mut calc_regex = $crate::CalcRegex::new();
        let root = generate!(@read_lines calc_regex $($lines)*);
        calc_regex.set_root(root);
        calc_regex
    });

}
