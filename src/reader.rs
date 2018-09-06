/*!
This module provides types that allow to parse different kinds of input against
calc-regular expressions and to retrieve captured values.

See [Reader, Record, and Capture] for a brief introduction to the types of this
module.

[Reader, Record, and Capture]: ../index.html#reader-record-and-capture
*/

use std::cmp;
use std::collections::HashMap;
use std::io;
use std::iter;
use std::mem;
use std::ops::Deref;

use regex::bytes::Regex;

use calc_regex::{CalcRegex, NodeIndex};
use error::{NameError, NameResult, ParserError, ParserResult};

/// An abstract reader to parse input against a calc-regular expressions.
///
/// Different kinds of input are represented by the `Input` trait.
/// Implementations are provided for byte arrays (`&[u8]`) and for byte streams
/// (`io::Read`).
///
/// Use either [`from_array`] or [`from_stream`] to initialize a `Reader` with
/// the corresponding one.
///
/// [`from_array`]: #method.from_array
/// [`from_stream`]: #method.from_stream
#[derive(Debug)]
pub struct Reader<I: Input> {
    input: I,
    /// A stack to keep track of the capturing process.
    ///
    /// Captures build up a hierarchy, where captures that encompass others,
    /// are their parents.
    ///
    /// For each level of the capture hierarchy, an entry is added to the stack
    /// when the capture is started and removed from the stack, when the
    /// capture hits its end point. At that point, the finished capture will be
    /// added to the now-top entry of the stack, which is its parent in the
    /// hierarchy.
    captures: Vec<(String, Capture)>,
}

impl<'a> Reader<ArrayInput<'a>> {
    /// Creates a `Reader` from a byte array reference.
    ///
    /// # Examples
    ///
    /// ```
    /// # use calc_regex::Reader;
    /// let array_reader = Reader::from_array(b"foo");
    /// ```
    pub fn from_array(input: &'a [u8]) -> Self {
        Reader::new(input)
    }
}

impl<R: io::Read> Reader<StreamInput<R>> {
    /// Creates a `Reader` from an
    /// [`io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html) stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// # use std::io;
    /// # use calc_regex::Reader;
    ///
    /// # fn foo() -> io::Result<()> {
    /// let f = File::open("foo.txt")?;
    /// let stream_reader = Reader::from_stream(f);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_stream(input: R) -> Self {
        Reader::new(input)
    }
}

/// Basic functions.
impl<I: Input> Reader<I> {
    /// Creates a new `Reader` on the given `Input`.
    fn new(input: I::Source) -> Self {
        Reader {
            input: Input::new(input),
            captures: Vec::new(),
        }
    }

    /// Extracts the parsed bytes to a `Record`.
    ///
    /// Captures can be obtained from the `Record`. The `Reader` is ready again
    /// for parsing after this.
    fn get_record(&mut self) -> Record<I::Data> {
        if let (_, Capture::Single(capture)) = self.captures.pop().unwrap() {
            Record {
                capture,
                data: self.input.split_here(),
            }
        } else {
            panic!("Expected single capture.")
        }
    }
}

/// High-level methods for parsing `CalcRegex`es.
impl<I: Input> Reader<I> {
    /// Parses a single `CalcRegex` into a `Record`.
    ///
    /// Expects to parse the complete input. Otherwise a `TrailingCharacters`
    /// error is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate calc_regex;
    /// # use calc_regex::Reader;
    /// # fn main() {
    /// let re = generate!(
    ///     foo = "foo!";
    /// );
    ///
    /// let mut reader = Reader::from_array(b"foo!");
    /// let record = reader.parse(&re).unwrap();
    ///
    /// assert_eq!(record.get_all(), b"foo!");
    /// # }
    /// ```
    pub fn parse(
        &mut self,
        calc_regex: &CalcRegex,
    ) -> ParserResult<Record<I::Data>> {
        let root = calc_regex.get_root();
        self.init_capture(&root.name.as_ref().unwrap());
        match root.length_bound {
            Some(bound) => calc_regex.parse_bounded(self, root, bound)?,
            None => calc_regex.parse_unbounded(self, root)?,
        }
        self.finalize_capture(&root.name.as_ref().unwrap());
        if self.input.is_empty()? {
            Ok(self.get_record())
        } else {
            Err(ParserError::TrailingCharacters)
        }
    }

    /// Parses concatenated words of a given `CalcRegex`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate calc_regex;
    /// # use calc_regex::Reader;
    /// # fn main() {
    /// let re = generate!(
    ///     foo = "foo!";
    /// );
    ///
    /// let mut reader = Reader::from_array(b"foo!foo!foo!");
    /// for result in reader.parse_many(&re) {
    ///     let record = result.unwrap();
    ///     assert_eq!(record.get_all(), b"foo!");
    /// }
    /// # }
    /// ```
    pub fn parse_many(&mut self, calc_regex: &CalcRegex) -> RecordIter<I> {
        RecordIter {
            calc_regex: calc_regex.clone(),
            reader: self,
        }
    }

    /// Parse a single record when iterating `Record`s.
    ///
    /// Same as `parse`, but doesn't expect the input to be empty when done.
    fn parse_record(
        &mut self,
        calc_regex: &CalcRegex,
    ) -> ParserResult<Record<I::Data>> {
        let root = calc_regex.get_root();
        self.init_capture(&root.name.as_ref().unwrap());
        match root.length_bound {
            Some(bound) => calc_regex.parse_bounded(self, root, bound)?,
            None => calc_regex.parse_unbounded(self, root)?,
        }
        self.finalize_capture(&root.name.as_ref().unwrap());
        Ok(self.get_record())
    }
}

/// (Crate-) Internal functions.
///
/// Lower-level methods used by `Reader` itself and by `CalcRegex`.
impl<I: Input> Reader<I> {
    ///////////////////////////////////////////////////////////////////////////
    //      Parse Calc Regex
    ///////////////////////////////////////////////////////////////////////////

    /// Parses an unlimited number of bytes from input against the given node of
    /// `calc_regex`.
    ///
    /// This wraps `CalcRegex::parse_unbounded`, enforcing length bounds defined
    /// with the node and doing captures.
    pub(crate) fn parse_unbounded(
        &mut self,
        calc_regex: &CalcRegex,
        node_index: NodeIndex,
    ) -> ParserResult<usize> {
        let node = calc_regex.get_node(node_index);
        let start_pos = self.pos();
        if let Some(ref name) = node.name {
            self.start_capture(name);
        }
        match node.length_bound {
            Some(bound) => calc_regex.parse_bounded(self, node, bound)?,
            None => calc_regex.parse_unbounded(self, node)?,
        }
        if let Some(ref name) = node.name {
            self.finish_capture(name);
        }
        Ok(self.pos() - start_pos)
    }

    /// Parses a bounded number of bytes from input against the given node of
    /// `calc_regex`.
    ///
    /// This wraps `CalcRegex::parse_bounded`, enforcing additional length
    /// bounds defined with the node and doing captures.
    pub(crate) fn parse_bounded(
        &mut self,
        calc_regex: &CalcRegex,
        node_index: NodeIndex,
        bound: usize,
    ) -> ParserResult<usize> {
        let node = calc_regex.get_node(node_index);
        let start_pos = self.pos();
        if let Some(ref name) = node.name {
            self.start_capture(name);
        }
        let bound = node.length_bound.map_or(
            bound, |n| cmp::min(bound, n));
        calc_regex.parse_bounded(self, node, bound)?;
        if let Some(ref name) = node.name {
            self.finish_capture(name);
        }
        Ok(self.pos() - start_pos)
    }

    /// Parses an exact number of bytes from input against the given node of
    /// `calc_regex`.
    ///
    /// This wraps `CalcRegex::parse_exact`, enforcing length bounds defined
    /// with the node and doing captures.
    pub(crate) fn parse_exact(
        &mut self,
        calc_regex: &CalcRegex,
        node_index: NodeIndex,
        length: usize,
    ) -> ParserResult<()> {
        let node = calc_regex.get_node(node_index);
        if let Some(length_bound) = node.length_bound {
            if length_bound < length {
                return Err(ParserError::ConflictingBounds {
                    old: length,
                    new: length_bound,
                });
            }
        }
        if let Some(ref name) = node.name {
            self.start_capture(name);
        }
        calc_regex.parse_exact(self, node, length)?;
        if let Some(ref name) = node.name {
            self.finish_capture(name);
        }
        Ok(())
    }

    ///////////////////////////////////////////////////////////////////////////
    //      Match Regex
    ///////////////////////////////////////////////////////////////////////////

    /// Reads indefinitely many bytes from input until a given regex matches.
    pub(crate) fn match_regex_unbounded(
        &mut self,
        re: &Regex,
    ) -> ParserResult<()> {
        let start_pos = self.input.pos();
        while !re.is_match(&self.input.bytes()[start_pos..self.input.pos()]) {
            self.input.read_next()?;
        }
        Ok(())
    }

    /// Reads up to `bound` bytes from input until a given regex matches.
    pub(crate) fn match_regex_bounded(
        &mut self,
        re: &Regex,
        bound: usize,
    ) -> ParserResult<()> {
        if re.is_match(&[]) {
            return Ok(())
        }
        let start_pos = self.input.pos();
        for _ in 0..bound {
            self.input.read_next()?;
            if re.is_match(&self.input.bytes()[start_pos..self.input.pos()]) {
                return Ok(())
            }
        }
        Err(ParserError::Regex {
            regex: re.as_str().to_owned(),
            value: self.input.bytes()[start_pos..self.input.pos()].to_vec()
        })
    }

    /// Reads exactly `length` bytes from input and try to match given regex.
    pub(crate) fn match_regex_exact(
        &mut self,
        re: &Regex,
        length: usize,
    ) -> ParserResult<()> {
        let start_pos = self.input.pos();
        self.input.read_n(length)?;
        let value = &self.input.bytes()[start_pos..self.input.pos()];
        if re.is_match(value) {
           Ok(())
       } else {
           Err(ParserError::Regex {
               regex: re.as_str().to_owned(),
               value: value.to_vec(),
           })
       }
    }

    ///////////////////////////////////////////////////////////////////////////
    //      Capture
    ///////////////////////////////////////////////////////////////////////////

    /// Initializes capturing system for a new `Reader`.
    fn init_capture(&mut self, name: &str) {
        // Create a new capture instance for the stack. `end_pos` will be set
        // by `finalize_capture`.
        let capture = SingleCapture {
            start_pos: self.input.pos(),
            end_pos: 0,
            children: HashMap::new(),
        };
        // Push to stack.
        self.captures.push((
            name.to_owned(), // Currently the name is not really used.
            Capture::Single(capture),
        ));
    }

    /// Finalizes capturing system after expression has been read.
    fn finalize_capture(&mut self, name: &str) {
        debug_assert_eq!(self.captures.len(), 1);
        let &mut (ref saved_name, ref mut capture) =
            self.captures.last_mut().unwrap();
        debug_assert_eq!(name, saved_name);
        if let Capture::Single(ref mut capture) = *capture {
            capture.end_pos = self.input.pos();
        } else {
            panic!("Expected single capture.");
        }
        // Leave the last capture on the stack for `get_record()` to take.
    }

    /// Starts a repeat capture.
    pub(crate) fn start_repeat(&mut self) {
        self.captures.push((
            // We don't know its name at this point. It will be set when
            // `finish_capture` is called for the first repeat entry.
            "".to_owned(),
            Capture::Repeat(Vec::new()),
        ));
    }

    pub(crate) fn finish_repeat(&mut self) {
        // We dismantle the capture stack as we constructed it, thus, we expect
        // a repeat capture to be on top.
        let (name, repeat) = self.captures.pop().unwrap();
        let repeat = if let Capture::Repeat(repeat) = repeat {
            repeat
        } else {
            panic!("Expected repeat capture.");
        };
        // Look for the ancestor to commit our newly completed capture to. We
        // skip special captures with names starting with `$`.
        let (_, parent_capture) =
            self.get_last_where_mut(|ref name, _| !name.starts_with('$'))
                .unwrap();
        // We don't support directly nested repeat captures.
        let parent = match *parent_capture {
            Capture::Single(ref mut capture) => capture,
            Capture::Repeat(_) => panic!("Expected single capture."),
        };
        // Put the completed repeat capture in its position.
        parent.children.insert(
            name,
            Box::new(Capture::Repeat(repeat),
        ));
    }

    /// Sets current cursor position as starting point of new named capture.
    ///
    /// If we already saved a capture with the given name, we add a tick to it.
    pub(crate) fn start_capture(&mut self, name: &str) {
        // Create a new capture instance for the stack. `end_pos` will be set
        // by `finish_capture`.
        let capture = SingleCapture {
            start_pos: self.input.pos(),
            end_pos: 0,
            children: HashMap::new(),
        };
        // Add ticks to the name if necessary.
        let name = self.get_unique_name(name);
        // Push to stack.
        self.captures.push((
            name,
            Capture::Single(capture),
        ));
    }

    /// Sets current cursor position as ending point of most recent capture.
    ///
    /// Captures can't overlap. Thus we expect the given name to match the top
    /// entry of our stack of active captures.
    pub(crate) fn finish_capture(&mut self, name: &str) {
        // We dismantle the capture stack as we constructed it, thus, we expect
        // a single capture to be on top.
        let (saved_name, mut capture) = if let (
            saved_name,
            Capture::Single(capture),
        ) = self.captures.pop().unwrap() {
            (saved_name, capture)
        } else {
            panic!("Expected single capture.");
        };
        // Ticks might have be added to our saved name. The rest should match
        // though.
        debug_assert!(saved_name.starts_with(name));
        // This is what we are here for.
        capture.end_pos = self.input.pos();
        // Look for the ancestor to commit our newly completed capture to. We
        // skip special captures with names starting with `$`.
        let (parent_name, parent_capture) =
            self.get_last_where_mut(|ref name, _| !name.starts_with('$'))
                .unwrap();
        match *parent_capture {
            // If we are adding to a repeat capture, we push on its vector.
            Capture::Repeat(ref mut parent_captures) => {
                // If this is the first value of our repeat, we need to set its
                // name here because it was not known when we started the repeat
                // capture.
                if parent_captures.is_empty() {
                    debug_assert_eq!(*parent_name, "");
                    *parent_name = saved_name;
                } else {
                    debug_assert_eq!(*parent_name, saved_name);
                }
                parent_captures.push(capture);
            }
            // If we are adding to a single capture, we insert into its map of
            // children.
            Capture::Single(ref mut parent_capture) => {
                parent_capture.children.insert(
                    saved_name,
                    Box::new(Capture::Single(capture)),
                );
            }
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    //      Helper Functions
    ///////////////////////////////////////////////////////////////////////////

    /// Gets the `Reader`'s current cursor position.
    pub(crate) fn pos(&self) -> usize {
        self.input.pos()
    }

    /// Gets a slice of the input.
    pub(crate) fn get_range(&self, range: (usize, usize)) -> &[u8] {
        let (start, end) = range;
        &self.input.bytes()[start..end]
    }

    /// Traverses the capture stack in reverse and returns the first (name,
    /// capture) pair that satisfies the predicate.
    fn get_last_where<F>(&self, pred: F) -> Option<(&String, &Capture)>
    where
        F: Fn(&String, &Capture) -> bool,
    {
        for &(ref name, ref capture) in self.captures.iter().rev() {
            if pred(name, capture) {
                return Some((name, capture));
            }
        }
        None
    }

    /// Traverses the capture stack in reverse and returns the first (name,
    /// capture) pair that satisfies the predicate, mutable version.
    fn get_last_where_mut<F>(
        &mut self,
        pred: F,
    ) -> Option<(&mut String, &mut Capture)>
    where
        F: Fn(&String, &Capture) -> bool,
    {
        for &mut (ref mut name, ref mut capture) in
            self.captures.iter_mut().rev()
        {
            if pred(name, capture) {
                return Some((name, capture));
            }
        }
        None
    }

    /// Adds ticks (`'`) to the name until it is unique in its scope.
    fn get_unique_name(&self, name: &str) -> String {
        let mut name = name.to_owned();
        // Get last item on capture stack that is a single capture.
        //
        // We don't care for repeating names in repeat captures -- names are
        // supposed to repeat with those.
        let parent = self.get_last_where(|_, ref capture| {
            match **capture {
                Capture::Single(_) => true,
                Capture::Repeat(_) => false,
            }
        });
        if let Some((_, &Capture::Single(ref capture))) = parent {
            while capture.children.contains_key(&name) {
                name += "'";
            }
        }
        name
    }
}

/// A record of captured names, to be obtained by calling
/// [`parse`](struct.Reader.html#method.parse) on a
/// [`Reader`](struct.Reader.html).
///
/// Provides several methods to retrieve captured values.
///
/// Captured values can be obtained directly from `Record` using
/// [`get_capture`].
/// Values of repeated expressions (used in a repetition or an occurrence-count
/// production; see [The Meta-Language]) can be iterated over using
/// [`get_captures`].
/// One can get an intermediate [`SubRecord`] or an iterator over `SubRecord`s
/// using [`get_sub_record`] or [`get_sub_records`], respectively.
///
/// `SubRecord`s are much like normal `Record`s, but they don’t start at
/// the actual root expression when resolving capture names but at some point
/// further down the hierarchy that is defined by the name given to
/// `get_sub_record` or `get_sub_records`.
/// See [`get_capture`] for futher information on how capture names are
/// resolved.
///
/// [`get_capture`]: struct.Record.html#method.get_capture
/// [`get_captures`]: struct.Record.html#method.get_captures
/// [`get_sub_record`]: struct.Record.html#method.get_sub_record
/// [`get_sub_records`]: struct.Record.html#method.get_sub_records
/// [`SubRecord`]: struct.SubRecord.html
/// [The Meta-Language]: ../macro.generate.html#the-meta-language
#[derive(Debug)]
pub struct Record<D: Deref<Target = [u8]>> {
    capture: SingleCapture,
    data: D,
}

/// Functions for retrieving captured values.
///
/// The interface of `Record` matches that of
/// [`SubRecord`](struct.SubRecord.html).
impl<D: Deref<Target = [u8]>> Record<D> {
    /// Gets part of the parsed bytes by name.
    ///
    /// In general, all names that are part of restricted productions (`:=`)
    /// are captured, with the exception of names that are used in an
    /// unrestricted way, e.g. with a Kleene star (`*`).
    ///
    /// Names correspond to identifiers used in production rules and are
    /// qualified with a hierarchy, i.e. if you want to access the value of
    /// `bar`, that was part of the definition of `foo`, you would ask for
    /// `foo.bar` here.
    /// Top-level names are excluded from this.
    ///
    /// In case of repetitions, a number is added to the qualified name, e.g.
    /// `foo[0]`, `foo[1]` and so on, if `foo` is repeated.
    /// See [`get_captures`](#method.get_captures) for reading repeated
    /// captures using iterators.
    ///
    /// If a named expression occures more then once in the same production, a
    /// tick (`'`) is added for each existing expression of that name in that
    /// production.
    ///
    /// For length and occurrence counted productions, there are the special
    /// names `$count` and `$value`, which are themselves qualified as usual,
    /// but are not included in the qualification chain of names further down,
    /// e.g. for a production `number:decimal, (foo, byte*)#decimal`, you could
    /// get the value of `number` either by `number` or `$count`, the value of
    /// `(foo, byte*)` by `$value`, and the value of `foo` by `foo` (not
    /// `$value.foo`).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate calc_regex;
    /// # fn main() {
    /// let re = generate!(
    ///     foo = "foo!";
    ///     bar := foo ^ 2;
    ///     baz := foo, bar, foo;
    /// );
    ///
    /// let mut reader = calc_regex::Reader::from_array(b"foo!foo!foo!foo!");
    /// let record = reader.parse(&re).unwrap();
    ///
    /// assert_eq!(record.get_capture("foo").unwrap(), b"foo!");
    /// assert_eq!(record.get_capture("bar.foo[0]").unwrap(), b"foo!");
    /// assert_eq!(record.get_capture("bar.foo[1]").unwrap(), b"foo!");
    /// assert_eq!(record.get_capture("foo'").unwrap(), b"foo!");
    /// # }
    /// ```
    pub fn get_capture(&self, name: &str) -> NameResult<&[u8]> {
        let capture = self.get_single_capture(&self.capture, name)?;
        let start = capture.start_pos;
        let end = capture.end_pos;
        Ok(&self.data[start..end])
    }

    /// Like `get_capture()` but on repeated captures.
    ///
    /// Instead of a byte array, an iterator is returned which has byte arrays
    /// as its items.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate calc_regex;
    /// # fn main() {
    /// let re = generate!(
    ///     foo = "foo!";
    ///     bar := foo ^ 3;
    /// );
    ///
    /// let mut reader = calc_regex::Reader::from_array(b"foo!foo!foo!");
    /// let record = reader.parse(&re).unwrap();
    ///
    /// for capture in record.get_captures("foo").unwrap() {
    ///     assert_eq!(capture, b"foo!")
    /// }
    /// # }
    /// ```
    pub fn get_captures<'a>(
        &'a self,
        name: &str,
    ) -> NameResult<CaptureIter<'a, D>> {
        let captures = self.get_repeat_captures(&self.capture, name)?;
        Ok(CaptureIter {
            record: &self,
            captures,
            index: 0,
        })
    }

    /// Gets all bytes that were read and parsed.
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate calc_regex;
    /// # fn main() {
    /// let re = generate!(
    ///     foo = "foo!";
    /// );
    ///
    /// let mut reader = calc_regex::Reader::from_array(b"foo!");
    /// let record = reader.parse(&re).unwrap();
    ///
    /// assert_eq!(record.get_all(), b"foo!")
    /// # }
    /// ```
    pub fn get_all(&self) -> &[u8] {
        &self.data
    }

    /// Gets a sub record that represents the record at the given namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate calc_regex;
    /// # fn main() {
    /// let re = generate!(
    ///     foo = "foo!";
    ///     bar := foo;
    ///     baz := bar;
    /// );
    ///
    /// let mut reader = calc_regex::Reader::from_array(b"foo!");
    /// let record = reader.parse(&re).unwrap();
    ///
    /// let sub_record = record.get_sub_record("bar").unwrap();
    /// assert_eq!(sub_record.get_capture("foo").unwrap(), b"foo!");
    /// # }
    /// ```
    pub fn get_sub_record<'a>(
        &'a self,
        name: &str,
    ) -> NameResult<SubRecord<'a, D>> {
        let capture = self.get_single_capture(&self.capture, name)?;
        Ok(SubRecord {
            record: &self,
            capture,
        })
    }

    /// Like `get_sub_record()` but on repeated captures.
    ///
    /// Instead of a sub record, an iterator is returned which has sub records
    /// as its items.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate calc_regex;
    /// # fn main() {
    /// let re = generate!(
    ///     foo = "foo!";
    ///     bar := foo;
    ///     baz := bar ^ 3;
    /// );
    ///
    /// let mut reader = calc_regex::Reader::from_array(b"foo!foo!foo!");
    /// let record = reader.parse(&re).unwrap();
    ///
    /// for sub_record in record.get_sub_records("bar").unwrap() {
    ///     assert_eq!(sub_record.get_capture("foo").unwrap(), b"foo!");
    /// }
    /// # }
    /// ```
    pub fn get_sub_records<'a>(
        &'a self,
        name: &str,
    ) -> NameResult<SubRecordIter<'a, D>> {
        let captures = self.get_repeat_captures(&self.capture, name)?;
        Ok(SubRecordIter {
            record: &self,
            captures,
            index: 0,
        })
    }
}

/// Internal functions.
impl<D: Deref<Target = [u8]>> Record<D> {
    /// Returns `true` if there are no captures at all.
    #[cfg(test)]
    pub(crate) fn capture_is_empty(&self) -> bool {
        self.capture.children.is_empty()
    }

    /// Prints debugging information for all captures.
    #[cfg(test)]
    pub fn print_captures(&self) {
        println!("{:#?}", self.capture);
    }

    /// Returns capture by a qualified name.
    ///
    /// If the given name or a fragment of it belongs to a repeat capture, it
    /// must be indexed with square brackets.
    ///
    /// Uses `root` as starting point.
    fn get_single_capture<'a>(
        &'a self,
        root: &'a SingleCapture,
        name: &str,
    ) -> NameResult<&SingleCapture> {
        let mut current_capture = root;
        // Each fragment represents a level of our capture hierarchy. For each
        // fragment, try to find its name as child of `current_capture` and
        // update `current_capture` to the found capture.
        for mut fragment in name.split(".") {
            // Read the index, if any.
            let repeat_index: Option<usize> =
                fragment.find('[').map_or(Ok(None), |pos| {
                    if !fragment.ends_with(']') {
                        return Err(NameError::InvalidCaptureName {
                            message: "missing closing ']'",
                        });
                    }
                    let index_str = &fragment[pos + 1..fragment.len() - 1];
                    fragment = &fragment[0..pos];
                    index_str.parse::<usize>().map(Some).or(Err(
                        NameError::InvalidCaptureName {
                            message: "non-numeric index",
                        },
                    ))
                })?;
            if let Some(capture) = current_capture.children.get(fragment) {
                match **capture {
                    // A single capture is used directly.
                    Capture::Single(ref capture) => {
                        if repeat_index.is_some() {
                            return Err(NameError::MisplacedRepeatAccess {
                                name: fragment.to_owned(),
                            });
                        }
                        current_capture = capture;
                    }
                    // A repeat capture must be indexed.
                    Capture::Repeat(ref captures) => {
                        if let Some(repeat_index) = repeat_index {
                            if captures.len() <= repeat_index {
                                return Err(NameError::OutOfBounds {
                                    name: fragment.to_owned(),
                                    index: repeat_index,
                                    len: captures.len(),
                                });
                            }
                            current_capture = &captures[repeat_index];
                        } else {
                            return Err(NameError::MisplacedSingleAccess {
                                name: fragment.to_owned(),
                            });
                        }
                    }
                }
            } else {
                return Err(NameError::NoSuchName {
                    name: fragment.to_owned()
                });
            }
        }
        Ok(current_capture)
    }

    /// Returns repeat captures by a qualified name.
    ///
    /// The given name must belog to a repeat capture without giving an index
    /// in brackets (repeat captures in the qualification chain must still be
    /// indexed).
    ///
    /// Uses `root` as starting point.
    fn get_repeat_captures<'a>(
        &'a self,
        root: &'a SingleCapture,
        name: &str,
    ) -> NameResult<&Vec<SingleCapture>> {
        // Split once at the last `.`.
        let mut split = name.rsplitn(2, '.');
        let last = split.next().ok_or(NameError::InvalidCaptureName {
            message: "empty string"
        })?;
        // If there is at least one `.`, resolve the name in front of the last
        // one and go from there.
        let capture = if let Some(init) = split.next() {
            self.get_single_capture(root, init)?
        } else {
            root
        };
        if let Some(capture) = capture.children.get(last) {
            if let Capture::Repeat(ref captures) = **capture {
                Ok(captures)
            } else {
                Err(NameError::MisplacedRepeatAccess {
                    name: last.to_owned(),
                })
            }
        } else {
            Err(NameError::NoSuchName { name: last.to_owned() })
        }
    }
}

/// An iterator over `Record`s, to be obtained by calling
/// [`parse_many`](struct.Reader.html#method.parse_many) on a
/// [`Reader`](struct.Reader.html).
#[derive(Debug)]
pub struct RecordIter<'a, I: 'a + Input> {
    calc_regex: CalcRegex,
    reader: &'a mut Reader<I>,
}

impl<'a, I: Input> iter::Iterator for RecordIter<'a, I> {
    type Item = ParserResult<Record<I::Data>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.reader.input.is_empty() {
            Ok(false) => Some(self.reader.parse_record(&self.calc_regex)),
            Ok(true) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

/// A sub record represents a part of a record with a given namespace for
/// captures.
///
/// The public interface of `SubRecord` equals that of [`Record`].
///
/// The only difference is, that `SubRecord` holds a reference to its `Record`,
/// while `Record` owns its data.
/// That means, memory is freed when a `Record` and all its `SubRecord`s go out
/// of scope.
///
/// No data is copied when creating a `SubRecord`.
///
/// [`Record`]: struct.Record.html
#[derive(Debug)]
pub struct SubRecord<'a, D: 'a + Deref<Target = [u8]>> {
    record: &'a Record<D>,
    capture: &'a SingleCapture,
}

impl<'a, D: 'a + Deref<Target = [u8]>> SubRecord<'a, D> {
    /// Gets part of the parsed bytes by name.
    ///
    /// See [`Record`](struct.Record.html#method.get_capture) for further
    /// information.
    pub fn get_capture(&self, name: &str) -> NameResult<&[u8]> {
        let capture = self.record.get_single_capture(self.capture, name)?;
        Ok(&self.record.data[capture.start_pos..capture.end_pos])
    }

    /// Like `get_capture()` but on repeated captures.
    ///
    /// See [`Record`](struct.Record.html#method.get_captures) for further
    /// information.
    pub fn get_captures(&self, name: &str) -> NameResult<CaptureIter<'a, D>> {
        let captures = self.record.get_repeat_captures(&self.capture, name)?;
        Ok(CaptureIter {
            record: &self.record,
            captures,
            index: 0,
        })
    }

    /// Gets all bytes that were read and parsed.
    ///
    /// See [`Record`](struct.Record.html#method.get_all) for further
    /// information.
    pub fn get_all(&self) -> &[u8] {
        &self.record.data[self.capture.start_pos..self.capture.end_pos]
    }

    /// Gets a sub record that represents the record at the given namespace.
    ///
    /// See [`Record`](struct.Record.html#method.get_sub_record) for further
    /// information.
    pub fn get_sub_record(&self, name: &str) -> NameResult<SubRecord<'a, D>> {
        let capture = self.record.get_single_capture(self.capture, name)?;
        Ok(SubRecord {
            record: self.record,
            capture,
        })
    }

    /// Like `get_sub_record()` but on repeated captures.
    ///
    /// See [`Record`](struct.Record.html#method.get_sub_records) for further
    /// information.
    pub fn get_sub_records(
        &self,
        name: &str,
    ) -> NameResult<SubRecordIter<'a, D>> {
        let captures = self.record.get_repeat_captures(self.capture, name)?;
        Ok(SubRecordIter {
            record: self.record,
            captures,
            index: 0,
        })
    }
}

/// An iterator over [`SubRecord`](struct.SubRecord.html)s.
///
/// See [`Record::get_sub_records`](struct.Record.html#method.get_sub_records)
/// for usage examples.
#[derive(Debug)]
pub struct SubRecordIter<'a, D: 'a + Deref<Target = [u8]>> {
    record: &'a Record<D>,
    captures: &'a Vec<SingleCapture>,
    index: usize,
}

impl<'a, D: 'a + Deref<Target = [u8]>> iter::Iterator
    for SubRecordIter<'a, D>
{
    type Item = SubRecord<'a, D>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.captures.len() {
            let capture = &self.captures[self.index];
            self.index += 1;
            Some(SubRecord {
                record: self.record,
                capture: &capture,
            })
        } else {
            None
        }
    }
}

/// Either a single named capture or one of a repeated capture.
///
/// Captures can be nested. This is used to implement resolution of qualified
/// capture names as described in `get_capture`.
#[derive(Debug)]
struct SingleCapture {
    /// The starting position of the capture within the `Reader`'s or
    /// `Record`'s `input` / `data` buffer.
    start_pos: usize,
    /// The ending position of the capture within the `Reader`'s or `Record`'s
    /// `input` / `data` buffer.
    end_pos: usize,
    /// Captures that are further down in the hierarchy of capture names, i.e.
    /// that are part of the this capture.
    children: HashMap<String, Box<Capture>>,
}

/// Either a single named capture or a vector of captures sharing the same
/// name.
#[derive(Debug)]
enum Capture {
    Single(SingleCapture),
    Repeat(Vec<SingleCapture>),
}

/// An iterator over capture values in the form of byte arrays.
///
/// See [`Record::get_captures`](struct.Record.html#method.get_captures) for
/// usage examples.
#[derive(Debug)]
pub struct CaptureIter<'a, D: 'a + Deref<Target = [u8]>> {
    record: &'a Record<D>,
    captures: &'a Vec<SingleCapture>,
    index: usize,
}

impl<'a, D: 'a + Deref<Target = [u8]>> iter::Iterator for CaptureIter<'a, D> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.captures.len() {
            let capture = &self.captures[self.index];
            self.index += 1;
            Some(&self.record.data[capture.start_pos..capture.end_pos])
        } else {
            None
        }
    }
}

/// A replaceable type to provide input to a `Reader`.
///
/// Unless you want to implement your own input type, consider this internal to
/// the `Reader`.
pub trait Input {
    /// The input type.
    type Source;
    /// The type that is used to store the input data.
    type Data: Deref<Target = [u8]>;

    /// Creates a new `Input` from `Source`.
    fn new(input: Self::Source) -> Self;

    /// Returns the current position of the reader.
    ///
    /// This is equivalent to the number of bytes read.
    fn pos(&self) -> usize;

    /// Returns a slice of all read bytes.
    fn bytes(&self) -> &[u8];

    /// Reads the next byte.
    fn read_next(&mut self) -> ParserResult<()>;

    /// Reads `n` bytes.
    fn read_n(&mut self, n: usize) -> ParserResult<()>;

    /// Checks whether there are more bytes to read.
    ///
    /// Internal data might be modified by calling this, however the result of
    /// other functions called on `Input` must not be different after
    /// `is_empty()` is called from what it would have been otherwise.
    fn is_empty(&mut self) -> ParserResult<bool>;

    /// Returns and forgets about the data read until now.
    ///
    /// Leaves itself as if newly created, but keeps the `Source`.
    fn split_here(&mut self) -> Self::Data;
}

/// `Input` implementation for byte array.
pub struct ArrayInput<'a> {
    // `ArrayInput` just reads from a byte array reference, keeping the current
    // position to offer the same interface as when reading from a stream.
    input: &'a [u8],
    start: usize,
    pos: usize,
}

impl<'a> Input for ArrayInput<'a> {
    type Source = &'a [u8];
    type Data = &'a [u8];

    fn new(input: &'a [u8]) -> Self {
        ArrayInput {
            input,
            start: 0,
            pos: 0,
        }
    }

    fn pos(&self) -> usize {
        self.pos - self.start
    }

    fn bytes(&self) -> &[u8] {
        &self.input[self.start..self.pos]
    }

    fn read_next(&mut self) -> ParserResult<()> {
        if self.pos + 1 > self.input.len() {
            Err(ParserError::UnexpectedEof)
        } else {
            self.pos += 1;
            Ok(())
        }
    }

    fn read_n(&mut self, n: usize) -> ParserResult<()> {
        if self.pos + n > self.input.len() {
            Err(ParserError::UnexpectedEof)
        } else {
            self.pos += n;
            Ok(())
        }
    }

    fn is_empty(&mut self) -> ParserResult<bool> {
        Ok(self.pos == self.input.len())
    }

    fn split_here(&mut self) -> &'a [u8] {
        let ret = &self.input[self.start..self.pos];
        self.start = self.pos;
        ret
    }
}

/// `Input` implementation for `io::Read` stream.
pub struct StreamInput<R: io::Read> {
    // `StreamInput` reads from a `io::Read`, saving all data to a `Vec<u8>`.
    // Since `io::Read` cannot tell us whether there is more data to be read,
    // we introduce an additional `pos` field, that holds the number of bytes
    // we were supposed to read. In case `is_empty()` is called, we might
    // read more than that to find out if we can. Read functions have to update
    // `pos` explicitly.
    input: R,
    data: Vec<u8>,
    pos: usize,
}

impl<R: io::Read> Input for StreamInput<R> {
    type Source = R;
    type Data = Vec<u8>;

    fn new(input: R) -> Self {
        StreamInput {
            input,
            data: Vec::new(),
            pos: 0,
        }
    }

    fn pos(&self) -> usize {
        // Note that we do not return `self.data.len()`, but the position
        // explicitly saved.
        self.pos
    }

    fn bytes(&self) -> &[u8] {
        &self.data[0 .. self.pos]
    }

    fn read_next(&mut self) -> ParserResult<()> {
        // Check if we already read the requested byte.
        if self.data.len() > self.pos {
            self.pos += 1;
            return Ok(())
        }
        // Read one byte from the stream.
        let mut byte = [0u8];
        match self.input.read(&mut byte) {
            Ok(1) => {},
            Ok(0) => return Err(ParserError::UnexpectedEof),
            Err(err) => return Err(ParserError::IoError { err }),
            Ok(_) => panic!("Read more than 1 byte into 1-byte buffer!"),

        }
        self.data.push(byte[0]);
        self.pos += 1;
        Ok(())
    }

    fn read_n(&mut self, n: usize) -> ParserResult<()> {
        // Check if we already read the requested bytes.
        if n <= (self.data.len() - self.pos) {
            self.pos += n;
            return Ok(())
        }
        // Read the remaining bytes from the stream.
        let to_read = n - (self.data.len() - self.pos);
        let mut vec: Vec<u8> = Vec::with_capacity(to_read);
        vec.resize(to_read, 0u8);
        {
            let bytes = vec.as_mut_slice();
            match self.input.read_exact(bytes) {
                Ok(()) => {},
                Err(ref err) if err.kind() == io::ErrorKind::UnexpectedEof =>
                    return Err(ParserError::UnexpectedEof),
                Err(err) => return Err(ParserError::IoError { err }),
            }
        }
        self.data.append(&mut vec);
        self.pos += n;
        Ok(())
    }

    fn is_empty(&mut self) -> ParserResult<bool> {
        // Check if we already read more bytes from the stream than needed.
        if self.data.len() > self.pos {
            return Ok(false)
        }
        // Try to read another byte, not adding to `self.pos` if successful.
        let mut byte = [0u8];
        match self.input.read(&mut byte) {
            Ok(1) => {},
            Ok(0) => return Ok(true),
            Err(err) => return Err(ParserError::IoError { err }),
            Ok(_) => panic!("Read more than 1 byte into 1-byte buffer!"),

        }
        self.data.push(byte[0]);
        Ok(false)
    }

    fn split_here(&mut self) -> Vec<u8> {
        let mut data = self.data.split_off(self.pos);
        mem::swap(&mut data, &mut self.data);
        self.pos = 0;
        data
    }
}

#[cfg(test)]
mod tests {
    macro_rules! run_tests { ($name:ident, $get_reader:path) => { mod $name {
        use ::*;
        use super::super::*;

        #[test]
        fn input() {
            let reader = $get_reader("foo".as_bytes());
            let mut input = reader.input;
            assert_eq!(input.pos(), 0);
            assert_eq!(input.bytes(), [0u8; 0]);
            assert!(!input.is_empty().unwrap());
            assert_eq!(input.pos(), 0);
            assert_eq!(input.bytes(), [0u8; 0]);
            input.read_n(0).unwrap();
            assert_eq!(input.pos(), 0);
            assert_eq!(input.bytes(), [0u8; 0]);
            input.read_n(2).unwrap();
            assert_eq!(input.pos(), 2);
            assert_eq!(input.bytes(), ['f' as u8, 'o' as u8]);
            assert!(!input.is_empty().unwrap());
            assert_eq!(input.pos(), 2);
            assert_eq!(input.bytes(), ['f' as u8, 'o' as u8]);
            input.read_next().unwrap();
            assert_eq!(input.pos(), 3);
            assert_eq!(input.bytes(), ['f' as u8, 'o' as u8, 'o' as u8]);
            assert!(input.is_empty().unwrap());
            assert_eq!(input.pos(), 3);
            assert_eq!(input.bytes(), ['f' as u8, 'o' as u8, 'o' as u8]);
            if let Err(ParserError::UnexpectedEof) = input.read_next() {
            } else { panic!("Expected Error::UnexpectedEof") }
            assert!(input.is_empty().unwrap());
            assert_eq!(input.pos(), 3);
            assert_eq!(input.bytes(), ['f' as u8, 'o' as u8, 'o' as u8]);
            if let Err(ParserError::UnexpectedEof) = input.read_n(1) {
            } else { panic!("Expected Error::UnexpectedEof") }
            input.read_n(0).unwrap();
        }

        #[test]
        fn parse_bounded_tight() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.get_root_mut().length_bound = None;
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            reader.parse_bounded(&re, root, 6).unwrap();
            reader.finalize_capture("foo");
            let record = reader.get_record();
            assert_eq!(record.get_all(), b"foobar");
        }

        #[test]
        fn parse_bounded_within() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.get_root_mut().length_bound = None;
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            reader.parse_bounded(&re, root, 7).unwrap();
            reader.finalize_capture("foo");
            let record = reader.get_record();
            assert_eq!(record.get_all(), b"foobar");
        }

        #[test]
        fn parse_bounded_exceeded() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.get_root_mut().length_bound = None;
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_bounded(&re, root, 5).unwrap_err();
            if let ParserError::Regex { ref regex, ref value } = err {
                assert_eq!(regex, "^(?-u:([a-z]){6})$");
                assert_eq!(value, b"fooba");
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }

        #[test]
        fn parse_bounded_length_bound_smaller() {
            let re = generate! {
                foo = ("a" - "z")^6;
            };
            assert_eq!(re.get_root().length_bound, Some(6));
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            reader.parse_bounded(&re, root, 7).unwrap();
            reader.finalize_capture("foo");
            let record = reader.get_record();
            assert_eq!(record.get_all(), b"foobar");
        }

        #[test]
        fn parse_bounded_length_bound_smaller_exceeded() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.set_root_length_bound(5);
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_bounded(&re, root, 6).unwrap_err();
            if let ParserError::Regex { ref regex, ref value } = err {
                assert_eq!(regex, "^(?-u:([a-z]){6})$");
                assert_eq!(value, b"fooba");
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }

        #[test]
        fn parse_bounded_length_bound_bigger() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.set_root_length_bound(7);
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            reader.parse_bounded(&re, root, 6).unwrap();
            reader.finalize_capture("foo");
            let record = reader.get_record();
            assert_eq!(record.get_all(), b"foobar");
        }

        #[test]
        fn parse_bounded_length_bound_bigger_exceeded() {
            let re = generate! {
                foo = ("a" - "z")^6;
            };
            assert_eq!(re.get_root().length_bound, Some(6));
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_bounded(&re, root, 5).unwrap_err();
            if let ParserError::Regex { ref regex, ref value } = err {
                assert_eq!(regex, "^(?-u:([a-z]){6})$");
                assert_eq!(value, b"fooba");
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }

        #[test]
        fn parse_bounded_length_bound_equal() {
            let re = generate! {
                foo = ("a" - "z")^6;
            };
            assert_eq!(re.get_root().length_bound, Some(6));
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            reader.parse_bounded(&re, root, 6).unwrap();
            reader.finalize_capture("foo");
            let record = reader.get_record();
            assert_eq!(record.get_all(), b"foobar");
        }

        #[test]
        fn parse_bounded_length_bound_equal_exceeded() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.set_root_length_bound(5);
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_bounded(&re, root, 5).unwrap_err();
            if let ParserError::Regex { ref regex, ref value } = err {
                assert_eq!(regex, "^(?-u:([a-z]){6})$");
                assert_eq!(value, b"fooba");
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }

        #[test]
        fn parse_exact() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.get_root_mut().length_bound = None;
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            reader.parse_exact(&re, root, 6).unwrap();
            reader.finalize_capture("foo");
            let record = reader.get_record();
            assert_eq!(record.get_all(), b"foobar");
        }

        #[test]
        fn parse_exact_non_prefix_free() {
            let re = generate! {
                foo = ("a" - "z")*;
            };
            assert_eq!(re.get_root().length_bound, None);
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            reader.parse_exact(&re, root, 6).unwrap();
            reader.finalize_capture("foo");
            let record = reader.get_record();
            assert_eq!(record.get_all(), b"foobar");
        }

        #[test]
        fn parse_exact_short() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.get_root_mut().length_bound = None;
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_exact(&re, root, 7).unwrap_err();
            if let ParserError::UnexpectedEof = err {
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }

        #[test]
        fn parse_exact_exceeded() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.get_root_mut().length_bound = None;
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_exact(&re, root, 5).unwrap_err();
            if let ParserError::Regex { ref regex, ref value } = err {
                assert_eq!(regex, "^(?-u:([a-z]){6})$");
                assert_eq!(value, b"fooba");
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }

        #[test]
        fn parse_exact_length_bound_smaller() {
            let re = generate! {
                foo = ("a" - "z")^6;
            };
            assert_eq!(re.get_root().length_bound, Some(6));
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_exact(&re, root, 7).unwrap_err();
            if let ParserError::ConflictingBounds { old, new } = err {
                assert_eq!(old, 7);
                assert_eq!(new, 6);
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }

        #[test]
        fn parse_exact_length_bound_smaller_exceeded() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.set_root_length_bound(5);
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_exact(&re, root, 6).unwrap_err();
            if let ParserError::ConflictingBounds { old, new } = err {
                assert_eq!(old, 6);
                assert_eq!(new, 5);
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }

        #[test]
        fn parse_exact_length_bound_within() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.set_root_length_bound(7);
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            reader.parse_exact(&re, root, 6).unwrap();
            reader.finalize_capture("foo");
            let record = reader.get_record();
            assert_eq!(record.get_all(), b"foobar");
        }

        #[test]
        fn parse_exact_length_bound_bigger_exceeded() {
            let re = generate! {
                foo = ("a" - "z")^6;
            };
            assert_eq!(re.get_root().length_bound, Some(6));
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_exact(&re, root, 5).unwrap_err();
            if let ParserError::Regex { ref regex, ref value } = err {
                assert_eq!(regex, "^(?-u:([a-z]){6})$");
                assert_eq!(value, b"fooba");
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }

        #[test]
        fn parse_exact_length_bound_tight() {
            let re = generate! {
                foo = ("a" - "z")^6;
            };
            assert_eq!(re.get_root().length_bound, Some(6));
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            reader.parse_exact(&re, root, 6).unwrap();
            reader.finalize_capture("foo");
            let record = reader.get_record();
            assert_eq!(record.get_all(), b"foobar");
        }

        #[test]
        fn parse_exact_length_bound_equal_exceeded() {
            let mut re = generate! {
                foo = ("a" - "z")^6;
            };
            re.set_root_length_bound(5);
            let mut reader = $get_reader("foobar".as_bytes());
            reader.init_capture("foo");
            let root = re.get_root_index();
            let err = reader.parse_exact(&re, root, 5).unwrap_err();
            if let ParserError::Regex { ref regex, ref value } = err {
                assert_eq!(regex, "^(?-u:([a-z]){6})$");
                assert_eq!(value, b"fooba");
            } else {
                panic!("Unexpected error: {:?}", err)
            }
        }
    }}}
    run_tests!(array, Reader::from_array);
    run_tests!(stream, Reader::from_stream);
}
