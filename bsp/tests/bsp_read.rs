#[test]
fn print_vertices_num() {
    let file = std::fs::read(env!("BSP_TEST")).unwrap();
    let map = bsp::Map::parse(&file).unwrap();
    println!(
        "Vertices: {}",
        map.lump_data(bsp::LumpType::Vertices).len() / (4 * 3)
    );
}

#[test]
fn print_entities_lump() {
    let file = std::fs::read(env!("BSP_TEST")).unwrap();
    let map = bsp::Map::parse(&file).unwrap();
    println!(
        "Entities: {}",
        std::str::from_utf8(map.lump_data(bsp::LumpType::Entities)).unwrap()
    );
}
