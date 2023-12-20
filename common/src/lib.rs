use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub url: Url,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct OAuthReturnResponse {
    pub session_id: String,
    pub user_id: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[allow(dead_code)]
pub struct UserInfoReponse {
    pub id: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NewUser {
    pub given_name: String,
    pub family_name: String,
    // pub picture: String,
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum CreateUserResponse {
    Error(ErrorResponse),
    Ok(),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct EmailPassword {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Session {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Email {
    pub email: String,
}

