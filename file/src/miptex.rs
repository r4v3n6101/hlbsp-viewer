use nom::{
    bytes::streaming::{take, take_until},
    combinator::{map, map_res},
    multi::count,
    number::streaming::le_u32,
    sequence::tuple,
};
use std::iter::once;

const MIP_NUM: usize = 4;
const NAME_LEN: usize = 16;
const COLOR_TABLE_SIZE: usize = 256 * 3;

type Input<'a> = &'a [u8];
type ParseResult<'a, O> = nom::IResult<Input<'a>, O, ParseError<'a>>;
type ParseError<'a> = nom::error::VerboseError<Input<'a>>;

fn take_cstr(i: &[u8], size: usize) -> ParseResult<&str> {
    let (i, cstr) = take(size)(i)?;
    let (_, cstr) = map_res(take_until("\0"), std::str::from_utf8)(cstr)?;
    Ok((i, cstr))
}

pub struct MipTexture<'a> {
    name: &'a str,
    width: u32,
    height: u32,
    color_indices: Option<[&'a [u8]; MIP_NUM]>,
    color_table: Option<&'a [u8]>,
}

impl<'a> MipTexture<'a> {
    pub fn parse(file: &'a [u8]) -> Result<MipTexture<'a>, nom::Err<ParseError<'a>>> {
        let (_, (name, width, height, offsets)) = tuple((
            { |i| take_cstr(i, NAME_LEN) },
            le_u32,
            le_u32,
            count(map(le_u32, |x| x as usize), MIP_NUM),
        ))(file)?;

        let (color_indices, color_table) = if offsets.iter().any(|&x| x == 0) {
            (None, None)
        } else {
            let mut color_indices: [&[u8]; MIP_NUM] = [&[]; MIP_NUM];
            for i in 0..MIP_NUM {
                let mip_offset = offsets[i];
                let mip_i = {
                    if mip_offset > file.len() {
                        return Err(nom::Err::Incomplete(nom::Needed::new(mip_offset)));
                    }
                    &file[mip_offset..]
                };
                let (_, mip_indices) =
                    take((width as usize * height as usize) / (1 << (2 * i)))(mip_i)?;
                color_indices[i] = mip_indices;
            }

            let color_table_offset = offsets[MIP_NUM - 1]
                + (width as usize * height as usize) / (1 << (2 * (MIP_NUM - 1)))
                + 2; // 2 is gap
            let color_table_i = {
                if color_table_offset > file.len() {
                    return Err(nom::Err::Incomplete(nom::Needed::new(color_table_offset)));
                    // TODO : not verbose error
                }
                &file[color_table_offset..]
            };

            let (_, color_table) = take(COLOR_TABLE_SIZE)(color_table_i)?;
            (Some(color_indices), Some(color_table))
        };

        Ok(MipTexture {
            name,
            width,
            height,
            color_indices,
            color_table,
        })
    }

    pub const fn layers() -> usize {
        MIP_NUM
    }

    pub const fn name(&self) -> &str {
        self.name
    }

    pub const fn main_width(&self) -> u32 {
        self.width
    }

    pub const fn main_height(&self) -> u32 {
        self.height
    }

    pub fn is_empty(&self) -> bool {
        self.color_table.is_none() || self.color_indices.is_none()
    }

    pub fn pixels(&self, mip_level: usize) -> Option<Vec<u8>> {
        let color_table = self.color_table?;
        Some(
            self.color_indices?[mip_level]
                .iter()
                .map(|&i| i as usize)
                .flat_map(|i| {
                    let r = color_table[3 * i];
                    let g = color_table[3 * i + 1];
                    let b = color_table[3 * i + 2];
                    let a = if r < 30 && g < 30 && b > 125 { 0 } else { 255 };
                    once(r).chain(once(g)).chain(once(b)).chain(once(a))
                })
                .collect(),
        )
    }
}
