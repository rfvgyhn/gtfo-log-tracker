use crate::game_data::StoryLog;
use crate::{
    file_contains_log_ids, try_get_log_id, try_get_new_level, INGAME_READ_REGEX, LEVEL_CHANGE_REGEX,
};
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

pub enum GameEvent {
    LogRead(u32),
    LevelSelected(String),
}

enum State {
    NotWatching,
    Watching(RecommendedWatcher, Receiver<notify::Result<Event>>),
    Failed(anyhow::Error),
}

pub fn watch(path: PathBuf, logs: Vec<StoryLog>) -> Subscription<GameEvent> {
    struct Watch;

    subscription::channel(
        std::any::TypeId::of::<Watch>(),
        100,
        |mut output| async move {
            let mut state = State::NotWatching;

            loop {
                match state {
                    State::NotWatching => {
                        state = async_watcher()
                            .and_then(|(mut watcher, rx)| {
                                watcher.watch(&path, RecursiveMode::NonRecursive)?;
                                println!("Watching '{}' for changes", path.display());
                                Ok(State::Watching(watcher, rx))
                            })
                            .unwrap_or_else(|e| State::Failed(e.into()))
                    }
                    State::Watching(ref _watcher, ref mut rx) => match rx.next().await {
                        Some(Ok(Event {
                            kind: EventKind::Create(_) | EventKind::Modify(_),
                            paths,
                            ..
                        })) => {
                            if let Some(path) = paths.first() {
                                let (id, level) = get_latest_data(path, &logs);
                                if let Some(id) = id {
                                    let _ = output.send(GameEvent::LogRead(id)).await;
                                }
                                if let Some(level) = level {
                                    let _ = output.send(GameEvent::LevelSelected(level)).await;
                                }
                            }
                        }
                        Some(Err(e)) => {
                            eprintln!("Failed to read file change - {e:?}");
                        }
                        _ => {}
                    },
                    State::Failed(ref e) => {
                        eprintln!("Unable to watch '{}' for changes - {:?}", path.display(), e)
                    }
                }
            }
        },
    )
}

fn get_latest_data(path: &Path, all_logs: &[StoryLog]) -> (Option<u32>, Option<String>) {
    let mut latest_id: Option<u32> = None;
    let mut latest_level: Option<String> = None;
    let should_check_file = path
        .file_name()
        .map(|s| file_contains_log_ids(&s.to_string_lossy()))
        .unwrap_or(false);

    if should_check_file {
        if let Ok(log_file) = File::open(path) {
            let lines = BufReader::new(log_file).lines().map_while(Result::ok);

            for line in lines {
                let line = line.as_str();
                if let Some(m) = INGAME_READ_REGEX.find(line) {
                    latest_id = try_get_log_id(&m, all_logs);
                }
                if let Some(m) = LEVEL_CHANGE_REGEX.captures(line).and_then(|c| c.get(1)) {
                    latest_level = try_get_new_level(&m);
                }
            }
        }
    }

    (latest_id, latest_level)
}
