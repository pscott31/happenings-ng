use happenings::list_users;
use leptos::*;

#[component]
pub fn Users() -> impl IntoView {
    let (users, set_users) = create_signal("loading..".to_string());
    let r = create_resource(
        || (),
        move |_| async move {
            let doofers = list_users().await;
            let doofer = format!("{:?}", doofers);
            set_users(doofer);
        },
    );
    view! { <h1>Users: {users}</h1> }
}

