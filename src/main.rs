use anyhow::{anyhow, Result};
use gtfo_log_tracker::iced_gui::GtfoLogTracker;
use gtfo_log_tracker::{game_data, Options};
use iced::Application;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    #[cfg(target_os = "linux")]
    env::set_var("MANGOHUD", "0");

    let settings = GtfoLogTracker::settings(Options {
        gtfo_path: args
            .get(2)
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!(""))
            .or_else(|_| game_data::find_user_data_path())
            .map_err(|e| anyhow!("Couldn't get GTFO user data path: {e:?}"))?,
        only_parse_from_logs: args
            .get(1)
            .map(|s| s.parse().unwrap_or(false))
            .unwrap_or(false),
    });
    GtfoLogTracker::run(settings)?;
    Ok(())
}
