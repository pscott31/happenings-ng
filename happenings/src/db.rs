use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub expires_at: DateTime<Utc>,
    pub user: Thing,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Credentials {
    OAuth,
    Password { hash: String, salt: String },
}
#[derive(Debug, Serialize)]
pub struct NewPerson {
    pub given_name: String,
    pub family_name: String,
    pub picture: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub credentials: Credentials,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Person {
    pub id: Thing,
    pub given_name: String,
    pub family_name: String,
    pub picture: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub credentials: Credentials,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    #[allow(dead_code)]
    pub id: Thing,
}

