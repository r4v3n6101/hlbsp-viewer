extern crate byteorder;

pub mod file;

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn wad_read() {
        let file = std::fs::File::open(env!("WAD_TEST")).unwrap();
        let mut wad_reader = file::WadReader::create(std::io::BufReader::new(file)).unwrap();
        let count = wad_reader.entries().unwrap().count();
        println!("{}", count);
        wad_reader
            .entries()
            .unwrap()
            .map(|e| e.unwrap().name)
            .for_each(|n| println!("{:?}", n));
    }
}
