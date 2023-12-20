use std::collections::HashMap;

use crate::config;
use crate::AppState;

use crate::db::*;
use crate::error_handling::*;
use anyhow::Context;
use anyhow::{anyhow, Result};
use axum::{extract::{Host, Query, State}, response::{IntoResponse, Json}};
use oauth2::CsrfToken;
use oauth2::PkceCodeChallenge;
use oauth2::{basic::BasicClient, reqwest::async_http_client, AuthorizationCode, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct OAuth2State {
    pkce_code_verifier: String,
    return_url: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenIDUserInfo {
    given_name: String,
    family_name: String,
    picture: String,
    email: String,
    email_verified: bool,
    sub: String,
}

fn build_oauth_client(cfg: &config::OAuthProvider, hostname: String) -> BasicClient {
    let scheme = if hostname.starts_with("localhost") {
        "http"
    } else {
        "https"
    };
    let redirect_url = format!("{}://{}/oauth_return", scheme, hostname);
    BasicClient::new(
        cfg.client_id.clone(),
        Some(cfg.client_secret.clone()),
        cfg.auth_url.clone(),
        Some(cfg.token_url.clone()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

fn first_oauth_config(state: &AppState) -> Result<config::OAuthProvider, AppError> {
    state
        .config
        .login
        .oauth_providers
        .first()
        .cloned()
        .ok_or(anyhow!("no oauth providers configured").into())
}

pub async fn login_handler(
    State(state): State<AppState>,
    Host(hostname): Host,
) -> Result<impl IntoResponse, AppError> {
    let cfg = first_oauth_config(&state)?;
    let client = build_oauth_client(&cfg, hostname);

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid profile email".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    let _record: Option<Record> = state
        .db
        .create(("oauth2_state", csrf_state.secret().clone()))
        .content(OAuth2State {
            pkce_code_verifier: pkce_code_verifier.secret().clone(),
            return_url: "/".to_string(),
        })
        .await?
        .ok_or(anyhow!("unable to create oauth state db record"))?;

    Ok(Json(common::LoginResponse { url: authorize_url }))
}

pub async fn oauth_return(
    Query(mut params): Query<HashMap<String, String>>,
    State(app_state): State<AppState>,
    Host(hostname): Host,
) -> Result<impl IntoResponse, AppError> {
    let state = CsrfToken::new(params.remove("state").context("oauth missing state")?);
    let code = AuthorizationCode::new(params.remove("code").context("oauth missing code")?);

    let oauth_state: OAuth2State = app_state
        .db
        .delete(("oauth2_state", state.secret()))
        .await?
        .context("unknown oauth csrf id")?;

    let cfg = first_oauth_config(&app_state)?;
    let client = build_oauth_client(&cfg, hostname);

    let token_response = client
        .exchange_code(code)
        .set_pkce_verifier(PkceCodeVerifier::new(oauth_state.pkce_code_verifier))
        .request_async(async_http_client)
        .await?;

    let access_token = token_response.access_token().secret();

    let url =
        "https://www.googleapis.com/oauth2/v3/userinfo?oauth_token=".to_owned() + access_token;
    let user_info = reqwest::get(url)
        .await
        .map_err(|_| anyhow!("OAuth: reqwest failed to query userinfo"))?
        .json::<OpenIDUserInfo>()
        .await
        .map_err(|_| anyhow!("OAuth: reqwest received invalid userinfo"))?;
    dbg!(&user_info);

    if !user_info.email_verified {
        Err(anyhow!("email address not verified"))?;
    }

    let mut result = app_state
        .db
        .query("SELECT * FROM user where email=$email;")
        .bind(("email", &user_info.email))
        .await?;

    let mut people: Vec<Person> = result.take(1)?;

    let person = match people.pop() {
        Some(person) => person,
        None => app_state
            .db
            .create("user")
            .content(NewPerson {
                given_name: user_info.given_name,
                family_name: user_info.family_name,
                picture: Some(user_info.picture),
                phone: None,
                email: user_info.email,
                credentials: Credentials::OAuth,
            })
            .await?
            .pop()
            .ok_or(anyhow!("failed to create new user"))?,
    };

    // // Create the session
    let session_id = super::create_session(app_state.db, person.id.id.to_string()).await?;
    Ok(Json(common::Session { id: session_id }))
}

