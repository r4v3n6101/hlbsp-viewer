use nom::{
    bytes::complete::{tag, take, take_until},
    combinator::{map, map_res},
    multi::count,
    number::complete::{le_u16, le_u32, le_u8},
    sequence::tuple,
};
use std::{collections::HashMap, iter::Iterator};

const WAD3_MAGIC: &[u8] = b"WAD3";
const NAME_LEN: usize = 16;

type Input<'a> = &'a [u8];
type ParseError<'a> = nom::error::VerboseError<Input<'a>>;
type ParseResult<'a, O> = nom::IResult<Input<'a>, O, ParseError<'a>>;
type OnlyResult<'a, O> = Result<O, nom::Err<ParseError<'a>>>;

fn take_cstr(i: &[u8], size: usize) -> ParseResult<&str> {
    let (i, cstr) = take(size)(i)?;
    let (_, cstr) = map_res(take_until("\0"), std::str::from_utf8)(cstr)?;
    Ok((i, cstr))
}

pub struct Entry<'a> {
    etype: u8,
    data: &'a [u8],
}

impl<'a> Entry<'a> {
    fn parse(i: &'a [u8], file: &'a [u8]) -> ParseResult<'a, (&'a str, Self)> {
        // There's no compression, because I don't find any wad using compression (seems it's LZSS)
        let (i, (offset, disk_size, _, etype, _, _)) = tuple((
            map(le_u32, |x| x as usize),
            map(le_u32, |x| x as usize),
            le_u32,
            le_u8,
            le_u8,
            le_u16,
        ))(i)?;
        let (i, name) = take_cstr(i, NAME_LEN)?;

        let (data_i, _) = take(offset)(file)?;
        let (_, data) = take(disk_size)(data_i)?;

        Ok((i, (name, Self { etype, data })))
    }

    pub const fn etype(&self) -> u8 {
        self.etype
    }

    pub const fn data(&self) -> &[u8] {
        self.data
    }
}

pub struct Archive<'a> {
    entries: HashMap<&'a str, Entry<'a>>,
}

impl<'a> Archive<'a> {
    pub fn parse(file: &'a [u8]) -> OnlyResult<Self> {
        let (_, (_, dir_num, dir_offset)) = tuple((
            tag(WAD3_MAGIC),
            map(le_u32, |x| x as usize),
            map(le_u32, |x| x as usize),
        ))(file)?;

        let (dir_i, _) = take(dir_offset)(file)?;
        let (_, entries) = map(count(|i| Entry::parse(i, file), dir_num), |x| {
            x.into_iter().collect()
        })(dir_i)?;
        Ok(Self { entries })
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &Entry)> {
        self.entries.iter().map(|(&name, entry)| (name, entry))
    }

    pub fn get_by_name<S: AsRef<str>>(&self, name: S) -> Option<&Entry> {
        self.entries.get(name.as_ref())
    }
}
