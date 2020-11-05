use nom::{
    bytes::complete::take_until,
    character::complete::{char as character, multispace0},
    multi::many0,
    sequence::{delimited, separated_pair},
};
use std::collections::HashMap;

type Input<'a> = &'a str;
type ParseError<'a> = nom::error::VerboseError<Input<'a>>;
type ParseResult<'a, O> = nom::IResult<Input<'a>, O, ParseError<'a>>;
type OnlyResult<'a, O> = Result<O, nom::Err<ParseError<'a>>>;

#[derive(Debug)]
pub struct Entity<'a> {
    properties: HashMap<&'a str, &'a str>,
}

impl<'a> Entity<'a> {
    fn entry(i: &str) -> ParseResult<(&str, &str)> {
        separated_pair(
            delimited(character('"'), take_until("\""), character('"')),
            multispace0,
            delimited(character('"'), take_until("\""), character('"')),
        )(i)
    }

    fn entries(i: &str) -> ParseResult<Vec<(&str, &str)>> {
        many0(delimited(multispace0, Self::entry, multispace0))(i)
    }

    fn parse(i: &'a str) -> ParseResult<'a, Self> {
        let (i, properties) = delimited(
            character('{'),
            delimited(multispace0, Self::entries, multispace0),
            character('}'),
        )(i)?;
        let properties = properties.into_iter().collect();
        Ok((i, Self { properties }))
    }

    pub fn properties(&self) -> &HashMap<&str, &str> {
        &self.properties
    }
}

#[derive(Debug)]
pub struct Entities<'a>(Vec<Entity<'a>>);

impl<'a> Entities<'a> {
    pub fn parse(i: &'a str) -> OnlyResult<Self> {
        let (_, ents) = many0(delimited(multispace0, Entity::parse, multispace0))(i)?;
        Ok(Self(ents))
    }

    pub fn entities(&self) -> &[Entity] {
        &self.0
    }
}
