use gtfo_log_tracker::game_data::StoryLog;
use gtfo_log_tracker::{game_data, get_read_log_ids_from_log_file, get_read_log_ids_from_play_fab};
use std::env;
use std::fmt::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(Box::from("Path to GTFO game directory required"));
    }

    let gtfo_path = &args[1];
    let only_parse_from_logs = args
        .get(2)
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);

    let all_logs = game_data::load_logs()?;

    let read_log_ids = if only_parse_from_logs {
        get_read_log_ids_from_log_file(gtfo_path, &all_logs)
    } else {
        get_read_log_ids_from_play_fab().await.or_else(|e| {
            println!(
                "Unable to get read log data from PlayFab: {}. Falling back to parsing log files.",
                e
            );
            get_read_log_ids_from_log_file(gtfo_path, &all_logs)
        })
    }?;
    let rows = map_to_rows(&all_logs, &read_log_ids);

    print_table(rows, read_log_ids.len(), all_logs.len());

    Ok(())
}

fn map_to_rows<'a>(
    logs: &'a [StoryLog],
    read_log_ids: &'a [u32],
) -> impl Iterator<Item = Row> + 'a {
    logs.iter().flat_map(|log| {
        log.locations.iter().map(|loc| Row {
            level: format!("R{}{}", loc.rundown, loc.level),
            name: loc.name.to_string(),
            id: log.id,
            read: read_log_ids.contains(&log.id),
            zone: if loc.zones == vec![0] {
                "Outside".to_string()
            } else {
                comma_join(&loc.zones)
            },
        })
    })
}

fn print_table<T>(rows: T, found: usize, total: usize)
where
    T: IntoIterator<Item = Row>,
{
    for row in rows {
        println!(
            "| {0: <5} | {1: <8} | {2: <12} | {3: <15} | {4: <11}",
            row.level,
            row.zone,
            row.name,
            row.id,
            if row.read { "\u{2714}" } else { "" }
        );
    }
    println!("Read: {}/{}", found, total);
}

fn comma_join(nums: &[u16]) -> String {
    nums.iter()
        .enumerate()
        .fold(String::new(), |mut output, (i, num)| {
            if i == 0 {
                let _ = write!(output, "{num}");
            } else {
                let _ = write!(output, ", {num}");
            }
            output
        })
}

struct Row {
    level: String,
    zone: String,
    name: String,
    id: u32,
    read: bool,
}
