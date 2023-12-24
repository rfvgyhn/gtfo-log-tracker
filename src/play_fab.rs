use anyhow::{anyhow, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter, Write};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", untagged)]
enum ApiResponse<T> {
    Success { data: T },
    Error(ErrorDetails),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ErrorDetails {
    code: u16,
    error: String,
    error_code: String,
    error_details: String,
    error_message: String,
    status: String,
}

impl Display for ErrorDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HTTP: {} - {} {{ error: {}, error_code: {}, error_details: {}, error_message: {} }}",
            self.code,
            self.status,
            self.error,
            self.error_code,
            self.error_details,
            self.error_message
        )
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginRequest {
    title_id: String,
    create_account: bool,
    steam_ticket: String,
    ticket_is_service_specific: bool,
}

impl LoginRequest {
    pub fn new(title_id: &str, steam_ticket: String) -> LoginRequest {
        LoginRequest {
            title_id: title_id.to_string(),
            create_account: false,
            steam_ticket,
            ticket_is_service_specific: false,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct LoginResponse {
    pub session_ticket: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct UserDataResponse {
    data: UserData,
}

#[derive(Deserialize, Debug)]
pub struct UserData {
    #[serde(rename = "readlogs")]
    pub read_logs: ReadLogs,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ReadLogs {
    #[serde(deserialize_with = "from_id_str")]
    pub value: Vec<u32>,
}

pub struct SessionTicket(String);

const GTFO_TITLE_ID: &str = "8f9";

pub async fn login(http_client: &reqwest::Client, steam_ticket: &[u8]) -> Result<SessionTicket> {
    let req_body = LoginRequest::new(GTFO_TITLE_ID, to_hex(steam_ticket));
    let res = http_client
        .post(format!(
            "https://{}.playfabapi.com/Client/LoginWithSteam",
            GTFO_TITLE_ID
        ))
        .json(&req_body)
        .send()
        .await?
        .json::<ApiResponse<LoginResponse>>()
        .await?;

    match res {
        ApiResponse::Success { data } => Ok(SessionTicket(data.session_ticket)),
        ApiResponse::Error(e) => Err(anyhow!("{e}")),
    }
}

pub async fn get_user_data(
    http_client: &reqwest::Client,
    session_ticket: SessionTicket,
) -> Result<UserData> {
    let res = http_client
        .post(format!(
            "https://{}.playfabapi.com/Client/GetUserData",
            GTFO_TITLE_ID
        ))
        .header("X-Authorization", session_ticket.0)
        .header("Content-Type", "application/json")
        .send()
        .await?
        .json::<ApiResponse<UserDataResponse>>()
        .await?;

    match res {
        ApiResponse::Success { data } => Ok(data.data),
        ApiResponse::Error(e) => Err(anyhow!("{e}")),
    }
}

fn from_id_str<'de, D>(deserializer: D) -> Result<Vec<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(parse_ids(&s))
}

fn parse_ids(ids_str: &str) -> Vec<u32> {
    let ids_str = if ids_str.starts_with('[') && ids_str.ends_with(']') {
        &ids_str[1..ids_str.len() - 1]
    } else {
        ids_str
    };

    ids_str
        .split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .collect()
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut output, b| {
        let _ = write!(output, "{b:02X}");
        output
    })
}

#[cfg(test)]
mod tests {
    mod parse_ids {
        use crate::play_fab::parse_ids;

        #[test]
        fn can_parse_empty_string() {
            let ids = parse_ids("");

            assert_eq!(ids.len(), 0);
        }

        #[test]
        fn can_parse_empty_array() {
            let ids = parse_ids("[]");

            assert_eq!(ids.len(), 0);
        }

        #[test]
        fn can_parse_array() {
            let ids = parse_ids("[12345]");

            assert_eq!(ids.len(), 1);
            assert_eq!(ids[0], 12345);
        }

        #[test]
        fn can_parse_single_item() {
            let ids = parse_ids("12345");

            assert_eq!(ids.len(), 1);
            assert_eq!(ids[0], 12345);
        }

        #[test]
        fn can_parse_comma_seperated() {
            let ids = parse_ids("12345, 54321");

            assert_eq!(ids.len(), 2);
            assert_eq!(ids[0], 12345);
            assert_eq!(ids[1], 54321);
        }

        #[test]
        fn ignores_invalid_numbers() {
            let ids = parse_ids("12345, abc");

            assert_eq!(ids.len(), 1);
            assert_eq!(ids[0], 12345);
        }

        #[test]
        fn can_handle_whitespace() {
            let ids = parse_ids("\r\n  12345,\r\n  54321\r\n");

            assert_eq!(ids.len(), 2);
            assert_eq!(ids[0], 12345);
            assert_eq!(ids[1], 54321);
        }
    }
}
