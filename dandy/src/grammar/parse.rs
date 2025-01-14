use crate::grammar::{Grammar, Production};
use crate::parser::ParsedGrammar;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GrammarParseError<'a> {
    #[error("'{0}' appears twice in the nonterminal symbols")]
    DuplicateNonterminal(&'a str),
    #[error("'{0}' appears twice in the terminal symbols")]
    DuplicateTerminal(&'a str),
    #[error("'{0}' is declared as both a nonterminal and a terminal symbol")]
    TerminalNonterminal(&'a str),
    #[error("The start symbol is not a nonterminal")]
    StartNotNonterminal,
    #[error("'{0}' has productions but is not a nonterminal symbol")]
    ProductionsNotNonterminal(&'a str),
    #[error("'{0}' appears in productions but is not a symbol")]
    ProductionsNotSymbol(&'a str),
    #[error("'{0}' appears twice in the list of productions; use | to separate alternatives")]
    DuplicateProduction(&'a str),
}

impl<'a> TryFrom<ParsedGrammar<'a>> for Grammar<'a> {
    type Error = GrammarParseError<'a>;

    fn try_from(value: ParsedGrammar<'a>) -> Result<Self, Self::Error> {
        use GrammarParseError::*;
        let ParsedGrammar { nonterminals, terminals, start, productions } = value;

        {
            let mut nonterminals_set = HashSet::new();
            nonterminals
                .iter()
                .try_for_each(|c| nonterminals_set.insert(c).then_some(()).ok_or(c))
                .map_err(|d| DuplicateNonterminal(d))?;

            let mut terminals_set = HashSet::new();
            terminals
                .iter()
                .try_for_each(|c| terminals_set.insert(c).then_some(()).ok_or(c))
                .map_err(|d| DuplicateTerminal(d))?;

            if let Some(x) = nonterminals_set.intersection(&terminals_set).next() {
                return Err(TerminalNonterminal(x));
            }

            if !nonterminals_set.contains(&start) {
                return Err(StartNotNonterminal);
            }

            let mut productions_set = HashSet::new();
            productions
                .iter()
                .try_for_each(|p| {
                    if !productions_set.insert(p.name) {
                        return Err(DuplicateProduction(p.name));
                    }
                    if !nonterminals_set.contains(&p.name) {
                        return Err(ProductionsNotNonterminal(p.name));
                    }
                    for alt in p.alternatives.iter() {
                        for s in alt {
                            if !(nonterminals_set.contains(&s) || terminals_set.contains(&s)) {
                                return Err(ProductionsNotSymbol(s));
                            }
                        }
                    }
                    Ok(())
                })?;
        }

        let grammar = Grammar {
            nonterminals,
            terminals,
            start,
            productions: productions.iter().map(|p| Production { name: p.name, alternatives: p.alternatives.to_vec() }).collect(),
        };
        Ok(grammar)
    }
}
