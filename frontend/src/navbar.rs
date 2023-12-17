use crate::app::SignInSignal;
use crate::sign_in::check_user;
use leptos::*;
use leptos_use::storage::{use_local_storage, JsonCodec};
use logging::*;

#[component]
pub fn NavBar() -> impl IntoView {
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;

    let (get_session, _, _) =
        use_local_storage::<Option<common::OAuthReturnResponse>, JsonCodec>("session_info");

    let user_info = create_resource(get_session, check_user);

    let dudger = move || match user_info() {
        None => "loading".into_view(),
        Some(Ok(ui)) => format!("{} {}", ui.given_name, ui.family_name).into_view(),
        Some(Err(_)) => {
            view! {
              <a class="button is-primary" on:click=move |_| sign_in_signal.set(true)>
                <strong>Sign in</strong>
              </a>
            }
            .into_view()
        }
    };

    view! {
      <nav class="navbar" role="navigation" aria-label="main navigation">
        <div class="navbar-brand">
          <a class="navbar-item" href="/">
            <img src="/static/logo.png" height="40"/>
          </a>

          <a
            role="button"
            class="navbar-burger"
            aria-label="menu"
            aria-expanded="false"
            data-target="navbarBasicExample"
          >
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
          </a>
        </div>

        <div id="navbarBasicExample" class="navbar-menu">
          <div class="navbar-start">
            <a class="navbar-item">Home</a>

            <a class="navbar-item">Documentation</a>

            <div class="navbar-item has-dropdown is-hoverable">
              <a class="navbar-link">More</a>

              <div class="navbar-dropdown">
                <a class="navbar-item">About</a>
                <a class="navbar-item">Jobs</a>
                <a class="navbar-item">Contact</a>
                <hr class="navbar-divider"/>
                <a class="navbar-item">Report an issue</a>
              </div>
            </div>
          </div>

          <div class="navbar-end">
            <div class="navbar-item">
              <div class="buttons">{dudger}</div>
            </div>
          </div>
        </div>
      </nav>
    }
}

