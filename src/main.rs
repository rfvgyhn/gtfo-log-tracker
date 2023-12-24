use anyhow::{bail, Result};
use gtfo_log_tracker::gui::GtfoLogTracker;
use gtfo_log_tracker::Options;
use iced::Application;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        bail!("Path to GTFO game directory required");
    }

    env::set_var("MANGOHUD", "0");

    let settings = GtfoLogTracker::settings(Options {
        gtfo_path: args[1].clone().into(),
        only_parse_from_logs: args
            .get(2)
            .map(|s| s.parse().unwrap_or(false))
            .unwrap_or(false),
    });
    GtfoLogTracker::run(settings)?;
    Ok(())
}
