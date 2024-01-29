#[cfg(not(target_arch = "wasm32"))]
use crate::schema::Schema;
use leptos::IntoAttribute;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Id<T>(String, std::marker::PhantomData<T>);

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}

impl<T> From<String> for Id<T> {
    fn from(s: String) -> Self { Self(s, std::marker::PhantomData) }
}

impl<T> From<&str> for Id<T> {
    fn from(s: &str) -> Self { Self(s.to_string(), std::marker::PhantomData) }
}

impl<T> From<Id<T>> for String {
    fn from(b: Id<T>) -> Self { b.0 }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Schema> From<&Id<T>> for surrealdb::sql::Thing {
    fn from(b: &Id<T>) -> Self {
        Self {
            tb: T::TABLE.to_string(),
            id: b.0.clone().into(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Schema> From<Id<T>> for surrealdb::sql::Thing {
    fn from(b: Id<T>) -> Self {
        Self {
            tb: T::TABLE.to_string(),
            id: b.0.clone().into(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> From<surrealdb::sql::Thing> for Id<T> {
    fn from(b: surrealdb::sql::Thing) -> Self { Self(b.id.to_string(), std::marker::PhantomData) }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Schema, R> surrealdb::opt::IntoResource<Option<R>> for &Id<T> {
    fn into_resource(self) -> surrealdb::Result<surrealdb::opt::Resource> {
        Ok(surrealdb::opt::Resource::RecordId(self.into()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Schema, R> surrealdb::opt::IntoResource<Option<R>> for Id<T> {
    fn into_resource(self) -> surrealdb::Result<surrealdb::opt::Resource> {
        Ok(surrealdb::opt::Resource::RecordId(self.into()))
    }
}

impl<T> IntoAttribute for Id<T> {
    fn into_attribute(self) -> leptos::Attribute { self.0.into_attribute() }
    fn into_attribute_boxed(self: Box<Self>) -> leptos::Attribute { self.into_attribute() }
}

