use async_trait::async_trait;
use common::{role::RoleId, user};
use leptos::*;
use leptos_struct_table::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
struct BulmaTableClasses;

impl TableClassesProvider for BulmaTableClasses {
    fn new() -> Self { Self }
    fn table(&self, classes: &str) -> String { format!("table {}", classes) }
}
#[derive(TableComponent, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[table(classes_provider = "BulmaTableClasses")]
pub struct User {
    #[table(key, skip)]
    pub id: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
    pub phone: Option<String>,
    #[table(renderer = "RoleListRenderer")]
    pub roles: Vec<RoleId>,
}

impl From<user::User> for User {
    fn from(u: user::User) -> Self {
        User {
            id: u.person.id.to_string(),
            given_name: u.person.given_name,
            family_name: u.person.family_name,
            email: u.person.email,
            phone: u.person.phone,
            roles: u.roles,
            // roles: u.roles.into_iter().map(|role| role.to_string()).collect(),
        }
    }
}

#[allow(unused_variables)]
#[component]
pub fn RoleListRenderer<F>(
    #[prop(into)] class: MaybeSignal<String>,
    #[prop(into)] value: MaybeSignal<Vec<RoleId>>,
    index: usize,
    on_change: F,
) -> impl IntoView
where
    F: Fn(Vec<RoleId>) + 'static,
{
    view! { <ul>{value().into_iter().map(move |role| view! { <li>{role.to_string()}</li> }).collect_view()}</ul> }
}

#[component]
pub fn Users() -> impl IntoView {
    let users = create_rw_signal::<Vec<User>>(vec![]);
    let _r = create_resource(
        || (),
        move |_| async move {
            let doofers: Vec<User> = user::list_users()
                .await
                .unwrap_or(vec![])
                .into_iter()
                .map(|u| u.into())
                .collect();

            users.set(doofers);
        },
    );
    view! {
      <section class="section">
        <div class="container">
          <h1>Users</h1>
          <UserTable items=users/>
        </div>
      </section>
    }
}

