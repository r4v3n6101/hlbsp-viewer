mod lumps;
mod parser;

use lumps::parse_entities_str;
pub use parser::{Entities, Entity};

const INFO_PLAYER_START_CLASSNAME: &'static str = "info_player_start";

pub type Vec3 = (f32, f32, f32);

pub fn parse_entities(i: &[u8]) -> Option<Entities> {
    let s = parse_entities_str(i).ok()?; // TODO : do not ok
    Entities::parse(s).ok() // TODO : same as above
}

pub fn get_skyname(entities: &Entities) -> Option<String> {
    entities
        .entities()
        .iter()
        .find_map(|e| e.properties().get("skyname"))
        .map(|e| e.to_string())
}

pub fn find_info_player_start<'a>(entities: &'a Entities) -> Option<&'a Entity<'a>> {
    entities.entities().iter().find(|e| {
        e.properties()
            .get("classname")
            .filter(|&classname| classname == &INFO_PLAYER_START_CLASSNAME)
            .is_some()
    })
}

fn parse_vector3(s: &str) -> Option<Vec3> {
    let mut n = s.split(' ').flat_map(|e| e.parse().ok());
    Some((n.next()?, n.next()?, n.next()?))
}

pub fn get_start_point(entity: &Entity) -> Option<Vec3> {
    entity
        .properties()
        .get("origin")
        .and_then(|o| parse_vector3(o))
}
