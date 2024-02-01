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

enum RevError {
    AttoError(attohttpc::Error),
    DeserizationError(miniserde::Error),
    Length(LengthError),
}

impl From<attohttpc::Error> for RevError {
    fn from(value: attohttpc::Error) -> Self {
        Self::AttoError(value)
    }
}

impl From<miniserde::Error> for RevError {
    fn from(value: miniserde::Error) -> Self {
        Self::DeserizationError(value)
    }
}

impl From<LengthError> for RevError {
    fn from(value: LengthError) -> Self {
        match value {
            LengthError::TooLong => Self::Length(LengthError::TooLong),
            LengthError::TooShort => Self::Length(LengthError::TooShort),
        }
    }
}

fn print_version() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!(r#"

    Revoker v{VERSION} - a lightweight Twitch OAuth token revoking tool.
        

    Usage:

        * revoker `<token>`
        
        * revoker `oauth:<>`
        
        * revoker `--help` or `-h` to print this dialogue

    "#
    );
}

fn main() {

    let args = env::args().collect::<Vec<_>>();

    let asd_args = &args;
    if asd_args.iter().any(|el| el == "--help" || el == "-h") {
        print_version();
        process::exit(0);
    }

    let oauth = match args.get(1) {
        Some(el) => el,
        None => {
            print_version();
            process::exit(0);
        }
    };

    match revoke_oauth(oauth.as_str()) {
        Ok(ret) => {
            println!("{ret}");
        }
        Err(RevError::AttoError(err)) => {
            eprintln!("Request error: {err}");
            process::exit(1);
        }
        Err(RevError::DeserizationError(err)) => {
            eprintln!("Error during deserialization: {err}");
            process::exit(1);
        }
        Err(RevError::Length(LengthError::TooLong)) => {
            eprintln!("Error: token too long");
            process::exit(1);
        }
        Err(RevError::Length(LengthError::TooShort)) => {
            eprintln!("Error: token too short");
            process::exit(1);
        }
    } 
}

fn revoke_oauth(arg_oauth: &str) -> Result<String, RevError> {
    let normal_oauth = normalize_oauth(arg_oauth);
    let oauth = check_oauth_len(normal_oauth)?;
    let response = attohttpc::get("https://id.twitch.tv/oauth2/validate")
        .bearer_auth(oauth)
        .send()?;

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
            let ok_response = response.text()?;
            json::from_str(&ok_response).expect("Twitch did a fucky wucky and changed the json response for the verification endpoint, please create an issue")
        }
        _ => {
            unreachable!(
                "Twitch have sent an undocumented status code ({}), skill issue on their part, ngl",
                response.status()
            );
        }
    };

    let revoking_response = attohttpc::post("https://id.twitch.tv/oauth2/revoke")
        .param("client_id", &resp_json.client_id)
        .param("token", oauth)
        .send()?;

    match revoking_response.status() {
        StatusCode::BAD_REQUEST => {
            eprintln!("Error: token is invalid");
            process::exit(1);
        }
        StatusCode::NOT_FOUND => {
            // We check for the message because we don't know if the endpoint itself is not found or if its the token
            let xdd: TwitchAuthError = json::from_str(&revoking_response.text()?).expect(
                "Revoking endpoint response is different. The endpoint changed. Create an issue.",
            );
            eprintln!("Error: {}", xdd.message);
            process::exit(1);
        }

        StatusCode::OK => Ok(format!(
            "Token for user \"{}\" has been revoked succesfully.",
            &resp_json.login
        )),

        _ => {
            unreachable!(
                "Twitch have sent an undocumented status code ({}), skill issue on their part, ngl",
                revoking_response.status()
            );
        }
    }
}
