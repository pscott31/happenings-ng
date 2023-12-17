use leptos::*;
use leptos_use::storage::{use_local_storage, JsonCodec};
use logging::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::{Array, Reflect};

use crate::app::SignInSignal;
use crate::error_handling::*;

#[component]
pub fn SignIn() -> impl IntoView {
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;
    let trigger_oauth_popup =
        create_action(move |()| oauth_popup(move || sign_in_signal.set(false)));

    view! {
      <div class="modal" class:is-active=sign_in_signal>
        <div class="modal-background"></div>
        <div class="modal-content" style="width: 30em">
          <div class="box is-flex is-flex-direction-column is-align-items-stretch has-text-centered">
            <img src="/static/logo-vertical.png" height="40" class="px-6"/>
            <h1 class="subtitle my-4">Sign in to continue</h1>
            <div class="field">
              <div class="control">
                <input class="input is-primary" type="text" placeholder="Email Address"/>
              </div>
            </div>
            <button class="button is-primary">Continue</button>
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
          </div>
        </div>
        <button class="modal-close is-large" aria-label="close" on:click=move |_| sign_in_signal.set(false)></button>
      </div>
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
    let login_url = format!("{}/api/login", base);

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

    let url = format!("{}/api/oauth_return{}", base, query);
    let resp = reqwest::get(url).await.unwrap();

    if let Err(_) = resp.error_for_status_ref() {
        let resp = resp
            .json::<common::ErrorResponse>()
            .await
            .map_err(|e| e.to_string())?;
        return Err(resp.message.into());
    }

    let resp = resp
        .json::<common::OAuthReturnResponse>()
        .await
        .map_err(|e| format!("unable to decode response: {}", e.to_string()))?;

    let (_, set_state, _) =
        use_local_storage::<Option<common::OAuthReturnResponse>, JsonCodec>("session_info");
    log!("{:?}", &resp);

    set_state(Some(resp));

    let opener = window.opener().unwrap();
    let post_message = Reflect::get(&opener, &JsValue::from_str("postMessage")).unwrap();

    let args = Array::new();
    args.push(&JsValue::from_str("auth_ok"));
    let _ = Reflect::apply(post_message.unchecked_ref(), &opener, &args)
        .map_err(|_| "unable to push auth event");

    Ok(())
}

pub async fn check_user(
    session: Option<common::OAuthReturnResponse>,
) -> Result<common::UserInfoReponse, AppError> {
    let window = leptos::window();
    let base = window.location().origin().unwrap(); //todo

    let url = format!("{}/api/user", base);

    let session = session.ok_or("no session in local storage")?;
    let resp = reqwest::Client::new()
        .get(url)
        .header("Authorization", session.session_id)
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

