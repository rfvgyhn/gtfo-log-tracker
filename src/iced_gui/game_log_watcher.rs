use crate::game_data::StoryLog;
use crate::{file_contains_log_ids, parse_read_ids};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use iced::{subscription, Subscription};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);
    let watcher = notify::recommended_watcher(move |res| {
        futures::executor::block_on(async {
            tx.send(res).await.unwrap();
        })
    })?;

    Ok((watcher, rx))
}

pub fn watch(path: PathBuf, logs: Vec<StoryLog>) -> Subscription<u32> {
    struct Watch;

    async_watcher()
        .and_then(|(mut watcher, rx)| {
            watcher.watch(&path, RecursiveMode::NonRecursive)?;
            println!("Watching '{}' for changes", path.display());
            Ok((watcher, rx))
        })
        .map(|(watcher, mut rx)| {
            subscription::channel(
                std::any::TypeId::of::<Watch>(),
                100,
                |mut output| async move {
                    let _w = watcher; // ensure watcher is captured
                    loop {
                        match rx.next().await {
                            Some(Ok(Event {
                                kind: EventKind::Create(_) | EventKind::Modify(_),
                                paths,
                                ..
                            })) => {
                                println!("{:?}", paths.first());
                                if let Some(path) = paths.first() {
                                    if let Some(id) = get_newest_id(path, &logs) {
                                        let _ = output.send(id).await;
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                eprintln!("Failed to read file change - {e:?}");
                            }
                            _ => {}
                        }
                    }
                },
            )
        })
        .unwrap_or_else(|e| {
            eprintln!("Unable to watch '{}' for changes - {:?}", path.display(), e);
            Subscription::none()
        })
}

fn get_newest_id(path: &Path, all_logs: &[StoryLog]) -> Option<u32> {
    let should_check_file = path
        .file_name()
        .map(|s| file_contains_log_ids(&s.to_string_lossy()))
        .unwrap_or(false);
    if should_check_file {
        if let Ok(log_file) = File::open(path) {
            let lines = BufReader::new(log_file)
                .lines()
                .map_while(std::result::Result::ok);
            return parse_read_ids(lines, all_logs).last().copied();
        }
    }

    None
}
