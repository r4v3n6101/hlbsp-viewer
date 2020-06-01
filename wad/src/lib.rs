extern crate byteorder;

pub mod file;

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn wad_read() {
        let data = std::io::Cursor::new(std::fs::read(env!("WAD_TEST")).unwrap());
        let mut wad_reader = file::WadReader::create(data).unwrap();
        let entries = wad_reader.read_entries().unwrap();
        assert_eq!(entries.len(), 3116);
        for e in &entries {
            println!("{:?}", e.name);
        }
    }
}
