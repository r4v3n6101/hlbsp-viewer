extern crate hlbsp2obj;
extern crate image;

use hlbsp2obj::{
    bsp::*,
    read_struct,
    texture::Texture,
    wad::{entries, read_name},
};
use std::{
    collections::{HashMap, HashSet},
    env::args,
    fs::{create_dir, File, read_dir, ReadDir, remove_dir_all},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

fn main() {
    let bsp_path = args().nth(1).expect("bsp path");
    let output_path = bsp_path.replace(".bsp", "/");

    println!("Reading {}", bsp_path);
    let bsp_file = File::open(bsp_path).expect("bsp file error");
    let size = bsp_file.metadata().unwrap().len() + 1;
    let mut bsp: Vec<u8> = Vec::with_capacity(size as usize);
    let mut buf_reader = BufReader::new(bsp_file);
    buf_reader.read_to_end(&mut bsp).unwrap();

    let dir_path = Path::new(&output_path);
    if dir_path.exists() {
        println!("Removing old dir");
        remove_dir_all(dir_path).expect("Error while removing output dir");
    }
    create_dir(dir_path).expect("Error while creating output dir");
    process(&bsp, &output_path);
}

type UV = (f32, f32);
type TextureMap = HashMap<String, Texture>;

fn write_obj<W: Write>(
    out: &mut W, vertices: &Vec<Vertex>, normals: &Vec<Vec3>, uvs: &Vec<UV>, groups: &String,
) {
    vertices.iter().for_each(|v| writeln!(out, "v {} {} {}", v.0, v.2, -v.1).unwrap());
    uvs.iter().for_each(|&(u, v)| writeln!(out, "vt {} {}", u, 1.0 - v).unwrap());
    normals.iter().for_each(|n| writeln!(out, "vn {} {} {}", n.0, n.2, -n.1).unwrap());
    writeln!(out, "{}", groups).unwrap();
}

fn write_mtl<W: Write>(out: &mut W, miptexs: &Vec<MipTex>, output_dir: &str) {
    let wad_path = args().nth(2).expect("wad path");
    let wad_dir = read_dir(wad_path).unwrap();
    let required: HashSet<String> =
        miptexs.iter().map(|miptex| { read_name(miptex.name) }).collect();
    println!("Required textures: {:?}", required);
    let textures: TextureMap = found_textures(wad_dir, required, 0);
    textures.iter().for_each(|(name, texture)| {
        writeln!(out, "newmtl {}", name).unwrap();
        writeln!(out, "map_Kd {}.png", name).unwrap();
        writeln!(out, "Tr 1").unwrap();

        let file = File::create(format!("{}{}.png", output_dir, name)).expect("png file error");
        let mut buf_writer = BufWriter::new(file);
        write_image(&mut buf_writer, texture);
    });
}

fn prepare_data(
    map: &BspMap, uvs: &mut Vec<UV>, normals: &mut Vec<Vec3>, groups: &mut String,
) {
    for (i, model) in map.models.iter().enumerate() {
        // I know it's dirty code
        type TextureGroup = Vec<String>;
        let mut tex_groups: HashMap<String, TextureGroup> = HashMap::new();

        for f in 0..model.faces {
            let face = &map.faces[(model.first_face + f) as usize];
            let texinfo = &map.texinfos[face.texinfo as usize];
            let miptex = &map.miptexs[texinfo.imip as usize];

            let name = read_name(miptex.name);
            if name == "sky" || name == "aaatrigger" {
                continue;
            }

            let tex_group = tex_groups.entry(name).or_insert(vec![]);

            let width = miptex.width as f32;
            let height = miptex.height as f32;

            let plane = &map.planes[face.plane as usize];
            let normal = if face.plane_side == 0 {
                plane.normal.clone()
            } else {
                -plane.normal.clone()
            };
            normals.push(normal);
            let normal_id = normals.len();

            let mut face_str = String::from("f ");
            for i in 0..face.edges {
                let surfedge = map.surfedges[(face.first_edge + (i as u32)) as usize];
                let vert = if surfedge > 0 {
                    map.edges[surfedge as usize].0
                } else {
                    map.edges[-surfedge as usize].1
                };
                let vertex = &map.vertices[vert as usize];
                let s = (vertex * &texinfo.vs) + texinfo.fs;
                let t = (vertex * &texinfo.vt) + texinfo.ft;
                uvs.push((s / width, t / height));
                face_str += &format!(" {}/{}/{}", vert + 1, uvs.len(), normal_id);
            }
            tex_group.push(face_str);
        }
        groups.push_str(&format!("g {}\n", i));
        tex_groups.iter().for_each(|(tex_name, faces)| {
            groups.push_str(&format!("usemtl {}\n", tex_name));
            faces.iter().for_each(|face_str| {
                groups.push_str(face_str);
                groups.push_str("\n");
            });
        });
    }
}

fn process(bsp: &[u8], output_dir: &String) {
    let map = BspMap::new(bsp).unwrap();

    let vertices = &map.vertices;
    let mut tex_coords: Vec<(f32, f32)> = Vec::with_capacity(vertices.len());
    let mut normals: Vec<Vec3> = Vec::with_capacity(vertices.len());
    let mut groups = String::new();
    println!("Collecting data");
    prepare_data(&map, &mut tex_coords, &mut normals, &mut groups);

    println!("Writing obj file");
    let obj_file = File::create(format!("{}out.obj", output_dir)).expect("obj file error");
    let mut obj_writer = BufWriter::new(obj_file);
    write_obj(&mut obj_writer, vertices, &normals, &tex_coords, &groups);
    obj_writer.flush().unwrap();

    println!("Writing textures & materials");
    let mtl_file = File::create(format!("{}out.mtl", output_dir)).expect("mtl file error");
    let mut mtl_writer = BufWriter::new(mtl_file);
    write_mtl(&mut mtl_writer, &map.miptexs, output_dir);
    mtl_writer.flush().unwrap();
}

fn write_image<W: Write>(out: &mut W, texture: &Texture) {
    use image::{ImageRgba8, ImageBuffer, Rgba, PNG};
    use std::mem::transmute;

    let mut img_buffer = ImageBuffer::new(texture.width, texture.height);
    for (x, y, pixel) in img_buffer.enumerate_pixels_mut() {
        let color: [u8; 4] = unsafe { transmute(texture.get(x, y)) };
        *pixel = Rgba(color);
    }

    ImageRgba8(img_buffer).save(out, PNG).expect("Error while writing image");
}

fn found_textures(dir: ReadDir, required: HashSet<String>, mip_level: usize) -> TextureMap {
    fn read_file(wad_file: File, required: &HashSet<String>, mip_level: usize) -> TextureMap {
        let size = wad_file.metadata().unwrap().len() + 1;
        let mut wad: Vec<u8> = Vec::with_capacity(size as usize);
        let mut buf_reader = BufReader::new(wad_file);
        buf_reader.read_to_end(&mut wad).unwrap();

        entries(&wad).iter().filter_map(|e| {
            let name = read_name(e.name);
            if required.contains(&name) {
                let tex_slice = &wad[e.file_pos as usize..];
                let miptex: MipTex = read_struct(tex_slice);
                let color_table = miptex.get_color_table(tex_slice);
                let texture = miptex.read_texture(tex_slice, color_table, mip_level);
                Some((name, texture))
            } else {
                None
            }
        }).collect()
    }

    dir.filter_map(|entry| {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or("".as_ref()) == "wad" {
            Some(path)
        } else {
            None
        }
    }).flat_map(|path| {
        let wad_file = File::open(path).unwrap();
        read_file(wad_file, &required, mip_level)
    }).collect()
}