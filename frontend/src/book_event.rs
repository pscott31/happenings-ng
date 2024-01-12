use crate::app::{MaybePersonSignal, SignInSignal, SignInStatus};
use crate::components::controls::*;
use crate::field::Field;
use crate::icon_button::{Color, IconButton};
use crate::reactive_list::{ReactiveList, TrackableList};
use happenings::booking::{self, get_booking, CreateBooking, CreatePaymentLink, Status};
use happenings::event::{get_event, Event};
use happenings::person::{get_person, Person};
use happenings::ticket::Ticket;
use leptos::*;
use leptos_icons::FaIcon::*;
use leptos_router::{use_params_map, Outlet};
use log::*;
use url::Url;

fn notify(msg: &str, color: Color) -> View {
    view! { <div class=format!("notification has-text-centered {}", color)>{msg.to_owned()}</div> }
        .into_view()
}

#[component]
pub fn BookingRoot() -> impl IntoView {
    view! { <Outlet/> }
}

#[component]
pub fn Booking() -> impl IntoView {
    let params = use_params_map();
    let booking_id = move || {
        params()
            .get("booking_id")
            .cloned()
            .unwrap_or_default()
            .to_string()
    };

    let booking = create_resource(booking_id, |id| async move { get_booking(id).await });

    let booking_summary = move || match booking.get() {
        None => view! { <p>Loading..</p> }.into_view(),
        Some(Err(e)) => {
            warn!("error loading booking: {:?}", e);
            notify("Error loading booking", Color::Danger)
        }
        Some(Ok(booking)) => view! { <BookingSummary booking=store_value(booking)/> }.into_view(),
    };

    view! {
      {booking_summary}
      <Outlet/>
    }
}

#[component]
pub fn BookingSummary(#[prop(into)] booking: Signal<booking::Booking>) -> impl IntoView {
    let event = create_resource(booking, |b| async move { get_event(b.event_id).await });
    let contact = create_resource(booking, |b| async move { get_person(b.contact_id).await });

    let event_name = move || event.get().map(|er| er.map(|e| e.name).unwrap_or_default());
    let event_tagline = move || {
        event
            .get()
            .map(|er| er.map(|e| e.tagline).unwrap_or_default())
    };

    let full_name = Signal::derive(move || {
        contact
            .get()
            .map(|cr| cr.map(|c| c.full_name()).unwrap_or_default())
            .unwrap_or_default()
    });

    let tickets_view = move || {
        booking
            .get()
            .tickets
            .into_iter()
            .enumerate()
            .map(|(i, t)| {
                let special = if t.dietary_requirements != "" {
                    t.dietary_requirements
                } else {
                    "no special requirements".to_string()
                };

                view! {
                  <Field label=move || format!("Ticket {}", { i + 1 })>
                    <div class="control">
                      <input class="input" disabled=true prop:value=t.ticket_type.name.clone()/>
                      <Checkbox label="Vegetarian" get=store_value(t.vegetarian)/>
                      <Checkbox label="Gluten Free" get=store_value(t.gluten_free)/>
                      <Text placeholder="Other dietary requirements" get=special/>
                    </div>
                  </Field>
                }
            })
            .collect_view()
    };

    view! {
      <section class="section">
        <div class="container">
          <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            <h1 class="title">{event_name}</h1>
            <p class="subtitle">{event_tagline}</p>
            <Field label=|| "Booking Contact">
              <Name get=full_name disabled=true/>
            </Field>
            {tickets_view}
          </Suspense>
        </div>
      </section>
    }
}

#[component]
pub fn CheckPayment() -> impl IntoView {
    let params = use_params_map();
    let booking_id = move || {
        params()
            .get("booking_id")
            .cloned()
            .unwrap_or_default()
            .to_string()
    };

    let status = create_resource(
        booking_id,
        |id| async move { booking::check_payment(id).await },
    );

    let status_view = move || match status.get() {
        None => view! { <p>"Checking Payment Status..."</p> }.into_view(),
        Some(Err(e)) => {
            warn!("error checking payment: {:?}", e);
            notify("Error checking payment", Color::Danger)
        }
        Some(Ok(booking)) => match booking.status {
            Status::Paid => notify("All paid, thanks", Color::Success),
            Status::PartiallyPaid => notify("Partial payment recieved", Color::Warning),
            Status::Draft => notify("Expected payment not found", Color::Danger),
            Status::Cancelled => notify("Order cancelled", Color::Danger),
            Status::Accepted => notify("Payment expected on the door", Color::Warning),
        },
    };

    view! {
      {status_view}
      <Outlet/>
    }
}

#[component]
pub fn EventPage() -> impl IntoView {
    // let params = use_params_map();
    // let event_id = params.with(|p| p.get("id").cloned().unwrap_or_default());

    let params = use_params_map();
    let (event, set_event) = create_signal::<Option<Event>>(None);

    let _event_res = create_resource(
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        move |id| async move {
            let er = get_event(id).await;
            match er {
                Ok(evt) => set_event(Some(evt)),
                Err(e) => {
                    warn!("problem getting event: {:?}", e);
                    set_event(None)
                }
            }
        },
    );

    provide_context(event);
    view! {
      <Suspense fallback=move || view! { <p>"Loading Event..."</p> }>
        <Outlet/>
      </Suspense>
    }
}

#[component]
pub fn ListBookings() -> impl IntoView {
    let event = use_context::<ReadSignal<Option<Event>>>().unwrap();
    let bookings = create_resource(event, |maybe_event| async move {
        match maybe_event {
            Some(event) => match booking::list_bookings(event.id.clone()).await {
                Ok(bookings) => bookings,
                Err(e) => {
                    warn!("Error listing bookings: {}", e);
                    Default::default()
                }
            },
            None => Default::default(),
        }
    });

    let event_name = move || {
        event()
            .map(|e| e.name)
            .unwrap_or("<unknown event>".to_string())
    };

    view! {
      <section class="section">
        <div class="container">
          <h1 class="title">Bookings for {event_name}</h1>
          <Outlet/>
        </div>
      </section>
    }
}

#[component]
pub fn BookingPage() -> impl IntoView {
    // let params = use_params_map();
    // let event_id = params.with(|p| p.get("id").cloned().unwrap_or_default());

    // TODO: use event from context
    let params = use_params_map();
    let event_res = create_resource(
        move || params.with(|p| p.get("id").cloned().unwrap_or_default()),
        move |id| get_event(id),
    );

    {
        move || match event_res.get() {
            None => view! { <p>"Loading..."</p> }.into_view(),
            Some(Err(_e)) => view! { <p>"oops"</p> }.into_view(), //TODO
            Some(Ok(event)) => view! { <NewBooking event=store_value(event)/> }.into_view(),
        }
    }
}

// Todo: This is a bit crappy
pub fn require_login() {
    let person = use_context::<MaybePersonSignal>().unwrap();
    let sign_in_signal = use_context::<SignInSignal>().unwrap().0;
    create_effect(move |_| {
        if person().is_none() && sign_in_signal.get() == SignInStatus::NotVisible {
            sign_in_signal.set(SignInStatus::Welcome)
        }
        if person().is_some() && sign_in_signal.get() != SignInStatus::NotVisible {
            sign_in_signal.set(SignInStatus::NotVisible)
        }
    });
}

#[component]
pub fn NewBooking(#[prop(into)] event: Signal<Event>) -> impl IntoView {
    require_login();
    let person = use_context::<MaybePersonSignal>().unwrap();

    // todo: reactive?
    provide_context(store_value(event().ticket_types()));

    move || match person.get() {
        None => view! { <p>"Loading"</p> }.into_view(),
        Some(p) => {
            let sp = store_value(p);
            view! { <NewBookingForPerson person=sp event=event/> }.into_view()
        }
    }
}

fn url_for_path(path: String) -> String {
    // TODO: must be a better way?better way?
    let window = web_sys::window().unwrap();
    let url = Url::parse(window.location().href().unwrap().as_ref()).unwrap();
    let mut redirect_url = url.clone();
    redirect_url.set_path(path.as_ref());
    redirect_url.to_string()
}

#[component]
pub fn BookingPayment() -> impl IntoView {
    let params = use_params_map();
    let booking_id = params.with(|p| p.get("booking_id").cloned().unwrap_or_default());

    let create_payment_link = create_server_action::<CreatePaymentLink>();

    create_payment_link.dispatch(CreatePaymentLink {
        booking_id: booking_id.clone(),
        redirect_to: url_for_path(format!("booking/{}/check_payment", &booking_id)),
    });

    create_effect(move |_| {
        create_payment_link.value().with(|x| {
            if let Some(Ok(res)) = x {
                web_sys::window().unwrap().location().replace(res).unwrap();
            }
        })
    });

    view! { <p>Paying for {booking_id}</p> }
}

#[component]
pub fn NewBookingForPerson(
    #[prop(into)] person: Signal<Person>,
    #[prop(into)] event: Signal<Event>,
) -> impl IntoView {
    let full_name = Signal::derive(move || person.get().full_name());
    let event_name = move || event.with(|e| e.name.clone());
    let event_tagline = move || event.with(|e| e.tagline.clone());

    let tickets = vec![Ticket::new(event().default_ticket_type.clone())];
    let (tickets, set_tickets) = create_signal::<ReactiveList<Ticket>>(tickets.into());

    let add_ticket = move || {
        set_tickets.tracked_push(Ticket::new(event().default_ticket_type.clone()));
    };

    let badgers = move || {
        tickets.with(|gl| {
            debug!("recomuting badger");
            gl.iter()
                .enumerate()
                .map(|(i, (&uid, &gv))| {
                    if i == 0 {
                        view! {
                          <Field label=move || format!("Ticket {}", { i + 1 })>
                            <TicketControl ticket=gv/>
                          </Field>
                        }
                    } else {
                        view! {
                          <Field label=move || {
                              view! {
                                {format!("Ticket {}", { i + 1 })}
                                <br/>
                                <IconButton on_click=move || set_tickets.tracked_remove(uid) icon=FaTrashSolid/>
                              }
                          }>
                            <TicketControl ticket=gv/>
                          </Field>
                        }
                    }
                })
                .collect_view()
        })
    };

    let create_booking = create_server_action::<CreateBooking>();

    let navigate = leptos_router::use_navigate();

    let on_submit = move || {
        let booking = CreateBooking {
            event: event().id.clone(),
            contact: person().id.clone(),
            tickets: tickets().into(),
        };
        create_booking.dispatch(booking);
    };

    create_effect(move |_| {
        create_booking.value().with(|x| {
            if let Some(Ok(res)) = x {
                navigate(
                    format!("/events/{}/book/{}/payment", res.event_id, res.id).as_ref(),
                    Default::default(),
                )
            }
        })
    });

    view! {
      <section class="section">
        <input type="hidden" name="event" value=event().id/>
        <input type="hidden" name="contact" value=person().id/>
        <div class="container">
          <h1 class="title">{event_name}</h1>
          <p class="subtitle">{event_tagline}</p>

          <div class="box">
            <Field label=|| "Booking Contact">
              <Name get=full_name disabled=true/>
            </Field>
            {badgers}

            <div class="field is-grouped is-flex-wrap-wrap">
              <p class="control">
                <IconButton icon=FaPlusSolid color=Color::Secondary on_click=add_ticket>
                  "Add Another Ticket"
                </IconButton>
              </p>
              <p class="control">
                <IconButton icon=FaBasketShoppingSolid color=Color::Primary on_click=on_submit>
                  Pay Now
                </IconButton>
              </p>
              <Outlet/>
            </div>
          </div>
        </div>
      </section>
    }
}

