use icondata as i;
use leptos::*;
use leptos_icons::Icon;

#[component]
pub fn Name(
    #[prop(into)] get: MaybeSignal<String>,
    #[prop(into, default = Callback::new(|_|{}))] set: Callback<String>,
    #[prop(into, default = MaybeSignal::Static(false))] disabled: MaybeSignal<bool>,
) -> impl IntoView {
    let name_err = {
        let get = get.clone();
        move || {
            if get().is_empty() {
                Some(view! { <p class="help is-danger">"Please enter your name"</p> })
            } else {
                None
            }
        }
    };
    let name_err = Signal::derive(name_err);
    view! {
      <div class="control has-icons-left">
        <input
          class="input"
          class:is-success=move || { name_err().is_none() }
          class:is-danger=move || { name_err().is_some() }
          type="text"
          placeholder="Name"
          prop:value=get
          on:change=move |ev| set(event_target_value(&ev))
          disabled=disabled()
        />
        <span class="icon is-small is-left">
          <Icon icon=i::FaUserSolid/>
        </span>
      </div>
      {name_err}
    }
}

