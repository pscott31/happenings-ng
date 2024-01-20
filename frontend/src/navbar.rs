use crate::app::{MaybePersonSignal, SessionID, SignInSignal, SignInStatus};
use leptos::*;
use leptos_router::A;
// use leptos_use::storage::{use_local_storage, JsonCodec};

#[component]
pub fn NavBar() -> impl IntoView {
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;
    let set_session = use_context::<WriteSignal<SessionID>>().unwrap();
    let user_info = use_context::<MaybePersonSignal>().unwrap();
    let menu_open = create_rw_signal(false);

    let dudger = move || match user_info() {
        Some(ui) => view! {
          <div class="navbar-item has-dropdown is-hoverable">
            <a class="navbar-link">{format!("{} {}", ui.given_name, ui.family_name)}</a>
            <div class="navbar-dropdown">
              // <a class="navbar-item">Profile</a>
              <a class="navbar-item" on:click=move |_| set_session(SessionID::NotSet)>
                Sign Out
              </a>
            </div>
          </div>
        }
        .into_view(),
        None => view! {
          <a class="button is-primary" on:click=move |_| sign_in_signal.set(SignInStatus::Welcome)>
            <strong>Sign in</strong>
          </a>
        }
        .into_view(),
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
            class:is-active=menu_open
            on:click=move |_| menu_open.set(!menu_open())
          >
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
          </a>
        </div>

        <div id="navbarBasicExample" class="navbar-menu" class:is-active=menu_open>
          <div class="navbar-start">
            <a class="navbar-item">Home</a>

            <A class="navbar-item" href="/events">
              Events
            </A>

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

