// Revoker / A CLI tool for convenient Twitch OAuth token revoking

// Copyright (C) 2022 / Mykola "TreuKS"

use http::status::StatusCode;
use reqwest;
//use serde::{Deserialize, Serialize};
use crate::modules::verify;

pub async fn revoke_token(
    token: String,
    verjs: verify::VerifiedJson,
) -> Result<String, verify::ACheckError> {
    let rq_client = reqwest::Client::new();

    let revoke_token = rq_client
        .post("https://id.twitch.tv/oauth2/revoke")
        .form(&[("client_id", verjs.client_id), ("token", token)])
        .send()
        .await?;

    match revoke_token.status() {
        StatusCode::OK => Ok(format!(
            "Token for the \"{}\" account has been revoked successfully.",
            verjs.login
        )),
        StatusCode::UNAUTHORIZED => Err(verify::ACheckError::InvalidToken(
            revoke_token
                .json::<verify::TwitchAuthError>()
                .await?
                .message,
        )),
        _ => Err(verify::ACheckError::UnexpectedCode(
            revoke_token.status().as_u16(),
        )),
    }
}
