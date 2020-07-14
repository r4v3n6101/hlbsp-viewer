use nom::{
    bytes::complete::take_until,
    combinator::{map, map_res},
    multi::many0,
    number::complete::{le_f32, le_u16, le_u32, le_u8},
    sequence::tuple,
};

pub type Vec3 = (f32, f32, f32);
type ParseResult<'a, O> = Result<O, nom::Err<(&'a [u8], nom::error::ErrorKind)>>;

pub fn parse_entities_str(i: &[u8]) -> ParseResult<&str> {
    let (_, s) = map_res(take_until("\0"), std::str::from_utf8)(i)?;
    Ok(s)
}

pub fn parse_vec3(i: &[u8]) -> nom::IResult<&[u8], Vec3> {
    tuple((le_f32, le_f32, le_f32))(i)
}

pub fn parse_vertices(i: &[u8]) -> ParseResult<Vec<Vec3>> {
    let (_, vertices) = many0(parse_vec3)(i)?;
    Ok(vertices)
}

pub fn parse_normal_from_plane(i: &[u8]) -> nom::IResult<&[u8], Vec3> {
    let (i, (normal, _, _)) = tuple((parse_vec3, le_f32, le_u32))(i)?;
    Ok((i, normal))
}

pub fn parse_normals_from_planes(i: &[u8]) -> ParseResult<Vec<Vec3>> {
    let (_, normals) = many0(parse_normal_from_plane)(i)?;
    Ok(normals)
}

pub fn parse_texinfo(i: &[u8]) -> nom::IResult<&[u8], ()> {
    let (i, (vs, ss, vt, st, miptex, _)) = tuple((
        parse_vec3,
        le_f32,
        parse_vec3,
        le_f32,
        map(le_u32, |x| x as usize),
        le_u32,
    ))(i)?;
    todo!()
}

pub fn parse_texinfos(i: &[u8]) -> ParseResult<Vec<()>> {
    let (_, texinfos) = many0(parse_texinfo)(i)?;
    Ok(texinfos)
}

pub fn parse_face(i: &[u8]) -> nom::IResult<&[u8], ()> {
    let (i, (plane_id, side, ledge_id, ledge_num, texinfo_id, _, _, _, _, lightmap)) = tuple((
        map(le_u16, |x| x as usize),
        map(le_u16, |x| x != 0),
        map(le_u32, |x| x as usize),
        map(le_u16, |x| x as usize),
        map(le_u16, |x| x as usize),
        le_u8,
        le_u8,
        le_u8,
        le_u8,
        map(le_u32, |x| x as usize),
    ))(i)?;
    todo!()
}

pub fn parse_faces(i: &[u8]) -> ParseResult<Vec<()>> {
    let (_, faces) = many0(parse_face)(i)?;
    Ok(faces)
}
