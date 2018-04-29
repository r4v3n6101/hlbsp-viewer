extern crate hlbsp2obj;
extern crate image;

use hlbsp2obj::{bsp::*, read_mul_structs, read_struct, texture::*, wad::*};
use std::{env::args, fs::*, io::*};

fn main() {
    let bsp_path = args().nth(1).unwrap();
    let wad_path = args().nth(2).unwrap();
    let obj_path = bsp_path.replace("bsp", "obj");
    let atlas_path = bsp_path.replace("bsp", "png");

    let bsp_file = File::open(bsp_path).unwrap();
    let size = bsp_file.metadata().unwrap().len() + 1;
    let mut bsp: Vec<u8> = Vec::with_capacity(size as usize);

    let mut buf_reader = BufReader::new(bsp_file);
    buf_reader.read_to_end(&mut bsp).unwrap();

    let obj_file = File::create(obj_path).unwrap();
    let mut obj_writer = BufWriter::new(obj_file);

    let textures: HashMap<String, Texture> = read_textures(wad_path.as_ref(), 0);

    let mut atlas_writer = BufWriter::new(File::create(atlas_path).unwrap());
    write_obj(&mut obj_writer, &mut atlas_writer, &bsp, &textures);
}

fn write_obj<W: Write>(
    obj_writer: &mut W,
    atlas_writer: &mut W,
    bsp: &[u8],
    wad_textures: &HashMap<String, Texture>,
) {
    let header: Header = read_struct(&bsp);
    let vertices: Vertices = header.lumps[LUMP_VERTICES].read_array(&bsp);
    let faces: Vec<Face> = header.lumps[LUMP_FACES].read_array(&bsp);
    let surfedges: Vec<i32> = header.lumps[LUMP_SURFEDGES].read_array(&bsp);
    let edges: Vec<Edge> = header.lumps[LUMP_EDGES].read_array(&bsp);
    let texinfos: Vec<TexInfo> = header.lumps[LUMP_TEXINFO].read_array(&bsp);

    ///////////////////////////////////////////////////////////////////////////////////////////////
    let offset = header.lumps[LUMP_TEXTURES].offset as usize;
    let mip_off: u32 = read_struct(&bsp[offset..]);
    let begin = offset + 4;
    let offsets: Vec<i32> = read_mul_structs(&bsp[begin..begin + mip_off as usize * 4]);

    let mip_texs: Vec<MipTex> =
        offsets.iter().map(|i| read_struct(&bsp[offset + *i as usize..])).collect();

    let required_texs: HashMap<String, &Texture> = mip_texs.iter().map(|mip_tex| {
        let name = read_name(mip_tex.name);
        let tex = wad_textures.get(&name).expect(&name);
        (name, tex)
    }).collect();

    let atlas = build_atlas(required_texs);
    let mut tex_coords: Vec<(f32, f32)> = Vec::with_capacity(vertices.len());
    faces.iter().for_each(|face| {
        let texinfo = &texinfos[face.texinfo as usize];
        let tex = &mip_texs[texinfo.imip as usize]; // TODO : Make something with atlas
        let name = read_name(tex.name);
        let rect = atlas.slots.get(&name).unwrap();
        let x = rect.x as f32;
        let y = rect.y as f32;
        for i in 0..face.edges {
            let surfedge_i = face.first_edge + (i as u32);
            let surfedge = surfedges[surfedge_i as usize];
            let vert = if surfedge > 0 {
                edges[surfedge as usize].vertices[0]
            } else {
                edges[-surfedge as usize].vertices[1]
            };
            let vertex = &vertices[vert as usize];
            let s = (vertex.dot(&texinfo.vs) + texinfo.fs) / tex.width as f32;
            let t = (vertex.dot(&texinfo.vt) + texinfo.ft) / tex.height as f32;

            let u = (x + s) / (atlas.texture.width as f32);
            let v = (y + t) / (atlas.texture.height as f32);
            println!("{}x{}", s, t);
            tex_coords.insert(vert as usize, (u, v));
        }
    });
    ///////////////////////////////////////////////////////////////////////////////////////////////

    vertices.iter().for_each(|vertex| {
        writeln!(obj_writer, "v {} {} {}", vertex.0, vertex.1, vertex.2).unwrap();
    });
    write!(obj_writer, "\n\n\n").unwrap();

    tex_coords.iter().for_each(|&(u, v)| {
        writeln!(obj_writer, "vt {} {}", u, v).unwrap();
    });
    write!(obj_writer, "\n\n\n").unwrap();

    faces.iter().for_each(|face| {
        write!(obj_writer, "f").unwrap();
        for i in 0..face.edges {
            let surfedge_i = face.first_edge + (i as u32);
            let surfedge = surfedges[surfedge_i as usize];
            let vert = if surfedge > 0 {
                edges[surfedge as usize].vertices[0]
            } else {
                edges[-surfedge as usize].vertices[1]
            };
            write!(obj_writer, " {}/{}", vert + 1, vert + 1).unwrap();
        }
        writeln!(obj_writer).unwrap();
    });
    obj_writer.flush().unwrap();
    write_atlas(atlas_writer, atlas);
}

fn write_atlas<W: Write>(writer: &mut W, atlas: TextureAtlas) {
    use image::{ImageRgba8, ImageBuffer, Rgba, PNG};
    use std::mem::transmute;

    let texture = atlas.texture;

    let mut img_buffer = ImageBuffer::new(texture.width, texture.height);
    for (x, y, pixel) in img_buffer.enumerate_pixels_mut() {
        let color: [u8; 4] = unsafe { transmute(texture.get(x, y)) };
        *pixel = Rgba(color);
    }

    ImageRgba8(img_buffer).save(writer, PNG).unwrap();
}