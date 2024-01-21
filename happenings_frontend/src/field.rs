use leptos::*;

#[component]
pub fn Field(children: Children, #[prop(optional, into)] label: ViewFn) -> impl IntoView
where
{
    let children = children()
        .nodes
        .into_iter()
        .map(|child| {
            view! { <div class="field">{child}</div> }
        })
        .collect_view();

    view! {
      <div class="field is-horizontal">
        <div class="field-label is-normal">
          <label class="label">{label.run()}</label>
        </div>
        <div class="field-body">{children}</div>
      </div>
    }
}

