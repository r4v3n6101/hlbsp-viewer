mod lumps;
mod parser;

use parser::Entities;
// TODO : remove
pub use lumps::parse_entities_str;

// TODO : remove
pub fn get_skyname(s: &str) -> Option<String> {
    let entities = Entities::parse(s).unwrap();
    entities
        .entities()
        .iter()
        .find_map(|e| e.properties().get("skyname"))
        .map(|e| e.to_string())
}
