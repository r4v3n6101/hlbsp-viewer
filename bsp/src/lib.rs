extern crate cstr;
extern crate byteorder;
extern crate cgmath;

pub mod io;
pub mod lump;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_smth() {
        use lump::LumpReader;
        use std::ffi::CString;
        let file = std::fs::File::open(env!("BSP_TEST")).unwrap();
        let mut ents = io::BspMapReader::create(file).unwrap();
        let lump = ents.read_lump(io::LumpType::Entities).unwrap();
        let cstr: CString = lump::Entity::read(lump).unwrap();
        println!("{}", cstr.to_str().unwrap());
    }
}
