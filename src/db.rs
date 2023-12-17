use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub expires_at: DateTime<Utc>,
    pub user: Thing,
}

#[derive(Debug, Serialize)]
pub struct NewPerson {
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Person {
    pub id: Thing,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    #[allow(dead_code)]
    pub id: Thing,
}

