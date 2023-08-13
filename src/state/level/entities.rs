use goldsrc_rs::map::{Entities, Entity};

const INFO_PLAYER_START_CLASSNAME: &str = "info_player_start";

pub type Vec3 = (f32, f32, f32);

pub fn get_skyname(entities: &Entities) -> Option<String> {
    entities
        .iter()
        .find_map(|e| e.get("skyname"))
        .map(|e| e.to_string())
}

pub fn find_info_player_start(entities: &Entities) -> Option<&Entity> {
    entities.iter().find(|e| {
        e
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
        .get("origin")
        .and_then(|o| parse_vector3(o))
}
