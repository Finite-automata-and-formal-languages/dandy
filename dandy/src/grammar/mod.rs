//! # Context-free grammars
pub use crate::parser::grammar as parse;

pub mod parse;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grammar<'a> {
    pub(crate) nonterminals: Vec<&'a str>,
    pub(crate) terminals: Vec<&'a str>,
    pub(crate) start: &'a str,
    pub(crate) productions: Vec<Production<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Production<'a> {
    pub(crate) name: &'a str,
    pub(crate) alternatives: Vec<Vec<&'a str>>,
}
