use crate::game_data::StoryLog;
use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use regex::{Match, Regex};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use time::{Date, Month, PrimitiveDateTime, Time};

pub mod game_data;
pub mod iced_gui;
mod play_fab;
#[cfg(target_os = "linux")]
mod steam;

#[derive(Default, Debug)]
pub struct Options {
    pub gtfo_path: PathBuf,
    pub use_playfab: bool,
}

static FILE_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"GTFO\.(\d{4}\.\d{2}\.\d{2}\.\d{2}\.\d{2}\.\d{2})_.*\.txt").unwrap());
pub static INGAME_READ_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([A-Z0-9]{3,4}-[A-Z0-9]{3,6}(?:-[A-Z0-9]{3})?)").unwrap());

pub static LEVEL_CHANGE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"SelectActiveExpedition.*(Local_\d+,\d,\d)").unwrap());

pub async fn get_logs(
    gtfo_path: PathBuf,
    use_playfab: bool,
) -> Result<(Vec<StoryLog>, HashSet<u32>)> {
    let all_logs = game_data::load_logs()?;

    log::info!("Total logs: {}", all_logs.len());

    let read_log_ids = if use_playfab {
        get_read_log_ids_from_play_fab().await.or_else(|e| {
            log::warn!(
                "Unable to read log data from PlayFab: {}. Falling back to parsing log files.",
                e
            );
            get_read_log_ids_from_log_dir(&gtfo_path, &all_logs)
        })
    } else {
        get_read_log_ids_from_log_dir(&gtfo_path, &all_logs)
    }?;

    Ok((all_logs, read_log_ids))
}

async fn get_read_log_ids_from_play_fab() -> Result<HashSet<u32>> {
    log::debug!("Getting log ids from Play Fab");
    log::debug!("Initializing Steam");
    match steamworks::Client::init_app(493520) {
        Ok((steam_client, _)) => {
            log::debug!("Getting steam auth session ticket");
            let (auth_ticket, ticket_bytes) = steam_client.user().authentication_session_ticket();
            let http_client = reqwest::Client::new();
            let user_data = match play_fab::login(&http_client, &ticket_bytes).await {
                Ok(ticket) => play_fab::get_user_data(&http_client, ticket).await,
                Err(e) => Err(e),
            };

            log::debug!("Cancelling steam auth session ticket");
            steam_client
                .user()
                .cancel_authentication_ticket(auth_ticket);

            let ids = user_data.map(|d| d.read_logs.value)?;
            log::info!("{} Read logs: {:?}", ids.len(), ids);
            Ok(HashSet::from_iter(ids))
        }
        Err(e) => Err(anyhow!("Failed to init Steam - {e}")),
    }
}

fn get_read_log_ids_from_log_dir(path: &Path, logs: &[StoryLog]) -> Result<HashSet<u32>> {
    log::debug!("Getting log ids from local user data folder");
    let log_path = fs::read_dir(path)
        .with_context(|| format!("Couldn't read directory '{}'", path.display()))?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let path = e.path();
            parse_file_name(&path).map(|d| (d, path))
        })
        .max_by(|(date1, _), (date2, _)| date1.cmp(date2))
        .map(|(_, path)| path)
        .or_else(|| {
            let player_path = path.join("Player.log");
            player_path.exists().then_some(player_path)
        })
        .ok_or_else(|| {
            anyhow!(
                "Couldn't find any CLIENT/MASTER.txt files or Player.log in '{}'",
                path.display()
            )
        })?;
    let log_file = File::open(&log_path)
        .with_context(|| format!("Couldn't open file '{}'", log_path.display()))?;
    let lines = BufReader::new(log_file).lines().map_while(Result::ok);
    let read_ids = parse_read_ids(lines, logs);

    log::info!("{} Read logs: {:?}", read_ids.len(), read_ids);

    Ok(HashSet::from_iter(read_ids))
}

pub fn try_get_log_id(m: &Match, all_logs: &[StoryLog]) -> Option<u32> {
    let mut name = String::new();
    m.as_str().clone_into(&mut name);
    game_data::get_id_from_name(&name, all_logs)
}

pub fn try_get_new_level(m: &Match) -> Option<String> {
    let mut name = String::new();
    m.as_str().clone_into(&mut name);
    game_data::get_level_from_local(&name)
}

fn parse_read_ids(lines: impl Iterator<Item = String>, logs: &[StoryLog]) -> Vec<u32> {
    static PREVIOUSLY_READ_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"Logs Read: \d+ / \d+ \| IDs: \[(\d+(?:,\s*\d+)*)]\s*$").unwrap());
    let mut read_ids = Vec::new();

    for line in lines {
        if let Some(m) = PREVIOUSLY_READ_REGEX
            .captures(line.as_str())
            .and_then(|c| c.get(1))
        {
            let ids = m
                .as_str()
                .split(',')
                .filter_map(|id| id.trim().parse::<u32>().ok());
            read_ids.extend(ids);
        }

        if let Some(id) = INGAME_READ_REGEX
            .find(line.as_str())
            .and_then(|m| try_get_log_id(&m, logs))
        {
            read_ids.push(id);
        }
    }

    read_ids
}

fn file_contains_log_ids(file_name: &str) -> bool {
    FILE_NAME_REGEX.is_match(file_name)
}

fn parse_file_name(path: &Path) -> Option<PrimitiveDateTime> {
    let file_name = path.file_name()?.to_str().unwrap_or_default();

    FILE_NAME_REGEX.captures(file_name)?.get(1).map(|m| {
        let parts: Vec<&str> = m.as_str().split('.').collect();
        let year = parts[0].parse::<i32>().unwrap();
        let nums: Vec<u8> = parts[1..]
            .iter()
            .map(|s| s.parse::<u8>().unwrap())
            .collect();
        let date =
            Date::from_calendar_date(year, Month::try_from(nums[0]).unwrap(), nums[1]).unwrap();
        let time = Time::from_hms(nums[2], nums[3], nums[4]).unwrap();
        PrimitiveDateTime::new(date, time)
    })
}

#[cfg(test)]
mod tests {
    mod file_contains_log_ids {
        use crate::file_contains_log_ids;

        #[test]
        fn matches_client_file() {
            let result = file_contains_log_ids("GTFO.2023.12.22.00.25.30_NoName_CLIENT.txt");

            assert!(result)
        }

        #[test]
        fn matches_master_file() {
            let result = file_contains_log_ids("GTFO.2023.12.22.00.25.30_NoName_MASTER.txt");

            assert!(result)
        }

        #[test]
        fn matches_single_player_file() {
            let result = file_contains_log_ids("GTFO.2023.12.22.00.25.30_NICKNAME_NETSTATUS.txt");

            assert!(result)
        }
    }
    mod parse_file_name {
        use crate::parse_file_name;
        use std::path::PathBuf;
        use time::Month;

        #[test]
        fn can_parse_if_valid_file_name() {
            let file_name = PathBuf::from("GTFO.2023.12.22.00.25.30_NoName_CLIENT.txt");

            let date = parse_file_name(&file_name).expect("failed to parse date");

            assert_eq!(date.year(), 2023);
            assert_eq!(date.month(), Month::December);
            assert_eq!(date.day(), 22);
            assert_eq!(date.hour(), 0);
            assert_eq!(date.minute(), 25);
            assert_eq!(date.second(), 30);
        }
    }
}
