use leptos::IntoAttribute;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub trait TableName {
    const TABLE_NAME: &'static str;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct GenericId<T>(String, std::marker::PhantomData<T>);

impl<T> Display for GenericId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}

impl<T> From<String> for GenericId<T> {
    fn from(s: String) -> Self { Self(s, std::marker::PhantomData) }
}

impl<T> From<GenericId<T>> for String {
    fn from(b: GenericId<T>) -> Self { b.0 }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: TableName> From<&GenericId<T>> for surrealdb::sql::Thing {
    fn from(b: &GenericId<T>) -> Self {
        Self {
            tb: T::TABLE_NAME.to_string(),
            id: b.0.clone().into(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: TableName> From<GenericId<T>> for surrealdb::sql::Thing {
    fn from(b: GenericId<T>) -> Self {
        Self {
            tb: T::TABLE_NAME.to_string(),
            id: b.0.clone().into(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> From<surrealdb::sql::Thing> for GenericId<T> {
    fn from(b: surrealdb::sql::Thing) -> Self { Self(b.id.to_string(), std::marker::PhantomData) }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: TableName, R> surrealdb::opt::IntoResource<Option<R>> for &GenericId<T> {
    fn into_resource(self) -> surrealdb::Result<surrealdb::opt::Resource> {
        Ok(surrealdb::opt::Resource::RecordId(self.into()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: TableName, R> surrealdb::opt::IntoResource<Option<R>> for GenericId<T> {
    fn into_resource(self) -> surrealdb::Result<surrealdb::opt::Resource> {
        Ok(surrealdb::opt::Resource::RecordId(self.into()))
    }
}

impl<T> IntoAttribute for GenericId<T> {
    fn into_attribute(self) -> leptos::Attribute { self.0.into_attribute() }
    fn into_attribute_boxed(self: Box<Self>) -> leptos::Attribute { self.into_attribute() }
}

