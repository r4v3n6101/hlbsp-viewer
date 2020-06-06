extern crate byteorder;
extern crate cstr;

pub mod io;
pub mod miptex;

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
        wad_reader.entries().for_each(|data| {
            let data = wad_reader.read_entry(data);
            let miptex = miptex::MipTexture::new(data.unwrap()).unwrap();
            println!("MipTex name: {:?}", miptex.name);
        });
    }
}
