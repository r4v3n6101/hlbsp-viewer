pub mod io;
pub mod map;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_smth() {
        let file = std::fs::File::open(env!("BSP_TEST")).unwrap();
        let reader = io::BspMapReader::create(file).unwrap();
        let map = map::Map::new(&reader).unwrap();
        assert!(map.textures().iter().all(|tex| tex.color_table.is_none()));
    }
}
