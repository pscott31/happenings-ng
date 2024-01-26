use common::auth::oauth::{check, OAuthRedirect};
use common::auth::password;
use common::error_handling::ErrorResponse;
use common::person::person_exists;
use leptos::*;
use leptos_router::*;
use logging::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Array, Reflect};

use crate::app::{SessionID, SignInSignal, SignInStatus};
use crate::error_handling::*;

#[component]
pub fn SignIn() -> impl IntoView {
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;
    let sign_in_page = move || match sign_in_signal() {
        SignInStatus::NotVisible => "".into_view(),
        SignInStatus::Welcome => SignInWelcome.into_view(),
        SignInStatus::CreateUser(email) => view! { <SignUpPassword email=email/> },
        SignInStatus::Password(email) => view! { <SignInPassword email=email/> },
    };

    view! {
      <div class="modal" class:is-active=move || sign_in_signal.get() != SignInStatus::NotVisible>
        <div class="modal-background"></div>
        <div class="modal-content" style="width: 30em">
          <div class="box is-flex is-flex-direction-column is-align-items-stretch has-text-centered">
            <img src="/static/logo-vertical.png" height="40" class="px-6"/>
            {sign_in_page}
          </div>
        </div>
        <button
          class="modal-close is-large"
          aria-label="close"
          on:click=move |_| sign_in_signal.set(SignInStatus::NotVisible)
        ></button>
      </div>
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct EmailPassword {
    pub email: String,
    pub password: String,
}
#[component]
pub fn SignInPassword(email: String) -> impl IntoView {
    let email = store_value(email);
    let (password, set_password) = create_signal("".to_string());
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;

    let set_session_id = use_context::<WriteSignal<SessionID>>().unwrap();

    let submit = create_action(move |ep: &EmailPassword| {
        let ep = ep.clone();
        async move {
            let session_id = password::signin(ep.email, ep.password)
                .await
                .map_err(|e| format!("{:?}", e))?;
            set_session_id(SessionID::Set(session_id));
            sign_in_signal.set(SignInStatus::NotVisible);
            Ok::<(), String>(())
        }
    });

    view! {
      <form on:submit=move |e| {
          e.prevent_default();
          submit
              .dispatch(EmailPassword {
                  email: email(),
                  password: password(),
              })
      }>

        <div class="block">
          <h1 class="subtitle my-4">Hello again</h1>
          <div class="field">
            <div class="control is-expanded">
              <input
                class="input"
                type="password"
                placeholder="Password"
                on:change=move |e| set_password(event_target_value(&e))
              />
            </div>
          </div>
          <ErrorNotification sig=submit.value()/>
          <button class="button is-primary is-fullwidth" type="submit">
            Continue
          </button>
        </div>
      </form>
    }
}

trait JsonError {
    async fn json_error_for_status(self) -> Result<Self, String>
    where
        Self: Sized;
}

impl JsonError for reqwest::Response {
    async fn json_error_for_status(self) -> Result<Self, String> {
        if self.error_for_status_ref().is_err() {
            let resp = self
                .json::<ErrorResponse>()
                .await
                .map_err(|e| e.to_string())?;
            return Err(resp.message);
        }
        Ok(self)
    }
}

// TODO I don't think we want this

#[component]
pub fn ErrorNotification<T, E>(#[prop(into)] sig: Signal<Option<Result<T, E>>>) -> impl IntoView
where
    T: Clone + 'static,
    E: Clone + Into<String> + 'static,
{
    move || match sig.get() {
        Some(Err(e)) => view! { <div class="notification is-danger">{e.into()}</div> }.into_view(),
        _ => "".into_view(),
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NewUser {
    pub given_name: String,
    pub family_name: String,
    // pub picture: String,
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
}
#[component]
pub fn SignUpPassword(email: String) -> impl IntoView {
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;

    let email = store_value(email);
    let (given_name, set_given_name) = create_signal("".to_string());
    let (family_name, set_family_name) = create_signal("".to_string());
    let (phone, set_phone) = create_signal("".to_string());
    let (password1, set_password1) = create_signal("".to_string());
    let (password2, set_password2) = create_signal("".to_string());

    let password_mismatch = Signal::derive(move || password1() != password2());
    let is_invalid = password_mismatch;

    let set_session_id = use_context::<WriteSignal<SessionID>>().unwrap();

    let submit = create_action(move |new_user: &NewUser| {
        let new_user = new_user.clone();
        async move {
            let _res = common::auth::password::signup_password(
                new_user.email.clone(),
                new_user.password.clone(),
                new_user.given_name,
                new_user.family_name,
                new_user.phone,
            )
            .await; //todo - check resut

            match common::auth::password::signin(new_user.email, new_user.password).await {
                // todo handle error
                Err(_e) => {}
                Ok(session_id) => {
                    set_session_id(SessionID::Set(session_id));
                    sign_in_signal.set(SignInStatus::NotVisible);
                }
            }

            Ok::<(), String>(())
        }
    });

    let error_notify = move || match submit.value()() {
        Some(Err(e)) => {
            view! { <div class="notification is-danger">{e.to_string()}</div> }.into_view()
        }
        _ => "".into_view(),
    };
    view! {
      <form on:submit=move |e| {
          e.prevent_default();
          if password1() != password2() {
              return;
          }
          submit
              .dispatch(NewUser {
                  email: email(),
                  given_name: given_name(),
                  family_name: family_name(),
                  phone: if !phone().is_empty() { Some(phone()) } else { None },
                  password: password1(),
              })
      }>

        <div class="block">
          <h1 class="subtitle my-4">Hello stranger</h1>
          <div>"Looks like you don't have an account yet. Let's fix that!"</div>
        </div>
        <div class="field is-grouped">
          <div class="control is-expanded">
            <input
              class="input"
              type="text"
              placeholder="Given Name"
              on:change=move |e| set_given_name(event_target_value(&e))
              value=given_name
            />
          </div>
          <div class="control is-expanded">
            <input
              class="input"
              type="text"
              placeholder="Family Name"
              on:change=move |e| set_family_name(event_target_value(&e))
            />
          </div>
        </div>
        <div class="field">
          <div class="control">
            <input
              class="input"
              type="text"
              placeholder="Phone Number (optional)"
              on:change=move |e| {
                  let ps = event_target_value(&e);
                  set_phone(ps);
              }
            />

          </div>
        </div>
        <div class="field is-grouped">
          <div class="control is-expanded">
            <input
              class="input"
              class:is-danger=password_mismatch
              type="password"
              placeholder="Password"
              on:change=move |e| set_password1(event_target_value(&e))
            />
          </div>
          <div class="control is-expanded">
            <input
              class="input"
              class:is-danger=password_mismatch
              type="password"
              placeholder="Password Confirmation"
              on:change=move |e| set_password2(event_target_value(&e))
            />
          </div>
        </div>
        <div class="field">
          <div class="control is-expanded">{error_notify}</div>
        </div>
        <div class="field">
          <div class="control">
            <button class="button is-primary is-fullwidth" disabled=is_invalid type="submit">
              Continue
            </button>
          </div>
        </div>
      </form>
    }
}

#[component]
pub fn SignInWelcome() -> impl IntoView {
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;
    let set_session = use_context::<WriteSignal<SessionID>>().unwrap();

    let on_success = move || {
        set_session(SessionID::from_cookie());
        sign_in_signal.set(SignInStatus::NotVisible);
    };

    let oauth_button = view! {
      <button
        class="button"
        type="button"
        on:click=move |_| {
            let _ = oauth_popup(OAuthRedirect::PATH, on_success);
        }
      >

        <span class="icon is-medium">
          <GoogleLogoSVG/>
        </span>
        <span>Sign in with Google</span>
      </button>
    };

    let (email, set_email) = create_signal("".to_string());

    let continue_pressed = create_action(move |email: &String| {
        let email = email.clone();
        async move {
            let user_exists = person_exists(email.clone()).await?;

            let next_status = if user_exists {
                SignInStatus::Password(email.clone())
            } else {
                SignInStatus::CreateUser(email.clone())
            };
            log!("updating sign_in_signal");
            sign_in_signal.set(next_status);
            Ok::<(), AppError>(())
        }
    });

    view! {
      <h1 class="subtitle my-4">Sign in to continue ay</h1>
      <form on:submit=move |e| {
          log!("form submission");
          e.prevent_default();
          continue_pressed.dispatch(email())
      }>
        <div class="field">
          <div class="control">
            <input
              class="input is-primary"
              type="text"
              placeholder="Email Address"
              on:change=move |e| set_email(event_target_value(&e))
            />
          </div>
        </div>
        <div class="field">
          <div class="control">
            <button class="button is-primary is-fullwidth" type="submit">
              Continue
            </button>
          </div>
        </div>
        <div class="level my-3">
          <hr class="level-item is-flex-shrink-2"/>
          <div class="is-size-7 px-2">OR</div>
          <hr class="level-item is-flex-shrink-2"/>
        </div>
        {oauth_button}
      </form>
    }
}

#[derive(Params, PartialEq, Clone)]
pub struct OAuthReturnParams {
    pub state: String,
    pub code: String,
}

#[component]
pub fn OAuthReturn() -> impl IntoView {
    let params = use_query::<OAuthReturnParams>();
    let set_session_id = use_context::<WriteSignal<SessionID>>().unwrap();

    let res = create_resource(params, move |param_res| async move {
        let p = match param_res {
            Ok(p) => p,
            Err(e) => return Err(format!("unable to read oauth query params: {}", e)),
        };

        match check(p.state, p.code).await {
            Err(e) => Err(format!("oauth check failed: {:?}", e)),
            Ok(session_id) => {
                set_session_id(SessionID::Set(session_id));
                close_popup();
                Ok(())
            }
        }
    });

    view! {
      <div style="min-height: 100vh; display: grid;">
        <div style="place-self: center" class="is-size-2">
          {move || match res.get() {
              None => "checking..".into_view(),
              Some(Ok(_)) => "success".into_view(),
              Some(Err(e)) => e.into_view(),
          }}

        </div>
      </div>
    }
}

fn oauth_popup<F>(url: &str, on_success: F) -> Result<(), AppError>
where
    F: Fn() + 'static,
{
    let popup = window()
        .open_with_url_and_target_and_features(url, "popup", "popup")
        .map_err(|_| "failed to open popup window".to_string())?
        .ok_or("failed to open popup window".to_string())?;

    // TODO: How do we remove this once we're done?
    let _remove_listener = leptos_use::use_event_listener(window(), ev::message, move |evt| {
        if evt.origin() == window().origin() {
            if let Some(msg_str) = evt.data().as_string() {
                if msg_str == "auth_ok" {
                    popup.close().unwrap(); // todo
                    on_success();
                }
            }
        }
    });

    Ok(())
}

fn close_popup() {
    let opener = window().opener().unwrap();
    let post_message = Reflect::get(&opener, &JsValue::from_str("postMessage")).unwrap();

    let args = Array::new();
    args.push(&JsValue::from_str("auth_ok"));
    let _ = Reflect::apply(post_message.unchecked_ref(), &opener, &args)
        .map_err(|_| "unable to push auth event");
}

#[component]
pub fn GoogleLogoSVG() -> impl IntoView {
    view! {
      <svg class="Bz112c Bz112c-E3DyYd" xmlns="https://www.w3.org/2000/svg" viewBox="0 0 48 48">
        <path
          fill="#4285F4"
          d="M45.12 24.5c0-1.56-.14-3.06-.4-4.5H24v8.51h11.84c-.51 2.75-2.06 5.08-4.39 6.64v5.52h7.11c4.16-3.83 6.56-9.47 6.56-16.17z"
        ></path>
        <path
          fill="#34A853"
          d="M24 46c5.94 0 10.92-1.97 14.56-5.33l-7.11-5.52c-1.97 1.32-4.49 2.1-7.45 2.1-5.73 0-10.58-3.87-12.31-9.07H4.34v5.7C7.96 41.07 15.4 46 24 46z"
        ></path>
        <path
          fill="#FBBC05"
          d="M11.69 28.18C11.25 26.86 11 25.45 11 24s.25-2.86.69-4.18v-5.7H4.34C2.85 17.09 2 20.45 2 24c0 3.55.85 6.91 2.34 9.88l7.35-5.7z"
        ></path>
        <path
          fill="#EA4335"
          d="M24 10.75c3.23 0 6.13 1.11 8.41 3.29l6.31-6.31C34.91 4.18 29.93 2 24 2 15.4 2 7.96 6.93 4.34 14.12l7.35 5.7c1.73-5.2 6.58-9.07 12.31-9.07z"
        ></path>
        <path fill="none" d="M2 2h44v44H2z"></path>
      </svg>
    }
}

