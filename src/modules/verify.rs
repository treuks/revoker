// Revoker / A CLI tool for convenient Twitch OAuth token revoking

// Copyright (C) 2022 / Mykola "TreuKS"


use http::status::StatusCode;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ParserError {
    TooLong,
    TooShort,
    Invalid,
    // InvalidCharacters,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ParserError::TooLong => "Token is too long",
            ParserError::TooShort => "Token is too short",
            ParserError::Invalid => "Token is invalid",
        })
    }
}

impl std::error::Error for ParserError {}

pub fn parse_pos_token(possible_token: String) -> Result<String, ParserError> {
    if possible_token.len() == 36 && possible_token.starts_with("oauth:") {
        return Ok(possible_token.trim_start_matches("oauth:").to_string());
    }

    match possible_token.len().cmp(&30) {
        std::cmp::Ordering::Greater => Err(ParserError::TooLong),
        std::cmp::Ordering::Less => Err(ParserError::TooShort),
        std::cmp::Ordering::Equal => Ok(possible_token),
    }
}

#[derive(Debug)]
pub enum ACheckError {
    InvalidToken(String),
    DeserializeError(serde_json::Error),
    ReqwestError(reqwest::Error),
    UnexpectedCode(u16),
    NotFound(u16),
}

impl From<reqwest::Error> for ACheckError {
    fn from(e: reqwest::Error) -> Self {
        Self::ReqwestError(e)
    }
}

impl From<serde_json::Error> for ACheckError {
    fn from(e: serde_json::Error) -> Self {
        Self::DeserializeError(e)
    }
}

impl std::fmt::Display for ACheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            ACheckError::InvalidToken(e) => format!("Error: {}", e),
            ACheckError::DeserializeError(e) => format!("Deserialization error: {}", e),
            ACheckError::ReqwestError(e) => format!("Reqwest error: {}", e),
            ACheckError::UnexpectedCode(e) => format!("Unexpected status code: {}", e),
            ACheckError::NotFound(e) => format!("Error code {}, Is Twitch down?", e),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchAuthError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for TwitchAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifiedJson {
    pub client_id: String,
    pub login: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub expires_in: u64,
}

pub async fn advanced_token_check(more_possible_token: &str) -> Result<VerifiedJson, ACheckError> {
    let req_client = reqwest::Client::new();

    let verification_res = req_client
        .get("https://id.twitch.tv/oauth2/validate")
        .bearer_auth(&more_possible_token)
        .send()
        .await?;

    match verification_res.status() {
        StatusCode::OK => Ok(verification_res.json().await?),
        StatusCode::UNAUTHORIZED => Err(ACheckError::InvalidToken(
            verification_res.json::<TwitchAuthError>().await?.message,
        )),
        StatusCode::NOT_FOUND => Err(ACheckError::NotFound(verification_res.status().as_u16())),
        _ => Err(ACheckError::UnexpectedCode(
            verification_res.status().as_u16(),
        )),
    }
}
