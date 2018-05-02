extern crate hlbsp2obj;
extern crate image;

use hlbsp2obj::{
    bsp::*,
    read_mul_structs,
    read_struct,
    texture::{HashMap, MipTex, Path, read_textures, Texture},
    wad::read_name,
};
use std::{
    env::args,
    fs::{create_dir, File, remove_dir_all},
    io::{BufReader, BufWriter, Read, Write},
};

fn main() {
    let bsp_path = args().nth(1).expect("bsp path");
    let wad_path = args().nth(2).expect("wad path");
    let output_path = bsp_path.replace(".bsp", "/");

    println!("Reading {}", bsp_path);
    let bsp_file = File::open(bsp_path).expect("bsp file error");
    let size = bsp_file.metadata().unwrap().len() + 1;
    let mut bsp: Vec<u8> = Vec::with_capacity(size as usize);
    let mut buf_reader = BufReader::new(bsp_file);
    buf_reader.read_to_end(&mut bsp).unwrap();

    println!("Reading textures");
    let textures: HashMap<String, Texture> = read_textures(wad_path.as_ref(), 0);

    let dir_path = Path::new(&output_path);
    if dir_path.exists() {
        println!("Found old dir, remove this");
        remove_dir_all(dir_path).expect("Error while removing output dir");
    }
    create_dir(dir_path).expect("Error while creating output dir");
    write_obj(&bsp, &textures, &output_path);
}

fn read_mip_tex(tex: &[u8]) -> Vec<MipTex> {
    let mip_off: u32 = read_struct(tex);
    let offsets: Vec<u32> = read_mul_structs(&tex[4..4 + mip_off as usize * 4]);
    return offsets.iter().map(|i| read_struct(&tex[*i as usize..])).collect();
}

fn write_obj(bsp: &[u8], wad_textures: &HashMap<String, Texture>, output_dir: &String) {
    let obj_file = File::create(format!("{}out.obj", output_dir)).expect("obj file error");
    let mut obj_writer = BufWriter::new(obj_file);
    let mtl_file = File::create(format!("{}out.mtl", output_dir)).expect("mtl file error");
    let mut mtl_writer = BufWriter::new(mtl_file);

    println!("Reading data from bsp");
    let header: Header = read_struct(&bsp);
    let vertices: Vertices = header.lumps[LUMP_VERTICES].read_array(&bsp);
    let faces: Vec<Face> = header.lumps[LUMP_FACES].read_array(&bsp);
    let surfedges: Vec<i32> = header.lumps[LUMP_SURFEDGES].read_array(&bsp);
    let edges: Vec<Edge> = header.lumps[LUMP_EDGES].read_array(&bsp);
    let texinfos: Vec<TexInfo> = header.lumps[LUMP_TEXINFO].read_array(&bsp);
    let texs_offset: usize = header.lumps[LUMP_TEXTURES].offset as usize;
    let miptexs: Vec<MipTex> = read_mip_tex(&bsp[texs_offset..]);

    println!("Process");
    let mut tex_coords: Vec<(f32, f32)> = Vec::with_capacity(vertices.len());
    type TextureGroup = Vec<String>;
    let mut tex_groups: HashMap<String, TextureGroup> = HashMap::with_capacity(miptexs.len());
    for face in faces {
        let texinfo = &texinfos[face.texinfo as usize];
        let miptex = &miptexs[texinfo.imip as usize];
        let name = read_name(miptex.name);
        if name == "sky" || name == "aaatrigger" {
            continue;
        }
        let group = tex_groups.entry(name).or_insert(vec![]);
        let width = miptex.width as f32;
        let height = miptex.height as f32;
        let mut face_str = String::from("f ");
        for i in 0..face.edges {
            let surfedge = surfedges[(face.first_edge + (i as u32)) as usize];
            let vert = if surfedge > 0 {
                edges[surfedge as usize].vertices[0]
            } else {
                edges[-surfedge as usize].vertices[1]
            };
            let vertex = &vertices[vert as usize];
            let s = (vertex * &texinfo.vs) + texinfo.fs;
            let t = (vertex * &texinfo.vt) + texinfo.ft;
            tex_coords.push((s / width, t / height));
            face_str += &format!(" {}/{}", vert + 1, tex_coords.len());
        }
        group.push(face_str);
    }

    println!("Writing data");
    writeln!(obj_writer, "mtllib out.mtl").unwrap();
    vertices.iter().for_each(|v| writeln!(obj_writer, "v {} {} {}", v.0, v.2, -v.1).unwrap());
    tex_coords.iter().for_each(|&(u, v)| writeln!(obj_writer, "vt {} {}", u, 1f32 - v).unwrap());
    tex_groups.iter().for_each(|(name, group)| {
        writeln!(obj_writer, "usemtl {}", name).unwrap();
        group.iter().for_each(|f| writeln!(obj_writer, "{}", f).unwrap());

        writeln!(mtl_writer, "newmtl {}", name).unwrap();
        writeln!(mtl_writer, "Tr 1").unwrap();
        writeln!(mtl_writer, "map_Kd {}.png", name).unwrap();
        writeln!(mtl_writer, "map_Ka {}.png", name).unwrap();

        let file = File::create(format!("{}{}.png", output_dir, name)).expect("png file error");
        let mut buf_writer = BufWriter::new(file);
        let texture = wad_textures.get(name).expect(&format!("Not found {}", name));
        println!("Writing texture: {}", name);
        write_image(&mut buf_writer, texture);
    });
    mtl_writer.flush().unwrap();
    obj_writer.flush().unwrap();
}

fn write_image<W: Write>(writer: &mut W, image: &Texture) {
    use image::{ImageRgba8, ImageBuffer, Rgba, PNG};
    use std::mem::transmute;

    let mut img_buffer = ImageBuffer::new(image.width, image.height);
    for (x, y, pixel) in img_buffer.enumerate_pixels_mut() {
        let color: [u8; 4] = unsafe { transmute(image.get(x, y)) };
        *pixel = Rgba(color);
    }

    ImageRgba8(img_buffer).save(writer, PNG).expect("Error while writing image");
}