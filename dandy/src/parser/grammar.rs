use crate::parser::{ParsedProduction, ParsedGrammar};
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_till1};
use nom::character::complete::{line_ending, not_line_ending, space0, space1};
use nom::combinator::{eof, map, opt, recognize, value, verify};
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

pub(crate) fn full_grammar(input: &str) -> IResult<&str, ParsedGrammar> {
    map(
        delimited(
            many0(space_comment_line),
            tuple((
                terminated(symbols("Nonterminals:"), many1(space_comment_line)),
                terminated(symbols("Terminals:"), many1(space_comment_line)),
                terminated(symbol("Start:"), many1(space_comment_line)),
                separated_list1(many1(space_comment_line), production)
            )),
            many0(space_comment_line),
        ),
        |(nonterminals, terminals, start, productions)| ParsedGrammar { nonterminals, terminals, start, productions },
    )(input)
}

fn symbols(prompt: &str) -> impl Fn(&str) -> IResult<&str, Vec<&str>> + use<'_> {
    move |input: &str| {
        preceded(
            delimited(
                space0,
                tag_no_case(prompt),
                space1
            ),
            separated_list0(space1, symbol_name)
        )(input)
    }
}

fn symbol(prompt: &str) -> impl Fn(&str) -> IResult<&str, &str> + use<'_> {
    move |input: &str| {
        preceded(
            delimited(
                space0,
                tag_no_case(prompt),
                space1
            ),
            symbol_name
        )(input)
    }
}

fn production(input: &str) -> IResult<&str, ParsedProduction> {
    map(
        preceded(
            space0,
            pair(
                terminated(symbol_name, delimited(space1, arrow, space1)),
                separated_list1(delimited(space0, pipe, space0), separated_list0(space1, symbol_name)),
            ),
        ),
        |(name, alternatives)| ParsedProduction {
            name,
            alternatives,
        },
    )(input)
}

fn symbol_name(input: &str) -> IResult<&str, &str> {
    verify(
        take_till1(|c: char| c.is_whitespace() || "#".contains(c)),
        |elem| !["|", "→", "->"].contains(&elem),
    )(input)
}

fn arrow(input: &str) -> IResult<&str, ()> {
    map(alt((tag("->"), tag("→"))), |_| ())(input)
}

fn pipe(input: &str) -> IResult<&str, ()> {
    map(tag("|"), |_| ())(input)
}

fn space_comment_line(input: &str) -> IResult<&str, ()> {
    // We need to allow a space-only or comment-only line to end with either
    // a line ending or eof, but we need to consume *something* otherwise
    // many0(space_comment_line) will be in an endless loop at eof
    value(
        (),
        verify(
            recognize(terminated(space_comment, alt((line_ending, eof)))),
            |consumed: &str| !consumed.is_empty(),
        ),
    )(input)
}

fn space_comment(input: &str) -> IResult<&str, ()> {
    value((), pair(space0, opt(comment)))(input)
}

fn comment(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("#"), not_line_ending))(input)
}
