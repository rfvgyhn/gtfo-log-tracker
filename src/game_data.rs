use crate::steam;
use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoryLog {
    pub id: u32,
    pub locations: Vec<Location>,
}

#[derive(Deserialize, Debug, Clone)]
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

// Parses strings like Local_32,2,0 to R1B1
pub fn get_level_from_local(id: &str) -> Option<String> {
    let parts: Vec<&str> = id.split(',').collect();
    if parts.len() != 3 {
        return None;
    }

    let rundown = match parts[0] {
        "Local_32" => 1,
        "Local_33" => 2,
        "Local_34" => 3,
        "Local_37" => 4,
        "Local_38" => 5,
        "Local_41" => 6,
        "Local_31" => 7,
        "Local_35" => 8,
        _ => return None,
    };
    let tier = match parts[1] {
        "1" => "A",
        "2" => "B",
        "3" => "C",
        "4" => "D",
        "5" => "E",
        _ => return None,
    };
    let expedition = parts[2].parse::<u8>().ok()? + 1;

    Some(format!("R{rundown}{tier}{expedition}"))
}

pub fn load_logs() -> Result<Vec<StoryLog>> {
    let logs = serde_json::from_str::<Vec<StoryLog>>(include_str!("../data/logs.json"))?;

    Ok(logs)
}

#[cfg(target_os = "linux")]
pub fn find_user_data_path() -> Result<PathBuf> {
    steam::find_proton_app_data_path()
        .map(|p| p.join("LocalLow/10 Chambers Collective/GTFO"))
        .ok_or_else(|| anyhow!("Couldn't find compatdata AppData path"))
}

#[cfg(target_os = "windows")]
pub fn find_user_data_path() -> Result<PathBuf> {
    let app_data = std::env::var("APPDATA").map(PathBuf::from)?;
    app_data.join(r"LocalLow\10 Chambers Collective\GTFO")
}
