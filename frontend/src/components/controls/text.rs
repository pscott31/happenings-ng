use leptos::*;
use leptos_icons::{FaIcon, Icon};

#[component]
pub fn Text(
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into)] get: MaybeSignal<String>,
    #[prop(into)] set: Callback<String>,
    #[prop(into, optional)] icon: Option<FaIcon>,
) -> impl IntoView {
    let icon_view = icon.map(|i| {
        view! {
          <span class=format!("icon is-small is-left")>
            <Icon icon=Icon::from(i)/>
          </span>
        }
    });

    let label_view = label.map(|l| {
        view! { <label class="label">{l}</label> }
    });

    view! {
      {label_view}
      <div class="control" class:has-icons-left=icon_view.is_some()>
        <input
          class="input"
          type="text"
          placeholder=placeholder
          prop:value=get
          on:change=move |ev| set(event_target_value(&ev))
        />
        {icon_view}
      </div>
    }
}

