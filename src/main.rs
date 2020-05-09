extern crate hlbsp;
extern crate image;
extern crate structopt;

use std::{
    collections::{HashMap, HashSet},
    fs::read,
    fs::File,
    io::{BufWriter, Write},
    iter::Iterator,
    path::{PathBuf, Path},
};

use hlbsp::{bsp::*, read_struct, texture::Texture, wad::entries};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tools helps you convert a .bsp file to .obj with .wad textures")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(default_value = "out", parse(from_os_str))]
    outputdir: PathBuf,

    #[structopt(long, short)]
    wad: Option<Vec<PathBuf>>,

    #[structopt(long, short, default_value = "0")]
    mip_level: u8,
}

fn main() {
    let opt = Opt::from_args();
    process(&opt);
}

fn process(opt: &Opt) {
    let bsp = read_bsp(&opt.input);
    let mut uvs: Vec<UV> = Vec::with_capacity(bsp.vertices.len());
    let mut normals: Vec<Vec3> = Vec::with_capacity(bsp.vertices.len());
    let mut groups = String::new();

    println!("Prepare data");
    prepare_data(&bsp, &mut uvs, &mut normals, &mut groups);

    println!("Write .obj");
    write_obj(&opt.outputdir, &bsp.vertices, &normals, &uvs, &groups);
    if let Some(wads) = &opt.wad {
        println!("Write .mtl and .png textures");
        write_mtl(&opt.outputdir, wads, &bsp.miptexs, opt.mip_level);
    }
}

fn read_bsp(bsp_path: &PathBuf) -> BspMap {
    let bytes = read(bsp_path).unwrap();
    return BspMap::new(&bytes).unwrap();
}

type UV = (f32, f32);

fn write_obj(outputdir: &PathBuf, vertices: &[Vertex], normals: &[Vec3], uvs: &[UV], groups: &str){
   let obj_path = outputdir.join("out.obj");
   let file = File::open(obj_path).unwrap();
   let mut writer = BufWriter::new(file);

   vertices
        .iter()
        .for_each(|v| writeln!(writer, "v {} {} {}", v.0, v.2, -v.1).unwrap());
    uvs.iter()
        .for_each(|&(u, v)| writeln!(writer, "vt {} {}", u, 1.0 - v).unwrap());
    normals
        .iter()
        .for_each(|n| writeln!(writer, "vn {} {} {}", n.0, n.2, -n.1).unwrap());
    writeln!(writer, "{}", groups).unwrap();
}

fn write_mtl(outputdir: &PathBuf, wads: &Vec<PathBuf>, miptexs: &Vec<MipTex>, mip_level: u8) {
    let mtl_path = outputdir.join("out.mtl");
    let file = File::open(mtl_path).unwrap();
    let mut writer = BufWriter::new(file);

    let textures: HashSet<String> = miptexs.iter().map(|mip| mip.get_name()).collect();
    iter_wads(wads, mip_level)
        .filter(|(name, _)| textures.contains(name))
        .for_each(|(name, tex)| {
            writeln!(writer, "newmtl {}", name).unwrap();
            writeln!(writer, "map_Kd {}.png", name).unwrap();
            writeln!(writer, "Tr 1").unwrap();

            let tex_path = outputdir.join(name);
            write_image(tex_path, &tex);
        });
}

fn prepare_data(map: &BspMap, uvs: &mut Vec<UV>, normals: &mut Vec<Vec3>, groups: &mut String) {
    for (i, model) in map.models.iter().enumerate() {
        // I know it's dirty code
        type TextureGroup = Vec<String>;
        let mut tex_groups: HashMap<String, TextureGroup> = HashMap::new();

        for f in 0..model.faces {
            let face = &map.faces[(model.first_face + f) as usize];
            let texinfo = &map.texinfos[face.texinfo as usize];
            let miptex = &map.miptexs[texinfo.imip as usize];

            let name = miptex.get_name();
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

fn write_image<P:AsRef<Path>>(output_path: P, texture: &Texture) {
    use image::{ImageBuffer, ImageRgba8, Rgba};
    use std::mem::transmute;

    let mut img_buffer = ImageBuffer::new(texture.width, texture.height);
    for (x, y, pixel) in img_buffer.enumerate_pixels_mut() {
        let color: [u8; 4] = unsafe { transmute(texture.get(x, y)) };
        *pixel = Rgba(color);
    }

    ImageRgba8(img_buffer)
        .save(output_path)
        .expect("Error while writing image");
}

type NamedTexture = (String, Texture);

fn iter_wad<P: AsRef<Path>>(path: P, mip_level: u8) -> impl Iterator<Item = NamedTexture> {
    let buf: Vec<u8> = read(path).unwrap();
    entries(&buf).into_iter().map(move |entry| {
        let mip = &buf[entry.file_pos as usize..];
        let miptex: MipTex = read_struct(mip); // todo : specify offset not to produce slices
        let name = miptex.get_name();
        let tex = miptex.get_texture(mip, mip_level);
        (name, tex)
    })
}

fn iter_wads<'a, P: AsRef<Path>>(
    wads: &'a [P],
    mip_level: u8,
) -> impl Iterator<Item = NamedTexture> + 'a {
    wads.iter().flat_map(move |path| iter_wad(path, mip_level))
}
