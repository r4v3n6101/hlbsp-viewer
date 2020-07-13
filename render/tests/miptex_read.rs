#[test]
fn compare_wad_miptex_names() {
    let file = std::fs::read(env!("WAD_TEST")).unwrap();
    let wad = wad::Archive::parse(&file).unwrap();

    wad.entries().for_each(|(&file_name, e)| {
        let wad_name = file_name;
        let miptex = render::miptex::MipTexture::parse(e.data()).unwrap();
        let miptex_name = miptex.name();
        assert!(wad_name.eq_ignore_ascii_case(miptex_name));
    });
}
