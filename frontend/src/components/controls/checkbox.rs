use leptos::*;

#[component]
pub fn Checkbox(
    #[prop(into)] label: String,
    #[prop(into)] get: Signal<bool>,
    #[prop(into)] set: Callback<bool>,
) -> impl IntoView {
    view! {
      <div class="control">
        <label class="checkbox">
          <input type="checkbox" prop:checked=get on:change=move |ev| set(event_target_checked(&ev))/>
          {format!(" {} ", label)}
        </label>
      </div>
    }
}

