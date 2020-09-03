use nom::{bytes::complete::take_until, combinator::map_res};

type Input<'a> = &'a [u8];
type OnlyResult<'a, O> = Result<O, nom::Err<ParseError<'a>>>;
type ParseError<'a> = nom::error::VerboseError<Input<'a>>;

pub fn parse_entities_str(i: &[u8]) -> OnlyResult<&str> {
    let (_, s) = map_res(take_until("\0"), std::str::from_utf8)(i)?;
    Ok(s)
}
