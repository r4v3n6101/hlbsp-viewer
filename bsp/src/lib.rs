pub mod io;
pub mod lump;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_smth() {
        let file = std::fs::File::open(env!("BSP_TEST")).unwrap();
        let ents = io::BspMapReader::create(file).unwrap();
        let lump = ents.read_lump(io::LumpType::Entities).unwrap();
        let cstr = lump::read_entity(lump).unwrap();
        println!("{}", cstr.to_str().unwrap());
    }
}
