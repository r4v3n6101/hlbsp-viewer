use bsp::{map_impl::IndexedMap, RawMap};

#[test]
fn print_vertices_indices_and_triangulated() {
    let file = std::fs::read(env!("BSP_TEST")).unwrap();
    let map = RawMap::parse(&file).unwrap();
    let gfx_map = IndexedMap::new(&map);
    let root_model = gfx_map.root_model();
    let vertices = gfx_map.calculate_vertices(root_model);
    let indices = gfx_map.indices(root_model);
    let triangulated_indices = gfx_map.indices_triangulated(root_model);

    println!("Vertices: {}", vertices.len());
    println!("Faces: {}", indices.count());
    println!("Indices (triangulated): {}", triangulated_indices.len());
}
