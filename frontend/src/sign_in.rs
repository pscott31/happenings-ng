use common::NewUser;
use leptos::*;
use leptos_use::storage::{use_local_storage, JsonCodec};
use logging::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Array, Reflect};

use crate::app::{SignInSignal, SignInStatus};
use crate::error_handling::*;

#[component]
pub fn SignIn() -> impl IntoView {
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;
    let sign_in_page = move || match sign_in_signal() {
        SignInStatus::NotVisible => "".into_view(),
        SignInStatus::Welcome => SignInWelcome.into_view(),
        SignInStatus::CreateUser(email) => view! { <SignInCreateUser email=email/> },
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

#[component]
pub fn SignInPassword(email: String) -> impl IntoView {
    let email = store_value(email);
    let (password, set_password) = create_signal("".to_string());
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;

    let submit =
        create_action(move |ep: &common::EmailPassword| {
            let ep = ep.clone();
            async move {
                log!("Submitting!");
                let base = window()
                    .location()
                    .origin()
                    .map_err(|_| "failed to get window origin")?;

                let session = reqwest::Client::new()
                    .post(format!("{}/api/auth/password/signin", base))
                    .json(&ep)
                    .send()
                    .await
                    .map_err(|e| format!("failed to call backend api: {}", e))?
                    .json::<common::Session>()
                    .await
                    .map_err(|e| format!("failed to deserialize api response: {}", e))?;

                log!("got session: {}", session.id);
                store_session(session);
                sign_in_signal.set(SignInStatus::NotVisible);
                Ok::<(), String>(())
            }
        });

    view! {
      <div class="block">
        <h1 class="subtitle my-4">Hello again</h1>
        <div>{move || format!("{:?}", submit.value())}</div>
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
        <button
          class="button is-primary"
          on:click=move |_| {
              submit
                  .dispatch(common::EmailPassword {
                      email: email(),
                      password: password(),
                  })
          }
        >

          Continue
        </button>
      </div>
    }
}

trait JsonError {
    async fn json_error_for_status(self) -> Result<Self, String>
    where
        Self: Sized;
}

impl JsonError for reqwest::Response {
    async fn json_error_for_status(self) -> Result<Self, String> {
        if let Err(_) = self.error_for_status_ref() {
            let resp = self
                .json::<common::ErrorResponse>()
                .await
                .map_err(|e| e.to_string())?;
            return Err(resp.message.into());
        }
        Ok(self)
    }
}

#[component]
pub fn SignInCreateUser(email: String) -> impl IntoView {
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;

    let email = store_value(email);
    let (given_name, set_given_name) = create_signal("".to_string());
    let (family_name, set_family_name) = create_signal("".to_string());
    let (phone, set_phone) = create_signal("".to_string());
    let (password1, set_password1) = create_signal("".to_string());
    let (password2, set_password2) = create_signal("".to_string());

    let password_mismatch = Signal::derive(move || password1() != password2());
    let is_invalid = password_mismatch;

    let submit = create_action(move |u: &NewUser| {
        let u2 = u.clone();
        async move {
            log!("Submitting!");
            let base = window()
                .location()
                .origin()
                .map_err(|_| "failed to get window origin")?;
            let url = format!("{}/api/auth/password/signup", base);

            let _resp1 = reqwest::Client::new()
                .post(url)
                .json(&u2)
                .send()
                .await
                .map_err(|e| format!("failed to call backend api: {}", e))?
                .json_error_for_status()
                .await?;

            // Ok now try logging in
            let session = reqwest::Client::new()
                .post(format!("{}/api/auth/password/signin", base))
                .json(&u2)
                .send()
                .await
                .map_err(|e| format!("failed to call backend api: {}", e))?
                .json::<common::Session>()
                .await
                .map_err(|e| format!("failed to deserialize api response: {}", e))?;

            log!("got session: {}", session.id);
            store_session(session);
            sign_in_signal.set(SignInStatus::NotVisible);
            Ok::<(), String>(())
        }
    });

    // errorbox = move || match submit.value() {Some(Err(e))};
    view! {
      <div class="notification is-danger">{move || format!("value? {:?}", submit.value()())}</div>
      <div class="block">
        <h1 class="subtitle my-4">Hello stranger</h1>
        <div>"No account currently exists for {email}. Let's fix that!"</div>
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
      <button
        class="button is-primary"
        disabled=is_invalid
        on:click=move |_| {
            if password1() != password2() {
                return;
            }
            submit
                .dispatch(common::NewUser {
                    email: email(),
                    given_name: given_name(),
                    family_name: family_name(),
                    phone: if phone() != "" { Some(phone()) } else { None },
                    password: password1(),
                })
        }
      >

        Continue
      </button>
    }
}

#[component]
pub fn SignInWelcome() -> impl IntoView {
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;
    let trigger_oauth_popup =
        create_action(move |()| oauth_popup(move || sign_in_signal.set(SignInStatus::NotVisible)));
    let (email, set_email) = create_signal("".to_string());

    let continue_pressed = create_action(move |email: &String| {
        let email = email.clone();
        async move {
            let base = window()
                .location()
                .origin()
                .map_err(|_| "failed to get window origin")?;
            let url = format!("{}/api/user_exists", base);

            let user_exists = reqwest::Client::new()
                .get(url)
                .query(&common::Email {
                    email: email.clone(),
                })
                .send()
                .await
                .map_err(|e| format!("failed to call backend api: {}", e))?
                .json::<bool>()
                .await
                .map_err(|e| format!("failed to deserialize api response: {}", e))?;

            let next_status = if user_exists {
                SignInStatus::Password(email)
            } else {
                SignInStatus::CreateUser(email)
            };
            log!("updating sign_in_signal");
            sign_in_signal.set(next_status);
            Ok::<(), AppError>(())
        }
    });

    view! {
      <h1 class="subtitle my-4">Sign in to continue</h1>
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
        <button class="button is-primary" type="submit">
          Continue
        </button>
        <div class="level my-3">
          <hr class="level-item is-flex-shrink-2"/>
          <div class="is-size-7 px-2">OR</div>
          <hr class="level-item is-flex-shrink-2"/>
        </div>
        <button class="button" on:click=move |_| trigger_oauth_popup.dispatch(())>
          <span class="icon is-medium">
            <GoogleLogoSVG/>
          </span>
          <span>Sign in with Google</span>
        </button>
      </form>
    }
}

#[component]
pub fn OAuthReturn() -> impl IntoView {
    let login_oauth = create_action(|()| validate_oauth_return());
    login_oauth.dispatch(());

    view! {
      <div style="min-height: 100vh; display: grid;">
        <div style="place-self: center" class="is-size-2">
          {move || match login_oauth.value().get() {
              None => "checking..".into_view(),
              Some(Ok(_)) => "success".into_view(),
              Some(Err(e)) => e.into_view(),
          }}

        </div>
      </div>
    }
}

async fn oauth_popup<F>(on_succces: F) -> Result<(), AppError>
where
    F: Fn() + 'static,
{
    let base = window()
        .location()
        .origin()
        .map_err(|_| "failed to get window origin")?;
    let login_url = format!("{}/api/auth/oauth/link", base);

    let resp = reqwest::get(login_url)
        .await
        .map_err(|e| format!("failed to call backend api: {}", e))?
        .json::<common::LoginResponse>()
        .await
        .map_err(|e| format!("failed to deserialize api response: {}", e))?;

    let popup = window()
        .open_with_url_and_target_and_features(resp.url.as_str(), "popup", "popup")
        .map_err(|_| format!("failed to open popup window"))?
        .ok_or(format!("failed to open popup window"))?;

    // TODO: How do we remove this once we're done?
    let _remove_listener =
        leptos_use::use_event_listener(window(), ev::message, move |evt| {
            if evt.origin() == window().origin() && evt.data().as_string().unwrap() == "auth_ok" {
                popup.close().unwrap(); // todo
                on_succces();
            }
        });

    Ok(())
}

async fn validate_oauth_return() -> Result<(), AppError> {
    let window = leptos::window();
    let base = window.location().origin().unwrap(); //todo
    let query = window.location().search().unwrap(); //todo

    let url = format!("{}/api/auth/oauth/return{}", base, query);
    let resp = reqwest::get(url).await.unwrap();

    if let Err(_) = resp.error_for_status_ref() {
        let resp = resp
            .json::<common::ErrorResponse>()
            .await
            .map_err(|e| e.to_string())?;
        return Err(resp.message.into());
    }

    let session = resp
        .json::<common::Session>()
        .await
        .map_err(|e| format!("unable to decode response: {}", e.to_string()))?;

    store_session(session);

    let opener = window.opener().unwrap();
    let post_message = Reflect::get(&opener, &JsValue::from_str("postMessage")).unwrap();

    let args = Array::new();
    args.push(&JsValue::from_str("auth_ok"));
    let _ = Reflect::apply(post_message.unchecked_ref(), &opener, &args)
        .map_err(|_| "unable to push auth event");

    Ok(())
}

pub async fn check_user(
    session: Option<common::Session>,
) -> Result<common::UserInfoReponse, AppError> {
    let window = leptos::window();
    let base = window.location().origin().unwrap(); //todo

    let url = format!("{}/api/user", base);

    let session = session.ok_or("no session in local storage")?;
    let resp = reqwest::Client::new()
        .get(url)
        .header("Authorization", session.id)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if let Err(_) = resp.error_for_status_ref() {
        let resp = resp
            .json::<common::ErrorResponse>()
            .await
            .map_err(|e| e.to_string())?;
        return Err(resp.message.into());
    }

    let person = resp
        .json::<common::UserInfoReponse>()
        .await
        .map_err(|e| format!("unable to decode response: {}", e.to_string()))?;

    Ok(person)
}

fn store_session(session: common::Session) {
    let (_, set_state, _) = use_local_storage::<Option<common::Session>, JsonCodec>("session");
    log!("{:?}", &session);
    set_state(Some(session));
}

pub fn clear_session() {
    let (_, set_state, _) = use_local_storage::<Option<common::Session>, JsonCodec>("session");
    set_state(None);
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

