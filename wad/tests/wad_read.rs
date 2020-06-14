#[test]
fn check_wad_entries_length() {
    let data = std::io::Cursor::new(std::fs::read(env!("WAD_TEST")).unwrap());
    let wad_reader = wad::io::WadReader::create(data).unwrap();
    let entries = wad_reader.entries();
    assert_eq!(entries.len(), 3116);
}

#[test]
fn print_wad_entry_names() {
    let data = std::io::Cursor::new(std::fs::read(env!("WAD_TEST")).unwrap());
    let wad_reader = wad::io::WadReader::create(data).unwrap();
    wad_reader.entries().iter().for_each(|data| {
        let wad_name_upper = data.name().to_str().unwrap();
        let data = wad_reader.read_entry(data).unwrap();
        let miptex = wad::miptex::MipTexture::new(&data).unwrap();
        let miptex_name = miptex.name.to_str().unwrap();
        assert_eq!(wad_name_upper.to_lowercase(), miptex_name.to_lowercase());
    });
}
