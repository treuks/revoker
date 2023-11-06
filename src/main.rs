use attohttpc::{self, StatusCode};
use miniserde::{json, Deserialize, Serialize};
use std::{cmp::Ordering, env, process};

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifiedJson {
    pub client_id: String,
    pub login: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub expires_in: i32, // In case it might be -1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchAuthError {
    pub status: u16,
    pub message: String,
}

fn normalize_oauth(potential_oauth: &str) -> &str {
    if potential_oauth.starts_with("oauth:") {
        potential_oauth.strip_prefix("oauth:").unwrap()
    } else {
        potential_oauth
    }
}

#[derive(Debug)]
pub enum LengthError {
    TooLong,
    TooShort,
}

fn check_oauth_len(oauth: &str) -> Result<&str, LengthError> {
    match oauth.len().cmp(&30) {
        Ordering::Greater => Err(LengthError::TooLong),
        Ordering::Less => Err(LengthError::TooShort),
        Ordering::Equal => Ok(oauth),
    }
}

fn main() {
    match env::args().nth(1) {
        Some(arg) => hate_nesting(&arg),
        None => {
            eprintln!("Error: Empty token field \n\n Usage: ./revoker <oauth>");
            process::exit(1);
        }
    }
}

fn hate_nesting(arg_oauth: &str) {
    let normal_oauth = normalize_oauth(arg_oauth);
    let oauth = match check_oauth_len(normal_oauth) {
        Ok(val) => val,
        Err(LengthError::TooShort) => {
            eprintln!("Error: Token is too short");
            process::exit(1);
        }
        Err(LengthError::TooLong) => {
            eprintln!("Error: Token is too long");
            process::exit(1);
        }
    };
    let response = attohttpc::get("https://id.twitch.tv/oauth2/validate")
        .bearer_auth(oauth)
        .send()
        .map_err(|err| {
            eprintln!("Error while sending request: {err}");
            process::exit(1);
        })
        .unwrap();

    let resp_json: VerifiedJson = match response.status() {
        StatusCode::UNAUTHORIZED => {
            eprintln!("Error: invalid access token");
            process::exit(1);
        }
        StatusCode::NOT_FOUND => {
            eprintln!("Error: Twitch might have changed the validation endpoint. Create an issue.");
            process::exit(1);
        }
        StatusCode::OK => {
            let ok_response = response.text_utf8().unwrap();
            json::from_str(&ok_response).expect("Twitch did a fucky wucky and changed the json response for the verification endpoint, please create an issue")
        }
        _ => {
            unreachable!(
                "Twitch have sent an undocumented status code, skill issue on their part, ngl"
            );
        }
    };

    let revoking_response = attohttpc::post("https://id.twitch.tv/oauth2/revoke")
        .param("client_id", &resp_json.client_id)
        .param("token", oauth)
        .send()
        .map_err(|err| {
            eprintln!("Error while sending request: {err}");
            process::exit(1);
        })
        .unwrap();

    match revoking_response.status() {
        StatusCode::BAD_REQUEST => {
            eprintln!("Error: token is invalid");
            process::exit(1);
        }
        StatusCode::NOT_FOUND => {
            // We check for the message because we don't know if the endpoint itself is not found or if its the token
            let xdd: TwitchAuthError = json::from_str(&revoking_response.text().unwrap()).expect(
                "Revoking endpoint response is different. The endpoint changed. Create an issue.",
            );
            eprintln!("Error: {}", xdd.message);
            process::exit(1);
        }

        StatusCode::OK => {
            println!(
                "Token for user \"{}\" has been revoked succesfully.",
                &resp_json.login
            )
        }

        _ => {
            unreachable!(
                "Twitch have sent an undocumented status code, skill issue on their part, ngl"
            );
        }
    }
}
