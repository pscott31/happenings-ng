use leptos::*;
use leptos_router::*;
use tracing::{info, warn};

use super::navbar::NavBar;
use super::not_found::NotFound;
use crate::book_event::{Booking, BookingPage, BookingRoot, CheckPayment, EventPage, GeneratePaymentLink, ListBookings};
use crate::events::Events;
use crate::sign_in::{OAuthReturn, SignIn};
use crate::users::Users;
use happenings::person::{get_logged_in_person, Person};

#[derive(Clone, Debug, PartialEq)]
pub enum SignInStatus {
    NotVisible,
    Welcome,
    Password(String),
    CreateUser(String),
}
#[derive(Copy, Clone)]
pub struct SignInSignal(pub RwSignal<SignInStatus>);
// #[derive(Copy, Clone)]
pub type MaybePersonSignal = Signal<Option<Person>>;

#[derive(Clone, Debug, PartialEq)]
pub enum SessionID {
    NotSet,
    Set(String),
}

impl SessionID {
    pub fn store_cookie(&self) {
        // TODO: Decide properly if we're using local storage or cookies
        #[cfg(target_arch = "wasm32")]
        match self {
            Self::Set(id) => wasm_cookies::set(
                "session_id",
                id.as_ref(),
                &wasm_cookies::CookieOptions {
                    path: Some("/"),
                    ..Default::default()
                },
            ),
            Self::NotSet => wasm_cookies::delete("session_id"),
        }
    }

    pub fn from_cookie() -> Self {
        // TODO: Decide properly if we're using local storage or cookies

        info!("Getting sid from cookie");
        #[cfg(target_arch = "wasm32")]
        match wasm_cookies::get("session_id") {
            None => Self::NotSet,
            Some(Err(_)) => Self::NotSet,
            Some(Ok(id)) => Self::Set(id),
        }
        #[cfg(not(target_arch = "wasm32"))]
        Self::NotSet
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(SignInSignal(create_rw_signal(SignInStatus::NotVisible)));

    let (session_id, set_session_id) = create_signal(SessionID::from_cookie());
    create_effect(move |_| session_id().store_cookie());

    provide_context(set_session_id);

    // let (get_session, _, _) = use_local_storage::<Option<common::Session>, JsonCodec>("session");
    let user_info = create_resource(session_id, |sid| async move {
        match sid {
            SessionID::Set(_) => match get_logged_in_person().await {
                Ok(p) => Some(p),
                Err(e) => {
                    warn!("Error getting logged in person: {:?}", e);
                    None
                }
            },
            SessionID::NotSet => None,
        }
    });

    let maybe_person = Signal::derive(move || user_info.get().flatten());
    // let maybe_person = Signal::derive(move || user_info.get().and_then(|r| r.ok()));
    provide_context::<MaybePersonSignal>(maybe_person);
    view! {
      <Router>
        <Routes>
          <Route path="/" view=|| with_navbar(Events())/>
          <Route path="/users" view=|| with_navbar(Users())/>
          <Route path="/events" view=|| with_navbar(Events())/>
          <Route path="/events/:id" view=|| with_navbar(EventPage())>
            <Route path="bookings" view=ListBookings/>
            <Route path="book" view=BookingPage/>
          </Route>

          <Route path="/booking" view=|| with_navbar(BookingRoot())>
            <Route path=":booking_id" view=Booking>
              <Route path="generate_payment_link" view=GeneratePaymentLink/>
              <Route path="check_payment" view=CheckPayment/>
              <Route path="" view=|| view! {}/>
            </Route>
          </Route>

          <Route path="/oauth_return" view=OAuthReturn/>
          <Route path="/*any" view=NotFound/>
        </Routes>
      </Router>

      <SignIn/>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    view! { hello }
}

pub fn with_navbar<T>(child: T) -> impl IntoView
where
    T: IntoView,
{
    view! {
      <NavBar/>
      {child}
    }
}

