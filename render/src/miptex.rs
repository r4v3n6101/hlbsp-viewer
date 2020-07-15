use nom::{
    bytes::streaming::{take, take_until},
    combinator::{map, map_res},
    multi::count,
    number::streaming::le_u32,
    sequence::tuple,
};

const NAME_LEN: usize = 16;
const MIP_NUM: usize = 4;
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
            (None, None) // TODO : rewrite to separate fn
        } else {
            let mut color_indices: [&[u8]; MIP_NUM] = [&[]; MIP_NUM];
            for i in 0..MIP_NUM {
                let mip_offset = offsets[i];
                let mip_i = {
                    if mip_offset > file.len() {
                        return Err(nom::Err::Incomplete(nom::Needed::Size(mip_offset)));
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
                    return Err(nom::Err::Incomplete(nom::Needed::Size(color_table_offset)));
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

    pub const fn name(&self) -> &str {
        self.name
    }

    pub fn width(&self, mip_level: usize) -> u32 {
        self.width / (1 << mip_level)
    }

    pub fn height(&self, mip_level: usize) -> u32 {
        self.height / (1 << mip_level)
    }

    // TODO : naming
    // TODO : working on alpha
    // TODO : RawImage2d may outlive
    pub fn make_texture(&self, mip_level: usize) -> Option<glium::texture::RawImage2d<'a, u8>> {
        if mip_level > MIP_NUM {
            return None;
        }
        let width = self.width(mip_level);
        let height = self.height(mip_level);
        let color_table = self.color_table?;
        let color_indices = self.color_indices?;
        let texture = color_indices[mip_level]
            .into_iter()
            .map(|&i| i as usize)
            .flat_map(|i| color_table[3 * i..3 * (i + 1)].iter().copied())
            .collect();
        Some(glium::texture::RawImage2d::from_raw_rgb(
            texture,
            (width, height),
        ))
    }
}
