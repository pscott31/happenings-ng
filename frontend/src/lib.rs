mod app;
mod book_event;
mod components;
mod email_field;
mod error_handling;
mod events;
mod field;
mod icon_button;
mod navbar;
mod not_found;
mod reactive_list;
mod sign_in;
mod slot_state;
mod users;
mod utils;

use app::App;
use leptos::*;
use slot_state::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn mount_app() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <App/> })
}

