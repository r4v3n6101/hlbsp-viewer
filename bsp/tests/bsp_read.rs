use bsp::{lumps::*, LumpType, RawMap};

#[test]
fn print_entities_lump() {
    let file = std::fs::read(env!("BSP_TEST")).unwrap();
    let map = RawMap::parse(&file).unwrap();
    let data = map.lump_data(LumpType::Entities);
    println!("Entities: {}", parse_entities_str(data).unwrap());
}

#[test]
fn test_lumps_parsers() {
    let file = std::fs::read(env!("BSP_TEST")).unwrap();
    let map = RawMap::parse(&file).unwrap();

    let _ = (
        parse_vertices(map.lump_data(LumpType::Vertices)).unwrap(),
        parse_edges(map.lump_data(LumpType::Edges)).unwrap(),
        parse_ledges(map.lump_data(LumpType::Surfegdes)).unwrap(),
        parse_faces(map.lump_data(LumpType::Faces)).unwrap(),
        parse_normals_from_planes(map.lump_data(LumpType::Planes)).unwrap(),
        parse_texinfos(map.lump_data(LumpType::TexInfo)).unwrap(),
        parse_models(map.lump_data(LumpType::Models)).unwrap(),
    );
}
