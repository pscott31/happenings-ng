cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    pub id: Thing,
}

}}

