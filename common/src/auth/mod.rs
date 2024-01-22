use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::person::{DbPerson, NewDbPerson};

pub mod oauth;
pub mod password;
pub mod session;

#[derive(Debug, Serialize, Deserialize)]
pub enum Credentials {
    OAuth,
    Password { hash: String, salt: String },
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct DbUser {
    #[serde(flatten)]
    person: DbPerson,
    credentials: Credentials,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct NewDbUser {
    #[serde(flatten)]
    person: NewDbPerson,
    credentials: Credentials,
}

