use bsp::RawMap;
use map_impl::IndexedMap;

#[test]
fn print_vertices_indices_and_triangulated() {
    let file = std::fs::read(env!("BSP_TEST")).unwrap();
    let map = RawMap::parse(&file).unwrap();
    let map = IndexedMap::new(&map);
    let root_model = map.root_model();
    let vertices = map.calculate_vertices(root_model);
    let indices: Vec<_> = map
        .indices_with_texture(root_model)
        .into_iter()
        .map(|(_, i)| i)
        .collect();
    let triangulated_indices = map.indices_triangulated(root_model);

    println!("Vertices: {}", vertices.len());
    println!("Faces: {}", indices.len());
    println!("Indices (triangulated): {}", triangulated_indices.len());
}

#[test]
fn print_entities() {
    let file = std::fs::read(env!("BSP_TEST")).unwrap();
    let map = RawMap::parse(&file).unwrap();
    let map = IndexedMap::new(&map);

    let entities = map_impl::entities::Entities::parse(map.entities()).unwrap();
    println!("{:?}", entities);
}
