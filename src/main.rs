#![windows_subsystem = "windows"]

use anyhow::{anyhow, Context, Result};
use gtfo_log_tracker::iced_gui::GtfoLogTracker;
use gtfo_log_tracker::{game_data, Options};
use iced::Application;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;
use std::path::PathBuf;
use std::{env, fs};

fn main() -> Result<()> {
    init_logger()?;

    let args: Vec<String> = env::args().collect();
    log_runtime_info(&args);

    let options = get_options(args)?;
    log::debug!("{options:?}");

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
            .with_context(|| "Couldn't get GTFO user data path")?,
        use_playfab: args
            .iter()
            .find(|s| *s == "--playfab")
            .map(|_| true)
            .unwrap_or(false),
    })
}

fn init_logger() -> Result<()> {
    #[cfg(target_os = "linux")]
    let log_dir = dirs::state_dir();
    #[cfg(target_os = "windows")]
    let log_dir = dirs::data_local_dir();

    let log_path = log_dir
        .ok_or_else(|| anyhow!("Unable to get log directory"))?
        .join("gtfo-log-tracker")
        .join("log.txt");

    fs::create_dir_all(log_path.parent().expect("Log path must include file name"))
        .with_context(|| format!("Couldn't create log dir for path '{}'", log_path.display()))?;

    let term_config = ConfigBuilder::default()
        .add_filter_allow_str(env!("CARGO_CRATE_NAME"))
        .build();

    let file_config = ConfigBuilder::default()
        .set_thread_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        .add_filter_allow_str(env!("CARGO_CRATE_NAME"))
        .build();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            term_config,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            file_config,
            File::create(log_path).with_context(|| "Couldn't create log file")?,
        ),
    ])
    .map_err(|e| e.into())
}

fn log_runtime_info(args: &[String]) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    log::info!("GTFO Log Tracker - v{VERSION}");
    log::debug!(
        r"
    Args: {}
    OS: {}",
        args.join(" "),
        env::consts::OS
    );
}
