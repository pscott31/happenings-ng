use serde::{Deserialize, Serialize};

use crate::generic_id::Id;

pub type RoleId = Id<Role>;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Role {
    pub id: RoleId,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DbRole {
    pub id: surrealdb::sql::Thing,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<DbRole> for Role {
    fn from(item: DbRole) -> Self { Self { id: item.id.into() } }
}

