use std::path::PathBuf;
use structopt::StructOpt;

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
}

fn main() {
    let opt = Opt::from_args();
    let file_content = std::fs::read(&opt.bsp_path).expect("Failed reading bsp file");
    let bsp_map = bsp::RawMap::parse(&file_content).expect("Failed parsing bsp map");
    let map_render = bsp::map_impl::IndexedMap::new(&bsp_map);
    print_vertices(&map_render);
    print_indices(&map_render, opt.triangulate);
}

fn print_vertices(map_render: &bsp::map_impl::IndexedMap) {
    let vertices = map_render.calculate_vertices(map_render.root_model());
    vertices
        .into_iter()
        .map(|v| v.position)
        .for_each(|v| println!("v {} {} {}", v[0], v[2], -v[1]));
}

fn print_indices(map_render: &bsp::map_impl::IndexedMap, triangulate: bool) {
    let model = map_render.root_model();
    if triangulate {
        let indices = map_render.indices_triangulated(model);
        for i in 0..indices.len() / 3 {
            let v1 = indices[3 * i];
            let v2 = indices[3 * i + 1];
            let v3 = indices[3 * i + 2];
            println!("f {} {} {}", v1 + 1, v2 + 1, v3 + 1);
        }
    } else {
        let indices = map_render.indices(model);
        indices.into_iter().for_each(|indices| {
            let mut s = String::from("f ");
            indices
                .into_iter()
                .for_each(|i| s += &format!("{} ", i + 1));
            println!("{}", s);
        });
    }
}
