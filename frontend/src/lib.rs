mod app;
mod components;
mod error_handling;
mod navbar;
mod not_found;
mod sign_in;
mod users;

use app::App;
use leptos::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn mount_app() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <App/> })
}

