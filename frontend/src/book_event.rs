use crate::app::{MaybePersonSignal, SignInSignal, SignInStatus};
use crate::components::controls::*;
use crate::components::modal::Modal;
use crate::field::Field;
use crate::icon_button::{Color, IconButton};
use crate::reactive_list::{ReactiveList, TrackableList};
use crate::slot_state_for_ticket;

use class_list::class_list;
use common::booking::{self, get_booking, BookingId, CreateBooking, Status};
use common::event::{get_event, get_slot_details, Event, EventId, SlotDetail};
use common::person::{get_person, Person};
use common::ticket::Ticket;
use icondata as i;
use itertools::Itertools;
use leptos::*;
use leptos_router::{use_params_map, Outlet};
use log::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use url::Url;

#[component]
pub fn Loader() -> impl IntoView {
    view! { <span class="loader"></span> }
}

fn notify(msg: &str, color: Color) -> View {
    view! { <div class=class_list!("notification has-text-centered", color)>{msg.to_owned()}</div> }
        .into_view()
}

fn notify_details(msg: &str, details: String, color: Color) -> View {
    let modal_visible = create_rw_signal(false);
    let details = store_value(details);

    view! {
      <div class=class_list!("notification has-text-centered", color)>
        <div class="block">{msg.to_owned()}</div>
        <div class="block">
          <a on:click=move |_| modal_visible.set(true)>Show Error Details</a>
        </div>
      </div>
      <Modal
        active=modal_visible
        close_requested=move || modal_visible.set(false)
        title="Error Details"
        footer=|| view! {}
      >
        <div class="content">
          <div class="block">
            <pre style="white-space: pre-wrap;">{details}</pre>
          </div>
        </div>
      </Modal>
    }
    .into_view()
}

#[component]
pub fn BookingRoot() -> impl IntoView {
    view! { <Outlet/> }
}

#[component]
pub fn Booking() -> impl IntoView {
    let params = use_params_map();
    let booking_id = move || -> BookingId {
        params()
            .get("booking_id")
            .cloned()
            .unwrap_or_default()
            .into()
    };

    let booking = create_resource(booking_id, |id| async move { get_booking(id).await });

    let booking_summary = move || match booking.get() {
        None => view! { <p>Loading.. <Loader/></p> }.into_view(),
        Some(Err(e)) => {
            warn!("error loading booking: {:?}", e);
            notify("Error loading booking", Color::Danger)
        }
        Some(Ok(booking)) => view! { <BookingSummary booking=store_value(booking)/> }.into_view(),
    };

    view! {
      <div class="section">
        <Outlet/>
        {booking_summary}
      </div>
    }
}

#[component]
pub fn BookingSummary(#[prop(into)] booking: Signal<booking::Booking>) -> impl IntoView {
    // TODO: these are now eagerly fetched, don't need to fetch again
    let event = create_resource(booking, |b| async move { get_event(b.event.id).await });
    let contact = create_resource(booking, |b| async move { get_person(b.contact.id).await });

    let event_name = move || event.get().map(|er| er.map(|e| e.name).unwrap_or_default());

    let full_name = Signal::derive(move || {
        contact
            .get()
            .map(|cr| cr.map(|c| c.full_name()).unwrap_or_default())
            .unwrap_or_default()
    });

    let ticket_table_data = move || {
        booking
            .get()
            .tickets
            .into_iter()
            .enumerate()
            .map(|(i, t)| {
                let special = if !t.dietary_requirements.is_empty() {
                    t.dietary_requirements
                } else {
                    "no special requirements".to_string()
                };

                view! {
                  <tr>
                    <td>{format!("Ticket {}", { i + 1 })}</td>
                    <td>{t.ticket_type.name.clone()}</td>
                    <td>{t.slot_name.clone()}</td>
                    <td>{if t.vegetarian { "yes" } else { "no" }}</td>
                    <td>{if t.gluten_free { "yes" } else { "no" }}</td>
                    <td>{special}</td>
                  </tr>
                }
            })
            .collect_view()
    };

    view! {
      <div class="container">
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
          <h1 class="title">{event_name}</h1>
          <h3 class="title is-5">Booking for {full_name}</h3>
          <table class="table">
            <tr>
              <th></th>
              <th>Type</th>
              <th>Slot</th>
              <th>Vegetarian</th>
              <th>Gluten Free</th>
              <th>Notes</th>
            </tr>

            {ticket_table_data}
          </table>

        </Suspense>
      </div>
    }
}

#[component]
pub fn CheckPayment() -> impl IntoView {
    let params = use_params_map();
    let booking_id = move || -> BookingId {
        params.with(|p| p.get("booking_id").cloned().unwrap_or_default().into())
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

#[derive(Clone)]
pub struct ContextEvent(pub StoredValue<Event>);
#[derive(Clone)]
pub struct ContextSlotDetails(pub StoredValue<Vec<SlotDetail>>);

#[component]
pub fn EventProvider() -> impl IntoView {
    let params = use_params_map();
    let event_id =
        move || -> EventId { params.with(|p| p.get("id").cloned().unwrap_or_default().into()) };

    let event = create_resource(event_id, move |id| async move { get_event(id).await });
    let slot_details = create_resource(
        event_id,
        move |id| async move { get_slot_details(id).await },
    );

    // TODO: Must be a better way with Show/Suspense/ErrorBoundary or something
    move || match (event.get(), slot_details.get()) {
        (Some(Err(e)), _) => {
            warn!("error loading event: {:?}", e);
            notify("Error loading event", Color::Danger).into_view()
        }
        (_, Some(Err(e))) => {
            warn!("error loading slots: {:?}", e);
            notify("Error loading slots", Color::Danger).into_view()
        }
        (Some(Ok(event)), Some(Ok(slot_details))) => {
            provide_context(store_value(event.ticket_types()));
            provide_context(ContextEvent(store_value(event)));
            provide_context(ContextSlotDetails(store_value(slot_details)));

            view! { <Outlet/> }.into_view()
        }
        (_, _) => view! { <p>"Loading.."</p> }.into_view(),
    }
}

#[component]
pub fn ListBookings() -> impl IntoView {
    let event = expect_context::<ContextEvent>().0;

    let bookings_res =
        create_resource(
            || {},
            move |_| async move {
                match booking::list_bookings(event().id.clone()).await {
                    Ok(bookings) => bookings,
                    Err(e) => {
                        warn!("Error listing bookings: {}", e);
                        Default::default()
                    }
                }
            },
        );
    let bookings = move || bookings_res.get().unwrap_or_default();
    let ticket_types = move || event().ticket_types();

    let total_tickets = move || {
        let mut totals: HashMap<String, usize> = HashMap::new();
        for booking in bookings() {
            for ticket in booking.tickets {
                *totals.entry(ticket.ticket_type.name).or_insert(0) += 1;
            }
        }
        totals
    };

    let total_ticket_value = move || {
        bookings()
            .iter()
            .fold(Decimal::ZERO, |a, b| a + b.total_ticket_value())
    };

    let total_paid = move || {
        bookings()
            .iter()
            .fold(Decimal::ZERO, |a, b| a + b.total_paid())
    };

    let event_name = move || event().name.clone();

    view! {
      <section class="section">
        <div class="container">
          <h1 class="title">Bookings for {event_name}</h1>
          <table class="table">
            <thead>
              <tr>
                <th>Contact</th>
                <For each=ticket_types key=move |tt| tt.name.clone() let:tt>
                  <th>{tt.name} Tickets</th>
                </For>
                <th>Slots</th>
                <th>Order Value</th>
                <th>Payment Recieved</th>
              </tr>
            </thead>
            <tbody>
              <For each=bookings key=move |b| b.id.clone() let:booking>
                <tr>
                  <td>
                    <a href=format!("/booking/{}", booking.id.clone())>{booking.contact.full_name()}</a>
                  </td>
                  {ticket_types()
                      .iter()
                      .map(|tt| { booking.tickets.iter().filter(|t| t.ticket_type == *tt).count() })
                      .map(|n| view! { <td class="has-text-right">{n}</td> })
                      .collect_view()}
                  <td class="has-text-right">
                    {booking
                        .tickets
                        .iter()
                        .map(|t| t.slot_name.as_ref().unwrap_or(&"none".to_string()).clone())
                        .sorted()
                        .dedup()
                        .collect::<Vec<String>>()
                        .join(", ")}
                  </td>
                  <td class="has-text-right">{format!("£{}", booking.total_ticket_value())}</td>
                  <td class="has-text-right">{format!("£{}", booking.total_paid())}</td>
                </tr>
              </For>
            </tbody>
            <tfoot>
              <tr>
                <td>Totals:</td>
                <For each=ticket_types key=move |tt| tt.name.clone() let:tt>
                  <td class="has-text-right">{move || total_tickets().get(tt.name.as_str()).cloned().unwrap_or(0)}</td>
                </For>
                <td></td>
                <td class="has-text-right">{move || format!("£{}", total_ticket_value())}</td>
                <td class="has-text-right">{move || format!("£{}", total_paid())}</td>
              </tr>
            </tfoot>
          </table>
          <Outlet/>
        </div>
      </section>
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

// TODO - what is this for? extracting MaybePersonSignal into stored value?
#[component]
pub fn NewBooking() -> impl IntoView {
    require_login();
    let person = expect_context::<MaybePersonSignal>();

    move || match person.get() {
        None => view! { <p>"Loading"</p> }.into_view(),
        Some(p) => {
            let sp = store_value(p);
            view! { <NewBookingForPerson person=sp/> }.into_view()
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
pub fn GeneratePaymentLink() -> impl IntoView {
    let params = use_params_map();
    let booking_id = move || -> BookingId {
        params.with(|p| p.get("booking_id").cloned().unwrap_or_default().into())
    };

    let status = create_resource(booking_id, |id| async move {
        booking::create_payment_link(
            id.clone(),
            url_for_path(format!("booking/{}/check_payment", &id)),
        )
        .await
    });

    create_effect(move |_| {
        if let Some(Ok(res)) = status.get() {
            web_sys::window()
                .unwrap()
                .location()
                .replace(res.as_ref())
                .unwrap();
        }
    });

    move || match status.get() {
        None => view! { <div class="block">Generating Payment Link.. <Loader/></div> }.into_view(),
        Some(Err(e)) => {
            warn!("error generating payment link: {:?}", e);
            notify_details(
                "Error generating payment link",
                e.to_string(),
                Color::Danger,
            )
        }
        Some(Ok(_)) => notify("Link generated, redirecting..", Color::Success),
    }
}

#[component]
pub fn NewBookingForPerson(#[prop(into)] person: Signal<Person>) -> impl IntoView {
    let event = expect_context::<ContextEvent>().0;
    let full_name = Signal::derive(move || person.get().full_name());

    let event_name = event().name.clone();
    let event_tagline = event().tagline.clone();

    let default_tt = event().default_ticket_type.clone();
    let tickets = vec![Ticket::new(default_tt)];
    let tickets: ReactiveList<Ticket> = tickets.into();
    let tickets = create_rw_signal(tickets);

    let slots = expect_context::<ContextSlotDetails>().0;

    let add_ticket = move || {
        let mut nt = Ticket::new(event().default_ticket_type.clone());

        if let Some((_, last_ticket)) = tickets().iter().last() {
            if let Some(slot_name) = last_ticket().slot_name.as_deref() {
                if slot_state_for_ticket(slots, slot_name, tickets(), None).can_take() {
                    nt.slot_name = Some(slot_name.to_string());
                }
            }
        }
        tickets.tracked_push(nt);
    };

    let create_booking = create_server_action::<CreateBooking>();
    let pending = create_booking.pending();

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
                    format!("/booking/{}/generate_payment_link", res.id).as_ref(),
                    Default::default(),
                )
            }
        })
    });

    let validation_errors = move || {
        tickets()
            .iter()
            .enumerate()
            .filter(|(_, (_, t))| t().slot_name.is_none())
            .map(|(i, (_, _))| format!("Ticket {} has not been assigned to a slot", i + 1))
            .collect::<Vec<String>>()
    };

    let disabled = Signal::derive(move || pending() | !validation_errors().is_empty());

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
            <TicketForm tickets=tickets/>
            <div class="field is-grouped is-flex-wrap-wrap">
              <p class="control">
                <IconButton icon=i::FaPlusSolid on_click=add_ticket>
                  "Add Another Ticket"
                </IconButton>
              </p>
              <p class="control">
                <IconButton
                  icon=i::FaBasketShoppingSolid
                  // color=pay_btn_color
                  on_click=on_submit
                  disabled=disabled
                  loading=pending
                >

                  Pay Now
                </IconButton>
                <p class="is-danger">{move || validation_errors().join(", ")}</p>
              </p>

            </div>

            <Outlet/>

          </div>
        </div>
      </section>
    }
}

#[component]
pub fn TicketForm(tickets: RwSignal<ReactiveList<Ticket>>) -> impl IntoView {
    move || {
        tickets.with(|gl| {
            gl.iter()
                .enumerate()
                .map(|(i, (&uid, &ticket))| {
                    if i == 0 {
                        view! {
                          <Field label=move || format!("Ticket {}", { i + 1 })>
                            <TicketControl ticket=ticket tickets=tickets/>
                          </Field>
                        }
                    } else {
                        view! {
                          <Field label=move || {
                              view! {
                                {format!("Ticket {}", { i + 1 })}
                                <br/>
                                <IconButton on_click=move || tickets.tracked_remove(uid) icon=i::FaTrashSolid/>
                              }
                          }>
                            <TicketControl ticket=ticket tickets=tickets/>
                          </Field>
                        }
                    }
                })
                .collect_view()
        })
    }
}

