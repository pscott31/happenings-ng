use email_address::*;
use icondata as i;
use leptos::logging::*;
use leptos::*;
use leptos_icons::Icon;
use std::str::FromStr;

#[component]
pub fn Email(
    #[prop(into)] get: Signal<String>,
    #[prop(into)] set: Callback<String>,
) -> impl IntoView {
    let email_address = Signal::derive(move || EmailAddress::from_str(&get()));
    let email_err = move || match email_address() {
        Ok(_) => None,
        Err(e) => {
            let msg = if get().is_empty() {
                "Please enter your email address".to_string()
            } else {
                format!("Invalid email address: {}", e)
            };
            Some(view! { <p class="help is-danger">{msg}</p> })
        }
    };

    let email_right_icon = move || {
        if email_address().is_ok() {
            Some(view! {
              <span class="icon is-small is-right">
                <Icon icon=i::FaCheckSolid/>
              </span>
            })
        } else {
            Some(view! {
              <span class="icon is-small is-right">
                <Icon icon=i::FaTriangleExclamationSolid/>
              </span>
            })
        }
    };

    view! {
      <div class="control has-icons-left has-icons-right">
        <input
          class="input"
          class:is-success=move || { email_address().is_ok() }
          class:is-danger=move || { email_address().is_err() }
          type="text"
          placeholder="Email Address"
          prop:value=get
          on:input=move |ev| {
              log!("yay: {:?}", email_address());
              set(event_target_value(&ev));
          }
        />

        <span class="icon is-small is-left">
          <Icon icon=i::FaEnvelopeSolid/>
        </span>
        {email_right_icon}
      </div>
      <div>{email_err}</div>
    }
}

