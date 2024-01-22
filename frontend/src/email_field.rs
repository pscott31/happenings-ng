// use email_address::*;
// use leptos::logging::*;
// use leptos::*;
// use leptos_icons::{FaIcon::*, Icon};
// use std::str::FromStr;

// #[component]
// pub fn Email(
//     #[prop(into)] get: Signal<String>,
//     #[prop(into)] set: Callback<String>,
// ) -> impl IntoView {
//     let email_address = Signal::derive(move || EmailAddress::from_str(&get()));
//     let email_err = move || match email_address() {
//         Ok(_) => None,
//         Err(e) => {
//             let msg = if get().is_empty() {
//                 "Please enter your email address".to_string()
//             } else {
//                 format!("Invalid email address: {}", e)
//             };
//             Some(view! { <p class="help is-danger">{msg}</p> })
//         }
//     };

//     let email_right_icon = move || {
//         if email_address().is_ok() {
//             Some(view! {
//               <span class="icon is-small is-right">
//                 <Icon icon=Icon::from(FaCheckSolid)/>
//               </span>
//             })
//         } else {
//             Some(view! {
//               <span class="icon is-small is-right">
//                 <Icon icon=Icon::from(FaTriangleExclamationSolid)/>
//               </span>
//             })
//         }
//     };

//     view! {
//       <div class="field is-horizontal">
//         <div class="field-label">
//           <label class="label">Email Address</label>
//         </div>
//         <div class="field-body">
//           <div class="field">
//             <div class="control has-icons-left has-icons-right">
//               <input
//                 class="input"
//                 class:is-success=move || { email_address().is_ok() }
//                 class:is-danger=move || { email_address().is_err() }
//                 type="text"
//                 placeholder="joe@blogs.com"
//                 on:input=move |ev| {
//                     log!("yay: {:?}", email_address());
//                     set(event_target_value(&ev));
//                 }
//               />

//               <span class="icon is-small is-left">
//                 <Icon icon=Icon::from(FaEnvelopeSolid)/>
//               </span>
//               {email_right_icon}
//             </div>
//             <div>{email_err}</div>
//           </div>
//         </div>
//       </div>
//     }
// }

// #[component]
// pub fn EmailControl(
//     #[prop(into)] get: Signal<String>,
//     #[prop(into)] set: Callback<String>,
// ) -> impl IntoView {
//     let email_address = Signal::derive(move || EmailAddress::from_str(&get()));
//     let email_err = move || match email_address() {
//         Ok(_) => None,
//         Err(e) => {
//             let msg = if get().is_empty() {
//                 "Please enter your email address".to_string()
//             } else {
//                 format!("Invalid email address: {}", e)
//             };
//             Some(view! { <p class="help is-danger">{msg}</p> })
//         }
//     };

//     let email_right_icon = move || {
//         if email_address().is_ok() {
//             Some(view! {
//               <span class="icon is-small is-right">
//                 <Icon icon=Icon::from(FaCheckSolid)/>
//               </span>
//             })
//         } else {
//             Some(view! {
//               <span class="icon is-small is-right">
//                 <Icon icon=Icon::from(FaTriangleExclamationSolid)/>
//               </span>
//             })
//         }
//     };

//     view! {
//       <div class="control has-icons-left has-icons-right">
//         <input
//           class="input"
//           class:is-success=move || { email_address().is_ok() }
//           class:is-danger=move || { email_address().is_err() }
//           type="text"
//           placeholder="Email Address"
//           on:input=move |ev| {
//               log!("yay: {:?}", email_address());
//               set(event_target_value(&ev));
//           }
//         />

//         <span class="icon is-small is-left">
//           <Icon icon=Icon::from(FaEnvelopeSolid)/>
//         </span>
//         {email_right_icon}
//       </div>
//       <div>{email_err}</div>
//     }
// }

