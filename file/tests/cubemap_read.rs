#[test]
fn read_cubemap() {
    let cubemap = file::cubemap::Cubemap::read(env!("CUBEMAP_NAME"), env!("CUBEMAP_DIR")).unwrap();
    assert_eq!(cubemap.dimension(), 256);
    assert!(cubemap.sides().iter().all(|x| x.len() == 256 * 256 * 4));
}
