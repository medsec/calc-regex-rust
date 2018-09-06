/*!
Internal module containing `CalcRegex`, a representation of a calc-regular
expression.
*/
use std::fmt;
use regex::bytes::Regex;

use error::{NameError, NameResult, ParserError, ParserResult};
use reader::{Input, Reader};

/// The type `CalcRegex` represents a calc-regular expression.
///
/// A `CalcRegex` instance is constructed with the [`generate!`] macro from a
/// meta-language, corresponding to the calc-regular expression it represents.
///
/// It can be applied to input using [`Reader`] and one of its parsing
/// functions.
///
/// [`generate!`]: macro.generate.html
/// [`Reader`]: reader/struct.Reader.html

// `CalcRegex` uses a tree-like directed acyclic graph structure to represent
// expressions. All nodes are stored in a flat vector. Edges are represented by
// the `NodeIndex` type, which is just a wrapped `usize`. This allows us to
// pass the whole `CalcRegex` around as mutable reference, which would not be
// trivially possible when using actual references for nodes. Additionally, we
// gain the option to easily clone the whole data structure.
//
// Since the graph structure is fixed after generation, we are save to not have
// invalid `NodeIndex`es, as long as we only use the ones corresponding to the
// correct `CalcRegex`.
#[derive(Clone, Debug)]
pub struct CalcRegex {
    /// A vector of all `Node`s used in the `CalcRegex`.
    nodes: Vec<Node>,
    /// Index of the root `Node`, on which parsing is started.
    root: NodeIndex,
}

/// A node of a `CalcRegex`.
///
/// A `CalcRegex` is constructed of these nodes. Each `Node` represents a
/// sub-expression, that can in turn contain other sub-expressions, represented
/// by other `Node`s. When following this chain, no circles are permitted.
///
/// `name` and `length_bound` are meta-data. `inner` holds the actual
/// sub-expression represented by this `Node`.
#[derive(Clone, Debug)]
pub(crate) struct Node {
    /// Name of this sub-expression.
    ///
    /// A name must be unique in a `CalcRegex`. It is used to pick a `Node`
    /// from a `CalcRegex` and to obtain captures from parsed input.
    pub name: Option<String>,
    /// The maximal number of bytes, that should be parsed from input when
    /// trying to match this sub-expression.
    pub length_bound: Option<usize>,
    /// The actual sub-expression.
    pub inner: Inner,
}

/// An index referring to the position of a `Node` within `CalcRegex`'es
/// `nodes` vector.
///
/// This is public so it can be passed around by `generate!`, however by using
/// a tuple struct, we hide its value.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeIndex(usize);

/// Possible sub-expressions in a `CalcRegex`.
///
/// In a `CalcRegex`, a directed acyclic graph of Nodes is built up, each
/// holding an instance of `Inner`. Each Path of this graph points eventually
/// to a `Regex` variant of `Inner`. The other variants represent a
/// sub-expression that consists of one or more other sub-expressions
/// represented by other `Node`s.
///
/// The variants of `Inner` represent the valid production rules of
/// calc-regular expressions.
#[derive(Clone)]
pub(crate) enum Inner {
    Regex(Regex),
    CalcRegex(NodeIndex),
    Concat(NodeIndex, NodeIndex),
    Repeat(NodeIndex, usize),
    KleeneStar(NodeIndex),
    /// `(r.f)s(t#f)`
    LengthCount {
        r: NodeIndex,
        s: Option<NodeIndex>,
        t: NodeIndex,
        f: Box<fn(&[u8]) -> Option<usize>>,
    },
    /// `(r.f)s(t^f)`
    OccurrenceCount {
        r: NodeIndex,
        s: Option<NodeIndex>,
        t: NodeIndex,
        f: Box<fn(&[u8]) -> Option<usize>>,
    },
}

// `Debug` cannot be derived for `CalcRegexChoice` because it cannot be derived
// for `f`. Implement it omitting `f`.
impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Inner::Regex(ref re) =>
                f.debug_tuple("Regex")
                    .field(re)
                    .finish(),
            Inner::CalcRegex(node_index) =>
                f.debug_tuple("CalcRegex")
                    .field(&node_index)
                    .finish(),
            Inner::Concat(lhs, rhs) =>
                f.debug_tuple("Concat")
                    .field(&lhs)
                    .field(&rhs)
                    .finish(),
            Inner::Repeat(node_index, count) =>
                f.debug_tuple("Repeat")
                    .field(&node_index)
                    .field(&count)
                    .finish(),
            Inner::KleeneStar(node_index) =>
                f.debug_tuple("KleeneStar")
                    .field(&node_index)
                    .finish(),
            Inner::LengthCount { r, s, t, .. } =>
                f.debug_struct("LengthCount")
                    .field("r", &r)
                    .field("s", &s)
                    .field("t", &t)
                    .finish(),
            Inner::OccurrenceCount { r, s, t, .. } =>
                f.debug_struct("OccurrenceCount")
                    .field("r", &r)
                    .field("s", &s)
                    .field("t", &t)
                    .finish(),
        }
    }
}

/// Public functions.
///
/// `name` in this context refers to an identifier used in an assignment inside
/// `generate!`.
/// `bound` refers to a number of bytes.
impl CalcRegex {
    /// Sets the subexpression with the given name to be the root expression.
    ///
    /// The root expression is the one that input is parsed against.
    /// By default that is the last expression given to `generate!`.
    pub fn set_root_by_name(&mut self, name: &str) -> NameResult<()> {
        let pos = self.get_position_by_name(name)
            .ok_or(NameError::NoSuchName { name: name.to_owned() })?;
        self.set_root(pos);
        Ok(())
    }

    /// Adds a length bound to the root expression.
    ///
    /// Parsing will be aborted as soon as the bound is exceeded or a
    /// length-counted expression is encountered that would exceed it when
    /// parsed.
    pub fn set_root_length_bound(&mut self, bound: usize) {
        let ref mut root = self.nodes[self.root.0];
        root.length_bound = Some(bound);
    }

    /// Adds a length bound to the subexpression with the given name.
    ///
    /// Parsing will be aborted as soon as the bound is exceeded or a
    /// length-counted expression is encountered that would exceed it when
    /// parsed.
    pub fn set_length_bound(
        &mut self,
        name: &str,
        bound: usize
    ) -> NameResult<()> {
        let ref mut node = self.get_node_mut_by_name(name)
            .ok_or(NameError::NoSuchName { name: name.to_owned() })?;
        node.length_bound = Some(bound);
        Ok(())
    }
}

/// Internal functions.
///
/// Might be public to allow usage by `generate!`.
#[doc(hidden)]
impl CalcRegex {
    /// Creates a new, empty `CalcRegex`.
    pub fn new() -> Self {
        CalcRegex {
            nodes: Vec::new(),
            root: NodeIndex(0),
        }
    }

    /// Returns a reference to the current root node of the `CalcRegex`.
    pub(crate) fn get_root(&self) -> &Node {
        &self.nodes[self.root.0]
    }

    /// Returns a mutable reference to the current root node of the
    /// `CalcRegex`.
    #[cfg(test)]
    pub(crate) fn get_root_mut(&mut self) -> &mut Node {
        &mut self.nodes[self.root.0]
    }

    /// Returns the index of the current root node of the `CalcRegex`.
    #[cfg(test)]
    pub(crate) fn get_root_index(&self) -> NodeIndex {
        self.root
    }

    /// Changes the current root of the `CalcRegex` by a given index.
    pub fn set_root(&mut self, node_index: NodeIndex) {
        self.root = node_index;
    }

    /// Gets a reference to a node of the `CalcRegex` by index.
    pub(crate) fn get_node(&self, node_index: NodeIndex) -> &Node {
        &self.nodes[node_index.0]
    }

    /// Gets the index of a node by name.
    ///
    /// Returns `None`, if the given name doesn't exist.
    fn get_position_by_name(&self, name: &str) -> Option<NodeIndex> {
        self.nodes.iter().position(|ref node| {
            node.name.as_ref().map_or(false, |node_name| node_name == name)
        }).map(NodeIndex)
    }

    /// Gets a mutable reference to a node by name.
    ///
    /// Returns `None`, if the given name doesn't exist.
    fn get_node_mut_by_name(&mut self, name: &str) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|ref node| {
            node.name.as_ref().map_or(false, |node_name| node_name == name)
        })
    }

    /// Appends the given node to saved nodes and returns its index.
    pub(crate) fn push_node(&mut self, node: Node) -> NodeIndex {
        // Names must be unique.
        if let Some(ref name) = node.name {
            assert!(!self.nodes.iter().any(|node| {
                node.name.as_ref() == Some(name)
            }), "A node named \"{}\" already exists!", name);
        }
        let node_index = NodeIndex(self.nodes.len());
        self.nodes.push(node);
        node_index
    }

    /// Parses an unlimited number of bytes from the given `Reader` against the
    /// sub-expression represented by the given `Node`.
    ///
    /// The given `Node` has to be part of the `CalcRegex`, this method is
    /// called on.
    ///
    /// This method is supposed to be called from a parsing routine of
    /// `Reader`. It deconstructs the given `Node`, either matching its
    /// containing regex, or using the `Reader` again to parse its
    /// sub-expressions.
    pub(crate) fn parse_unbounded<I: Input>(
        &self,
        reader: &mut Reader<I>,
        node: &Node,
    ) -> ParserResult<()> {
        match node.inner {
            Inner::Regex(ref regex) => {
                reader.match_regex_unbounded(regex)?;
            }
            Inner::CalcRegex(node_index) => {
                reader.parse_unbounded(self, node_index)?;
            }
            Inner::Concat(r, s) => {
                reader.parse_unbounded(self, r)?;
                reader.parse_unbounded(self, s)?;
            }
            Inner::Repeat(node_index, n) => {
                reader.start_repeat();
                for _ in 0..n {
                    reader.parse_unbounded(self, node_index)?;
                }
                reader.finish_repeat();
            }
            Inner::KleeneStar(_) => {
                panic!("KleeneStar can only be parsed with parse_exact().")
            }
            Inner::LengthCount { r, s, t, ref f } => {
                let count = self.read_count(reader, f, &mut |reader| {
                    reader.parse_unbounded(self, r)?;
                    Ok(())
                })?;
                if let Some(node_index) = s {
                    reader.parse_unbounded(self, node_index)?;
                }
                reader.start_capture("$value");
                reader.parse_exact(self, t, count)?;
                reader.finish_capture("$value");
            }
            Inner::OccurrenceCount { r, s, t, ref f } => {
                let count = self.read_count(reader, f, &mut |reader| {
                    reader.parse_unbounded(self, r)?;
                    Ok(())
                })?;
                if let Some(node_index) = s {
                    reader.parse_unbounded(self, node_index)?;
                }
                reader.start_capture("$value");
                reader.start_repeat();
                for _ in 0..count {
                    reader.parse_unbounded(self, t)?;
                }
                reader.finish_repeat();
                reader.finish_capture("$value");
            }
        }
        Ok(())
    }

    /// Parses a bounded number of bytes from the given `Reader` against the
    /// sub-expression represented by the given `Node`.
    ///
    /// If the given number of bytes is or would be exceeded, an `Error` is
    /// returned.
    ///
    /// The given `Node` has to be part of the `CalcRegex`, this method is
    /// called on.
    ///
    /// This method is supposed to be called from a parsing routine of
    /// `Reader`. It deconstructs the given `Node`, either matching its
    /// containing regex, or using the `Reader` again to parse its
    /// sub-expressions.
    pub(crate) fn parse_bounded<I: Input>(
        &self,
        reader: &mut Reader<I>,
        node: &Node,
        bound: usize
    ) -> ParserResult<()> {
        match node.inner {
            Inner::Regex(ref regex) => {
                reader.match_regex_bounded(regex, bound)?;
            }
            Inner::CalcRegex(node_index) => {
                reader.parse_bounded(self, node_index, bound)?;
            }
            Inner::Concat(r, s) => {
                let length_r = reader.parse_bounded(self, r, bound)?;
                let bound_s = bound - length_r;
                reader.parse_bounded(self, s, bound_s)?;
            }
            Inner::Repeat(node_index, n) => {
                let mut bound = bound;
                reader.start_repeat();
                for _ in 0..n {
                    bound -= reader.parse_bounded(self, node_index, bound)?;
                }
                reader.finish_repeat();
            }
            Inner::KleeneStar(_) => {
                panic!("KleeneStar can only be parsed with parse_exact().")
            }
            Inner::LengthCount { r, s, t, ref f } => {
                let mut bound = bound;
                let count = self.read_count(reader, f, &mut |reader| {
                    bound -= reader.parse_bounded(self, r, bound)?;
                    Ok(())
                })?;
                if let Some(node_index) = s {
                    bound -= reader.parse_bounded(self, node_index, bound)?;
                }
                if bound < count {
                    return Err(ParserError::ConflictingBounds {
                        old: bound,
                        new: count,
                    });
                }
                reader.start_capture("$value");
                reader.parse_exact(self, t, count)?;
                reader.finish_capture("$value");
            }
            Inner::OccurrenceCount { r, s, t, ref f } => {
                let mut bound = bound;
                let count = self.read_count(reader, f, &mut |reader| {
                    bound -= reader.parse_bounded(self, r, bound)?;
                    Ok(())
                })?;
                if let Some(node_index) = s {
                    bound -= reader.parse_bounded(self, node_index, bound)?;
                }
                reader.start_capture("$value");
                reader.start_repeat();
                for _ in 0..count {
                    bound -= reader.parse_bounded(self, t, bound)?;
                }
                reader.finish_repeat();
                reader.finish_capture("$value");
            }
        }
        Ok(())
    }

    /// Parses exactly a given number of bytes from the given `Reader` against
    /// the sub-expression represented by the given `Node`.
    ///
    /// If the given number of bytes cannot be parsed exactly, an `Error` is
    /// returned.
    ///
    /// The given `Node` has to be part of the `CalcRegex`, this method is
    /// called on.
    ///
    /// This method is supposed to be called from a parsing routine of
    /// `Reader`. It deconstructs the given `Node`, either matching its
    /// containing regex, or using the `Reader` again to parse its
    /// sub-expressions.
    pub(crate) fn parse_exact<I: Input>(
        &self,
        reader: &mut Reader<I>,
        node: &Node,
        length: usize
    ) -> ParserResult<()> {
        match node.inner {
            Inner::Regex(ref regex) => {
                reader.match_regex_exact(regex, length)?;
            }
            Inner::CalcRegex(node_index) => {
                reader.parse_exact(self, node_index, length)?;
            }
            Inner::Concat(r, s) => {
                let length_r = reader.parse_bounded(self, r, length)?;
                let length_s = length - length_r;
                reader.parse_exact(self, s, length_s)?;
            }
            Inner::Repeat(node_index, n) => {
                let mut length = length;
                reader.start_repeat();
                for _ in 0..n-1 {
                    length -= reader.parse_bounded(self, node_index, length)?;
                }
                reader.parse_exact(self, node_index, length)?;
                reader.finish_repeat();
            }
            Inner::KleeneStar(node_index) => {
                let mut length = length;
                reader.start_repeat();
                while length > 0 {
                    length -= reader.parse_bounded(self, node_index, length)?;
                }
                reader.finish_repeat();
            }
            Inner::LengthCount { r, s, t, ref f } => {
                let mut length = length;
                let count = self.read_count(reader, f, &mut |reader| {
                    length -= reader.parse_bounded(self, r, length)?;
                    Ok(())
                })?;
                if let Some(node_index) = s {
                    reader.parse_exact(self, node_index, length - count)?;
                } else if length != count {
                    return Err(ParserError::ConflictingBounds {
                        old: length,
                        new: count,
                    });
                }
                reader.start_capture("$value");
                reader.parse_exact(self, t, count)?;
                reader.finish_capture("$value");
            }
            Inner::OccurrenceCount { r, s, t, ref f } => {
                let mut length = length;
                let count = self.read_count(reader, f, &mut |reader| {
                    length -= reader.parse_bounded(self, r, length)?;
                    Ok(())
                })?;
                if let Some(node_index) = s {
                    length -= reader.parse_bounded(self, node_index, length)?;
                }
                reader.start_capture("$value");
                reader.start_repeat();
                for _ in 0..count-1 {
                    length -= reader.parse_bounded(self, t, length)?;
                }
                reader.parse_exact(self, t, length)?;
                reader.finish_repeat();
                reader.finish_capture("$value");
            }
        }
        Ok(())
    }

    /// Reads the count value by calling `parse` and than calling `f` on the
    /// parsed byte slice.
    fn read_count<I: Input>(
        &self,
        reader: &mut Reader<I>,
        f: &fn(&[u8]) -> Option<usize>,
        parse: &mut FnMut(&mut Reader<I>) -> ParserResult<()>,
    ) -> ParserResult<usize> {
        reader.start_capture("$count");
        let start_pos = reader.pos();
        parse(reader)?;
        reader.finish_capture("$count");
        let end_pos = reader.pos();
        let raw_count = reader.get_range((start_pos, end_pos));
        f(raw_count).ok_or(ParserError::CannotReadCount {
            raw_count: raw_count.to_vec(),
        })
    }
}
