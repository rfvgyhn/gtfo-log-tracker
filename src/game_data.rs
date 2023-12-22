use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StoryLog {
    pub id: u32,
    pub locations: Vec<Location>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub rundown: u8,
    pub level: String,
    pub zones: Vec<u16>,
    pub name: String,
}

pub fn get_id_from_name(name: &str, logs: &[StoryLog]) -> Option<u32> {
    logs.iter().find_map(|log| {
        if log.locations.iter().any(|loc| loc.name == name) {
            Some(log.id)
        } else {
            None
        }
    })
}

pub fn load_logs() -> Result<Vec<StoryLog>, Box<dyn Error>> {
    let logs = serde_json::from_str::<Vec<StoryLog>>(include_str!("../data/logs.json"))?;

    Ok(logs)
}