extern crate hlbsp2obj;
extern crate image;

use hlbsp2obj::{
    bsp::*,
    read_struct,
    texture::{build_atlas, HashMap, MipTex, Texture, TextureAtlas},
    wad::{
        entries,
        read_name,
    },
};
use std::{
    env::args,
    fs::*,
    io::{
        BufReader,
        BufWriter,
        Read,
        Write,
    },
    path::*,
};

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

    let wad_path = Path::new(&wad_path);
    let textures: HashMap<String, Texture> = if wad_path.is_dir() {
        read_wad_dir(wad_path, 0)
    } else {
        read_wad(wad_path, 0)
    };
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

    let offset = header.lumps[LUMP_TEXTURES].offset as usize;
    let mip_off: u32 = read_struct(&bsp[offset..]);
    let offsets: Vec<i32> = read_mul_structs(&bsp[offset + 4..offset + mip_off as usize * 4]);
    let mip_texs: Vec<MipTex> =
        offsets.iter().map(|i| read_struct(&bsp[offset + *i as usize..])).collect();

    let required_texs: HashMap<String, &Texture> = mip_texs.iter().map(|mip_tex| {
        let name = read_name(mip_tex.name);
        let tex = wad_textures.get(&name).unwrap();
        (name, tex)
    }).collect();

    let atlas = build_atlas(required_texs);

    vertices.iter().for_each(|vertex| {
        writeln!(obj_writer, "v {} {} {}", vertex.0, vertex.1, vertex.2).unwrap();
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
            write!(obj_writer, " {}", vert + 1).unwrap();
        }
        writeln!(obj_writer).unwrap();
    });
    obj_writer.flush().unwrap();
    write_atlas(atlas_writer, atlas);
    // TODO : Add texture support
}

fn read_wad_dir<P: AsRef<Path>>(path: P, mip_level: usize) -> HashMap<String, Texture> {
    let dir = read_dir(path).unwrap();
    dir.flat_map(|entry| read_wad(entry.unwrap().path(), mip_level)).collect()
}

fn read_wad<P: AsRef<Path>>(path: P, mip_level: usize) -> HashMap<String, Texture> {
    let wad_file = File::open(path).unwrap();
    let size = wad_file.metadata().unwrap().len() + 1;
    let mut wad: Vec<u8> = Vec::with_capacity(size as usize);

    let mut buf_reader = BufReader::new(wad_file);
    buf_reader.read_to_end(&mut wad).unwrap();

    entries(&wad).iter().map(|entry| {
        let tex_offset = entry.file_pos as usize;
        let tex: MipTex = read_struct(&wad[tex_offset..]);

        let col_table = tex.get_color_table(&wad[tex_offset..]);
        let texture = tex.read_texture(&wad[tex_offset..], col_table, mip_level);
        let name = read_name(tex.name);
        (name, texture)
    }).collect()
}

fn write_atlas<W: Write>(writer: &mut W, atlas: TextureAtlas) {
    use image::*;
    use std::mem::transmute;

    let texture = atlas.texture;

    let mut img_buffer = ImageBuffer::new(texture.width, texture.height);
    for (x, y, pixel) in img_buffer.enumerate_pixels_mut() {
        let color: [u8; 4] = unsafe { transmute(texture.get(x, y)) };
        *pixel = Rgba(color);
    }

    ImageRgba8(img_buffer).save(writer, PNG).unwrap();
}