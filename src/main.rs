use itertools::Itertools;
use map_impl::miptex::MipTexture;
use map_impl::IndexedMap;
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    iter::Iterator,
    path::PathBuf,
};
use structopt::StructOpt;
use wad::Archive;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "hlbsp2obj",
    about = "A program allows you to convert hlbsp (v30 bsp) maps to obj model"
)]
struct Opt {
    #[structopt(
        short,
        long = "bsp",
        parse(from_os_str),
        help = "Path to bsp(v30) file"
    )]
    bsp_path: PathBuf,
    #[structopt(
        short,
        long,
        help = "Triangulate faces or not (will produce larger obj)"
    )]
    triangulate: bool,
    #[structopt(
        short,
        long,
        default_value = "out",
        parse(from_os_str),
        help = "Output directory where everything will be saved"
    )]
    output_dir: PathBuf,
    #[structopt(short, long, default_value = "0", help = "Mip level of textures")]
    mip_level: usize,
    #[structopt(
        short,
        long = "wads",
        parse(from_os_str),
        help = "Wad files with textures"
    )]
    wad_files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    let file_content = std::fs::read(&opt.bsp_path).expect("Failed reading bsp file");
    let bsp_map = bsp::RawMap::parse(&file_content).expect("Failed parsing bsp map");
    let mut map = IndexedMap::new(&bsp_map);
    let mut obj_writer = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(opt.output_dir.join("out.obj"))
            .expect("Failed opening output obj file"),
    );
    let mut mtl_writer = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(opt.output_dir.join("out.mtl"))
            .expect("Failed opening output obj file"),
    );
    let wad_files: Vec<_> = opt
        .wad_files
        .iter()
        .map(|path| std::fs::read(path).expect("Failed reading wad"))
        .collect();
    let archives: Vec<_> = wad_files
        .iter()
        .map(|file| Archive::parse(file).expect("Error parsing wad"))
        .collect();
    for archive in &archives {
        map.replace_empty_textures(archive);
    }
    print_vertices(&mut obj_writer, &map);
    writeln!(&mut obj_writer, "mtllib out.mtl").expect("Failed writing mtllib");
    print_faces_and_save_textures(
        &mut obj_writer,
        &mut mtl_writer,
        &opt.output_dir,
        &map,
        opt.triangulate,
        opt.mip_level,
    );
}

fn print_vertices<W: Write>(writer: &mut W, map: &IndexedMap) {
    let vertices = map.calculate_vertices(map.root_model());

    vertices.iter().for_each(|v| {
        writeln!(
            writer,
            "v {} {} {}",
            v.position[0], v.position[2], -v.position[1]
        )
        .expect("Failed writing position")
    });
    vertices.iter().for_each(|v| {
        writeln!(writer, "vt {} {}", v.tex_coords[0], 1.0 - v.tex_coords[1])
            .expect("Failed writing texture coords")
    });
    vertices.iter().for_each(|v| {
        writeln!(
            writer,
            "vn {} {} {}",
            v.normal[0], v.normal[2], -v.normal[1]
        )
        .expect("Failed writing normal")
    });
}

fn print_mtl_materials<W: Write>(mtl_writer: &mut W, name: &str) {
    writeln!(mtl_writer, "newmtl {}", name).expect("Failed writing mtl");
    writeln!(mtl_writer, "Tr 1").expect("Failed writing mtl");
    writeln!(mtl_writer, "map_Kd {}.png", name).expect("Failed writing mtl");
    writeln!(mtl_writer, "map_Ka {}.png", name).expect("Failed writing mtl");
}

fn print_faces_and_save_textures<W: Write>(
    obj_writer: &mut W,
    mtl_writer: &mut W,
    output_dir: &PathBuf,
    map: &IndexedMap,
    triangulate: bool,
    mip_level: usize,
) {
    let model = map.root_model();
    if triangulate {
        // TODO : will be replaced with new group by
        let mut indices = map.indices_triangulated(model);
        indices.sort_by(|(a, _), (b, _)| a.name().partial_cmp(b.name()).unwrap());
        for (key, group) in &indices.into_iter().group_by(|&(tex, _)| tex) {
            let name = key.name();
            writeln!(obj_writer, "usemtl {}", name).expect("Failed writing usemtl");
            print_mtl_materials(mtl_writer, name);
            save_texture(output_dir, key, mip_level);
            group.for_each(|(_, indices)| {
                for i in 0..indices.len() / 3 {
                    let v1 = indices[3 * i];
                    let v2 = indices[3 * i + 1];
                    let v3 = indices[3 * i + 2];
                    writeln!(
                        obj_writer,
                        "f {0}/{0}/{0} {1}/{1}/{1} {2}/{2}/{2}",
                        v1 + 1,
                        v2 + 1,
                        v3 + 1
                    )
                    .expect("Failed writing face");
                }
            });
        }
    } else {
        // TODO : same as above
        let mut indices = map.indices_with_texture(model);
        indices.sort_by(|(a, _), (b, _)| a.name().partial_cmp(b.name()).unwrap());
        for (key, group) in &indices.into_iter().group_by(|&(tex, _)| tex) {
            let name = key.name();
            writeln!(obj_writer, "usemtl {}", name).expect("Failed writing usemtl");
            print_mtl_materials(mtl_writer, name);
            save_texture(output_dir, key, mip_level);
            group.for_each(|(_, indices)| {
                let mut s = String::from("f ");
                indices
                    .into_iter()
                    .for_each(|i| s += &format!("{0}/{0}/{0} ", i + 1));
                writeln!(obj_writer, "{}", s).expect("Failed writing face");
            });
        }
    }
}

fn save_texture(output_dir: &PathBuf, texture: &MipTexture, mip_level: usize) {
    let name = texture.name();
    let (width, height) = (
        texture.width(mip_level).expect("Failed acquiring width"),
        texture.height(mip_level).expect("Failed acquiring height"),
    );
    if !texture.is_empty() {
        let file_name = String::from(texture.name()) + ".png";
        image::save_buffer(
            output_dir.join(file_name),
            &texture
                .pixels(mip_level)
                .expect("Failed acquiring pixels of texture"),
            width,
            height,
            image::ColorType::Rgb8,
        )
        .expect("Failed saving texture");
    } else {
        println!("Not found: {}", name);
    }
}
