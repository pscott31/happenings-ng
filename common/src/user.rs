use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::person::{DbPerson, NewDbPerson};

#[derive(Debug, Serialize, Deserialize)]
pub enum Credentials {
    OAuth,
    Password { hash: String, salt: String },
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct DbUser {
    #[serde(flatten)]
    pub person: DbPerson,
    pub credentials: Credentials,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct NewDbUser {
    #[serde(flatten)]
    pub person: NewDbPerson,
    pub credentials: Credentials,
}

