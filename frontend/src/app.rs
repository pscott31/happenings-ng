use leptos::*;
use leptos_router::*;
use logging::*;

use super::navbar::NavBar;
use super::not_found::NotFound;
use crate::sign_in::{OAuthReturn, SignIn};

#[derive(Copy, Clone)]
pub struct SignInSignal(pub RwSignal<bool>);

#[component]
pub fn App() -> impl IntoView {
    provide_context(SignInSignal(create_rw_signal(false)));

    view! {
      <Router>
        <Routes>
          <Route path="/" view=|| with_navbar(Home())/>
          <Route path="/oauth_return" view=OAuthReturn/>
          <Route path="/*any" view=NotFound/>
        </Routes>
      </Router>

      <SignIn/>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    view! { hello }
}

pub fn with_navbar<T>(child: T) -> impl IntoView
where
    T: IntoView,
{
    view! {
      <NavBar/>
      {child}
    }
}

