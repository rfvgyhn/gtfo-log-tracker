use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const GTFO_APP_ID: u32 = 493520;

pub fn find_proton_app_data_path() -> Option<PathBuf> {
    let home = env::var_os("HOME").map(PathBuf::from)?;

    [".steam/steam/steamapps/", ".local/share/Steam/steamapps/"]
        .iter()
        .find_map(|p| {
            let path = home.join(p).join("libraryfolders.vdf");
            let file = File::open(path).ok()?;
            let lines = BufReader::new(file).lines().map_while(Result::ok);

            parse_library_path(GTFO_APP_ID, lines).map(|p| {
                p.join(format!(
                    "steamapps/compatdata/{GTFO_APP_ID}/pfx/drive_c/users/steamuser/AppData"
                ))
            })
        })
}

fn parse_library_path(app_id: u32, lines: impl Iterator<Item = String>) -> Option<PathBuf> {
    let id = format!("\"{}", app_id);
    let mut last_seen_path: Option<PathBuf> = None;

    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("\"path\"") {
            last_seen_path = line
                .splitn(4, '"')
                .last()
                .map(|s| PathBuf::from(&s[..s.len() - 1]))
        } else if trimmed.starts_with(&id) {
            return last_seen_path;
        }
    }

    None
}

#[cfg(test)]
mod test {
    mod parse_library_path {
        use crate::steam::{parse_library_path, GTFO_APP_ID};

        #[test]
        fn finds_path_if_one_game_library() {
            let path = "/a/path";
            let vdf = format!(
                r#"
                "libraryfolders"
                {{
                    {}                
                }}"#,
                generate_library(path, 0, GTFO_APP_ID)
            );
            let lines = vdf.lines().map(String::from);

            let result = parse_library_path(GTFO_APP_ID, lines);

            assert!(result.is_some());
            assert_eq!(result.unwrap().to_string_lossy(), path);
        }

        #[test]
        fn finds_path_if_multiple_game_libraries() {
            let expected_path = "/a/path";
            let vdf = format!(
                r#"
                "libraryfolders"
                {{
                    {}
                    {}
                }}"#,
                generate_library("/wrong/path", 0, 0),
                generate_library(expected_path, 1, GTFO_APP_ID),
            );
            let lines = vdf.lines().map(String::from);

            let result = parse_library_path(GTFO_APP_ID, lines);

            assert!(result.is_some());
            assert_eq!(result.unwrap().to_string_lossy(), expected_path);
        }

        fn generate_library(path: &str, id: u8, app_id: u32) -> String {
            format!(
                r#"
            "{id}"
            {{
                "path"		"{path}"
                "label"		""
                "contentid"		"12345"
                "totalsize"		"0"
                "update_clean_bytes_tally"		"43076708540"
                "time_last_update_corruption"		"0"
                "apps"
                {{
                    "12345"		"450962885"
                    "{app_id}"		"5367440856"
                }}
            }}"#
            )
        }
    }
}
