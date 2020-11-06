pub mod lumps; // TODO

use nom::{
    bytes::complete::take,
    combinator::{map, verify},
    multi::count,
    number::complete::le_u32,
    sequence::tuple,
};

const LUMPS_NUM: usize = 15;
const HLBSP_VERSION: u32 = 30;

pub enum LumpType {
    Entities,
    Planes,
    Textures,
    Vertices,
    Visibility,
    Nodes,
    TexInfo,
    Faces,
    Lighting,
    Clipnodes,
    Leaves,
    Marksurfaces,
    Edges,
    Surfegdes,
    Models,
}

type Input<'a> = &'a [u8];
type ParseError<'a> = nom::error::VerboseError<Input<'a>>;
type ParseResult<'a, O> = nom::IResult<Input<'a>, O, ParseError<'a>>;
type OnlyResult<'a, O> = Result<O, nom::Err<ParseError<'a>>>;

pub struct Lump<'a> {
    data: &'a [u8],
}

impl<'a> Lump<'a> {
    fn parse(i: &'a [u8], file: &'a [u8]) -> ParseResult<'a, Self> {
        let (i, (offset, size)) =
            tuple((map(le_u32, |x| x as usize), map(le_u32, |x| x as usize)))(i)?;
        let (lump_i, _) = take(offset)(file)?;
        let (_, data) = take(size)(lump_i)?;

        Ok((i, Self { data }))
    }
}

pub struct RawMap<'a> {
    lumps: Vec<Lump<'a>>,
}

impl<'a> RawMap<'a> {
    pub fn parse(file: &'a [u8]) -> OnlyResult<Self> {
        let (_, (_, lumps)) = tuple((
            verify(le_u32, |&x| x == HLBSP_VERSION),
            count(|i| Lump::parse(i, file), LUMPS_NUM),
        ))(file)?;
        Ok(RawMap { lumps })
    }

    pub fn lump_data(&self, lump_type: LumpType) -> &[u8] {
        self.lumps[lump_type as usize].data
    }
}
