// TODO : Reformat to human-read structures

use crate::miptex::MipTexture;
use nom::{
    bytes::complete::{take, take_until},
    combinator::{map, map_res},
    multi::{count, many0},
    number::complete::{le_f32, le_i32, le_u16, le_u32, le_u8},
    sequence::tuple,
};

pub type Vec3 = (f32, f32, f32);
type Input<'a> = &'a [u8];
type ParseResult<'a, O> = nom::IResult<Input<'a>, O, ParseError<'a>>;
type OnlyResult<'a, O> = Result<O, nom::Err<ParseError<'a>>>;
type ParseError<'a> = nom::error::VerboseError<Input<'a>>;

pub struct TexInfo {
    pub vs: Vec3,
    pub ss: f32,
    pub vt: Vec3,
    pub st: f32,
    pub texture_id: usize,
}

pub struct Face {
    pub plane_id: usize,
    pub side: bool,
    pub surfedge_id: usize,
    pub surfedge_num: usize,
    pub texinfo_id: usize,
    pub lightmap: usize,
}

pub struct Model {
    pub origin: Vec3,
    pub face_id: usize,
    pub face_num: usize,
}

pub fn parse_entities_str(i: &[u8]) -> OnlyResult<&str> {
    let (_, s) = map_res(take_until("\0"), std::str::from_utf8)(i)?;
    Ok(s)
}

fn parse_vec3(i: &[u8]) -> ParseResult<Vec3> {
    tuple((le_f32, le_f32, le_f32))(i)
}

pub fn parse_vertices(i: &[u8]) -> OnlyResult<Vec<Vec3>> {
    let (_, vertices) = many0(parse_vec3)(i)?;
    Ok(vertices)
}

fn parse_edge(i: &[u8]) -> ParseResult<(u16, u16)> {
    tuple((le_u16, le_u16))(i)
}

pub fn parse_edges(i: &[u8]) -> OnlyResult<Vec<(u16, u16)>> {
    let (_, edges) = many0(parse_edge)(i)?;
    Ok(edges)
}

pub fn parse_surfedges(i: &[u8]) -> OnlyResult<Vec<i32>> {
    let (_, surfedges) = many0(le_i32)(i)?;
    Ok(surfedges)
}

fn parse_normal_from_plane(i: &[u8]) -> ParseResult<Vec3> {
    let (i, (normal, _, _)) = tuple((parse_vec3, le_f32, le_u32))(i)?;
    Ok((i, normal))
}

pub fn parse_normals_from_planes(i: &[u8]) -> OnlyResult<Vec<Vec3>> {
    let (_, normals) = many0(parse_normal_from_plane)(i)?;
    Ok(normals)
}

fn parse_texinfo(i: &[u8]) -> ParseResult<TexInfo> {
    let (i, (vs, ss, vt, st, texture_id, _)) = tuple((
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
            texture_id,
        },
    ))
}

pub fn parse_texinfos(i: &[u8]) -> OnlyResult<Vec<TexInfo>> {
    let (_, texinfos) = many0(parse_texinfo)(i)?;
    Ok(texinfos)
}

fn parse_face(i: &[u8]) -> ParseResult<Face> {
    let (i, (plane_id, side, surfedge_id, surfedge_num, texinfo_id, _, _, _, _, lightmap)) =
        tuple((
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
            surfedge_id,
            surfedge_num,
            texinfo_id,
            lightmap,
        },
    ))
}

pub fn parse_faces(i: &[u8]) -> OnlyResult<Vec<Face>> {
    let (_, faces) = many0(parse_face)(i)?;
    Ok(faces)
}

fn parse_model(i: &[u8]) -> ParseResult<Model> {
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

pub fn parse_models(i: &[u8]) -> OnlyResult<Vec<Model>> {
    let (_, models) = many0(parse_model)(i)?;
    Ok(models)
}

pub fn parse_textures(lump: &[u8]) -> OnlyResult<Vec<MipTexture>> {
    let (i, offsets_num) = map(le_u32, |x| x as usize)(lump)?;
    let (_, offsets) = count(le_u32, offsets_num)(i)?;
    offsets
        .into_iter()
        .map(|offset| {
            let offset = offset as usize;
            let (mip_i, _) = take(offset)(lump)?;
            MipTexture::parse(mip_i)
        })
        .collect()
}
