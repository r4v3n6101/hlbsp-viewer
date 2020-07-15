use nom::{
    bytes::complete::take_until,
    combinator::{map, map_res},
    multi::many0,
    number::complete::{le_f32, le_i16, le_u16, le_u32, le_u8},
    sequence::tuple,
};

pub type Vec3 = (f32, f32, f32);
type ParseResult<'a, O> = Result<O, nom::Err<(&'a [u8], nom::error::ErrorKind)>>;

pub struct TexInfo {
    pub vs: Vec3,
    pub ss: f32,
    pub vt: Vec3,
    pub st: f32,
    pub miptex: usize,
}

pub struct Face {
    pub plane_id: usize,
    pub side: bool,
    pub ledge_id: usize,
    pub ledge_num: usize,
    pub texinfo_id: usize,
    pub lightmap: usize,
}

pub struct Model {
    pub origin: Vec3,
    pub face_id: usize,
    pub face_num: usize,
}

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

pub fn parse_edge(i: &[u8]) -> nom::IResult<&[u8], (usize, usize)> {
    tuple((map(le_u32, |x| x as usize), map(le_u32, |x| x as usize)))(i)
}

pub fn parse_edges(i: &[u8]) -> ParseResult<Vec<(usize, usize)>> {
    let (_, edges) = many0(parse_edge)(i)?;
    Ok(edges)
}

pub fn parse_ledges(i: &[u8]) -> ParseResult<Vec<i16>> {
    let (_, ledges) = many0(le_i16)(i)?;
    Ok(ledges)
}

pub fn parse_normal_from_plane(i: &[u8]) -> nom::IResult<&[u8], Vec3> {
    let (i, (normal, _, _)) = tuple((parse_vec3, le_f32, le_u32))(i)?;
    Ok((i, normal))
}

pub fn parse_normals_from_planes(i: &[u8]) -> ParseResult<Vec<Vec3>> {
    let (_, normals) = many0(parse_normal_from_plane)(i)?;
    Ok(normals)
}

pub fn parse_texinfo(i: &[u8]) -> nom::IResult<&[u8], TexInfo> {
    let (i, (vs, ss, vt, st, miptex, _)) = tuple((
        parse_vec3,
        le_f32,
        parse_vec3,
        le_f32,
        map(le_u32, |x| x as usize),
        le_u32,
    ))(i)?;
    Ok((
        i,
        TexInfo {
            vs,
            ss,
            vt,
            st,
            miptex,
        },
    ))
}

pub fn parse_texinfos(i: &[u8]) -> ParseResult<Vec<TexInfo>> {
    let (_, texinfos) = many0(parse_texinfo)(i)?;
    Ok(texinfos)
}

pub fn parse_face(i: &[u8]) -> nom::IResult<&[u8], Face> {
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
    Ok((
        i,
        Face {
            plane_id,
            side,
            ledge_id,
            ledge_num,
            texinfo_id,
            lightmap,
        },
    ))
}

pub fn parse_faces(i: &[u8]) -> ParseResult<Vec<Face>> {
    let (_, faces) = many0(parse_face)(i)?;
    Ok(faces)
}

pub fn parse_model(i: &[u8]) -> nom::IResult<&[u8], Model> {
    let (i, (_, origin, _, _, _, _, _, face_id, face_num)) = tuple((
        tuple((parse_vec3, parse_vec3)),
        parse_vec3,
        le_u32,
        le_u32,
        le_u32,
        le_u32,
        le_u32,
        map(le_u32, |x| x as usize),
        map(le_u32, |x| x as usize),
    ))(i)?;
    Ok((
        i,
        Model {
            origin,
            face_id,
            face_num,
        },
    ))
}

pub fn parse_models(i: &[u8]) -> ParseResult<Vec<Model>> {
    let (_, models) = many0(parse_model)(i)?;
    Ok(models)
}
