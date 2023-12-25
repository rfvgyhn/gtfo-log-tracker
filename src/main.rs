use anyhow::{anyhow, Result};
use gtfo_log_tracker::iced_gui::GtfoLogTracker;
use gtfo_log_tracker::{game_data, Options};
use iced::Application;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let options = get_options(args)?;

    #[cfg(target_os = "linux")]
    env::set_var("MANGOHUD", "0");

    GtfoLogTracker::run(GtfoLogTracker::settings(options))?;
    Ok(())
}

fn get_options(args: Vec<String>) -> Result<Options> {
    Ok(Options {
        gtfo_path: args
            .iter()
            .position(|s| s == "--data-path")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!(""))
            .or_else(|_| game_data::find_user_data_path())
            .map_err(|e| anyhow!("Couldn't get GTFO user data path: {e:?}"))?,
        use_playfab: args
            .iter()
            .find(|s| *s == "--playfab")
            .map(|_| true)
            .unwrap_or(false),
    })
}
