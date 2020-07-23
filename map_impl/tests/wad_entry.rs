use map_impl::miptex::MipTexture;

#[test]
fn test_entry_and_miptex_names() {
    let file = std::fs::read(env!("WAD_TEST")).unwrap();
    let wad = wad::Archive::parse(&file).unwrap();

    wad.entries().for_each(|(&file_name, e)| {
        let wad_name = file_name;
        let miptex = MipTexture::parse(e.data()).unwrap();
        let miptex_name = miptex.name();
        assert!(wad_name.eq_ignore_ascii_case(miptex_name));
    });
}

#[test]
fn export_entries() {
    let output_dir = std::path::PathBuf::from(env!("OUT_DIR"));
    let mip_level = 0;
    let file = std::fs::read(env!("WAD_TEST")).unwrap();
    let wad = wad::Archive::parse(&file).unwrap();

    wad.entries()
        .map(|(&file_name, e)| (file_name, MipTexture::parse(e.data()).unwrap()))
        .for_each(|(file_name, miptex)| {
            let (width, height) = (
                miptex.width(mip_level).unwrap(),
                miptex.height(mip_level).unwrap(),
            );
            let file_name = String::from(file_name) + ".png";
            image::save_buffer(
                output_dir.join(file_name),
                &miptex.pixels(mip_level).unwrap(),
                width,
                height,
                image::ColorType::Rgb8,
            )
            .unwrap();
            println!("Saved {}", file_name);
        });
}
