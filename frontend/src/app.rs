use leptos::*;
use leptos_router::*;
use leptos_use::storage::{use_local_storage, JsonCodec};

use super::navbar::NavBar;
use super::not_found::NotFound;
use crate::book_event::{Booking, BookingPage, BookingPayment, BookingRoot, CheckPayment};
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

#[component]
pub fn App() -> impl IntoView {
    provide_context(SignInSignal(create_rw_signal(SignInStatus::NotVisible)));

    let (get_session, _, _) = use_local_storage::<Option<common::Session>, JsonCodec>("session");
    let user_info = create_resource(get_session, |_| get_logged_in_person());
    let maybe_person = Signal::derive(move || user_info.get().and_then(|r| r.ok()));
    provide_context::<MaybePersonSignal>(maybe_person);
    view! {
      <Router>
        <Routes>
          <Route path="/" view=|| with_navbar(Home())/>
          <Route path="/users" view=|| with_navbar(Users())/>
          <Route path="/events" view=|| with_navbar(Events())/>
          <Route path="/events/:id/book" view=|| with_navbar(BookingPage())>
            // <BookingRoutes/>
            <Route path="" view=|| view! { <p>Default  stuff</p> }/>
            <Route path=":booking_id/payment" view=BookingPayment/>
          </Route>

          <Route path="/booking" view=|| with_navbar(BookingRoot())>
            <Route path=":booking_id" view=Booking>
              <Route path="check_payment" view=CheckPayment/>
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

