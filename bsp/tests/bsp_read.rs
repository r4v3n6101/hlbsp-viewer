#[test]
fn check_that_there_is_no_textures_in_bsp() {
    let file = std::fs::File::open(env!("BSP_TEST")).unwrap();
    let reader = bsp::io::BspMapReader::create(file).unwrap();
    let map = bsp::map::Map::new(&reader).unwrap();
    assert!(map.textures.iter().all(|tex| tex.color_table.is_none()));
}
