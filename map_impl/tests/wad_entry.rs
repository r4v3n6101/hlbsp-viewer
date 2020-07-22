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
            let mut imgbuf = image::ImageBuffer::new(width, height);
            for x in 0..width {
                for y in 0..height {
                    *imgbuf.get_pixel_mut(x, y) =
                        image::Rgb(miptex.color(mip_level, x, y).unwrap());
                }
            }
            let file_name = String::from(file_name) + ".png";
            println!("Saved {}", file_name);
            imgbuf.save(output_dir.join(file_name)).unwrap();
        });
}
