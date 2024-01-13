// extern crate phonenumber as pn;

// use leptos::*;

// #[component]
// pub fn PhoneNumber(
//     #[prop(into)] get: Signal<String>,
//     #[prop(into)] set: Callback<String>,
// ) -> impl IntoView {
//     let on_change = move |ev: leptos::ev::Event| set(event_target_value(&ev));

//     let is_valid = move || {
//         get.with(|s| match pn::parse(Some(pn::country::Id::GB), s) {
//             Ok(pn) => pn.is_valid(),
//             Err(_) => false,
//         })
//     };

//     let error_msg = move || {
//         (!is_valid())
//             .then_some(view! { <p class="help is-danger">Please enter a valid phone number</p> })
//     };

//     view! {
//       <p class="control is-expanded">
//         <input class="input" type="tel" placeholder="Phone number (optional)" prop:value=get on:change=on_change/>
//       </p>
//       <div>{error_msg}</div>
//     }
// }

