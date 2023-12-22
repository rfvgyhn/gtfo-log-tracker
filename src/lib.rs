use crate::game_data::StoryLog;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use time::{Date, Month, PrimitiveDateTime, Time};

pub mod game_data;
mod play_fab;

type Error = Box<dyn std::error::Error>;

pub async fn get_read_log_ids_from_play_fab() -> Result<Vec<u32>, Error> {
    match steamworks::Client::init_app(493520) {
        Ok((steam_client, _)) => {
            let (auth_ticket, ticket_bytes) = steam_client.user().authentication_session_ticket();
            let http_client = reqwest::Client::new();
            let user_data = match play_fab::login(&http_client, &ticket_bytes).await {
                Ok(ticket) => play_fab::get_user_data(&http_client, ticket).await,
                Err(e) => Err(e),
            };

            steam_client
                .user()
                .cancel_authentication_ticket(auth_ticket);

            user_data.map(|d| d.read_logs.value)
        }
        Err(e) => {
            eprintln!("Couldn't init steam");
            Err(Box::from(e))
        }
    }
}

pub fn get_read_log_ids_from_log_file(path: &str, logs: &[StoryLog]) -> Result<Vec<u32>, Error> {
    let log_path = fs::read_dir(path)?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let path = e.path();
            parse_file_name(&path).map(|d| (d, path))
        })
        .max_by(|(date1, _), (date2, _)| date1.cmp(date2))
        .map(|(_, path)| path)
        .or_else(|| {
            let player_path = Path::new(path).join("Player.log");
            player_path.exists().then_some(player_path)
        })
        .ok_or_else(|| {
            format!("Couldn't find any CLIENT/MASTER.txt files or Player.log in '{path}'")
        })?;
    let previously_read_regex =
        Regex::new(r"Logs Read: \d+ / \d+ \| IDs: \[(\d+(?:,\s*\d+)*)]\s*$")?;
    let ingame_read_regex = Regex::new(r"([A-Z0-9]{3,4}-[A-Z0-9]{3,6}(?:-[A-Z0-9]{3})?)")?;
    let log_file = File::open(log_path)?;
    let mut read_ids = HashSet::new();

    for line in BufReader::new(log_file).lines().map_while(Result::ok) {
        if let Some(captures) = previously_read_regex.captures(line.as_str()) {
            if let Some(m) = captures.get(1) {
                let ids = m
                    .as_str()
                    .split(',')
                    .filter_map(|id| id.trim().parse::<u32>().ok());
                read_ids.extend(ids);
            }
        }

        if let Some(m) = ingame_read_regex.find(line.as_str()) {
            let mut name = String::new();
            m.as_str().clone_into(&mut name);
            if let Some(id) = game_data::get_id_from_name(&name, logs) {
                read_ids.insert(id);
            }
        }
    }

    Ok(read_ids.into_iter().collect())
}

fn parse_file_name(path: &Path) -> Option<PrimitiveDateTime> {
    static FILE_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"GTFO\.(\d{4}\.\d{2}\.\d{2}\.\d{2}\.\d{2}\.\d{2})_[A-Za-z0-9_-]+(?:CLIENT|MASTER)\.txt").unwrap()
    });
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
