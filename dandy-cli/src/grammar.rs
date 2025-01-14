use crate::{DandyArgs, ParseGrammarArgs};
use dandy::grammar::parse::GrammarParseError;
use dandy::grammar::Grammar;
use dandy::parser;
use std::{fs, io};
use thiserror::Error;

pub fn parse_grammar<'a>(
    _main_args: &DandyArgs,
    args: &ParseGrammarArgs,
    #[allow(unused_variables, unused_mut)] mut output: impl FnMut(&str),
) -> Result<(), String> {
    let file = fs::read_to_string(&args.grammar).map_err(|e| Error::InputFile(e).to_string())?;

    let grammar: Grammar = do_parse_grammar(&file).map_err(|e| e.to_string())?;

    println!("{:?}", grammar);
    Ok(())
}

pub fn do_parse_grammar<'a>(
    file: &'a str
) -> Result<Grammar<'a>, Error<'a>> {

    let grammar: Grammar = parser::grammar(file)
        .map_err(Error::GrammarParse)?
        .try_into()
        .map_err(Error::Grammar)?;

    Ok(grammar)
}

#[derive(Error, Debug)]
pub enum Error<'a> {
    #[error("Error parsing grammar: {0}")]
    GrammarParse(nom::error::Error<&'a str>),
    #[error("Error validating grammar: {0}")]
    Grammar(GrammarParseError<'a>),
    #[error("Error reading input file: {0}")]
    InputFile(#[from] io::Error),
}
