use leptos::*;
use wasm_bindgen::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
      <h1>Hello from leptos land 9</h1>
    }
}

#[wasm_bindgen(start)]
pub fn mount_app() {
    leptos::mount_to_body(|| view! { <App/> })
}
