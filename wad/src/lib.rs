pub mod io;
pub mod miptex;
mod name;

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn wad_read() {
        let data = std::io::Cursor::new(std::fs::read(env!("WAD_TEST")).unwrap());
        let wad_reader = io::WadReader::create(data).unwrap();
        let entries = wad_reader.entries();
        assert_eq!(entries.len(), 3116);
    }

    #[test]
    fn read_wad_miptex() {
        let data = std::io::Cursor::new(std::fs::read(env!("WAD_TEST")).unwrap());
        let wad_reader = io::WadReader::create(data).unwrap();
        wad_reader.entries().iter().for_each(|data| {
            println!("WadEntry name: {:?}", data.name().to_str().unwrap());
            let data = wad_reader.read_entry(data).unwrap();
            let miptex = miptex::MipTexture::new(&data).unwrap();
            println!("MipTex name: {:?}", miptex.name);
        });
    }
}
