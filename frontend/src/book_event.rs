use crate::app::MaybePersonSignal;
use crate::components::controls::*;
use crate::field::Field;
use crate::icon_button::{Color, IconButton};
use crate::reactive_list::{ReactiveList, TrackableList};
use happenings::booking::CreateBooking;
use happenings::events::{get_event, Event};
use happenings::people::Person;
use happenings::ticket::Ticket;
use leptos::*;
use leptos_icons::FaIcon::*;
use leptos_router::{use_params_map, ActionForm};
use log::*;

#[component]
pub fn BookingPage() -> impl IntoView {
    // let params = use_params_map();
    // let event_id = params.with(|p| p.get("id").cloned().unwrap_or_default());

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

#[component]
pub fn NewBooking(#[prop(into)] event: Signal<Event>) -> impl IntoView {
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
    let on_submit = move || {
        let booking = CreateBooking {
            event: event().id.clone(),
            contact: person().id.clone(),
            tickets: tickets().into(),
        };
        create_booking.dispatch(booking);
    };

    view! {
      <section class="section">
        <ActionForm action=create_booking>
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
                // <input type="submit" value="Pay Now"/>

                // {move || { if pending() { "Generating Link..." } else { "Proceed to Payment" } }}
                </p>
              </div>
            </div>
          </div>
        </ActionForm>
      </section>
    }

    // // let person = store_value(person);
    // // let event = store_value(event);
    // let ticket_types = store_value(event().ticket_types());
    // provide_context(ticket_types);

    // // let booking = NewBooking::(event().clone(), person());
    // // let booking = NewBookingBuilder::default()
    // //     .event_id(event().id)
    // //     .contact_id(person().id)
    // //     .tickets(vec![Ticket::new(event().standard_ticket.clone())])
    // //     .build();

    // let booking = NewBooking {
    //     event_id: event().id,
    //     contact_id: person().id,
    //     tickets: vec![Ticket::new(event().standard_ticket.clone())],
    //     status: booking::Status::Draft,
    //     payments: vec![],
    // };

    // let booking = create_rw_signal::<NewBooking>(booking);

    // let (tickets, set_tickets) = create_signal::<ReactiveList<Ticket>>(booking().tickets.into());
    // let (error_seen, set_error_seen) = create_signal::<usize>(0);

    // let badgers = move || {
    //     tickets.with(|gl| {
    //         debug!("recomuting badger");
    //         gl.iter()
    //             .enumerate()
    //             .map(|(i, (&uid, &gv))| {
    //                 if i == 0 {
    //                     view! {
    //                       <Field label=move || format!("Ticket {}", { i + 1 })>
    //                         <TicketControl ticket=gv/>
    //                       </Field>
    //                     }
    //                 } else {
    //                     view! {
    //                       <Field label=move || {
    //                           view! {
    //                             {format!("Ticket {}", { i + 1 })}
    //                             <br/>
    //                             <IconButton on_click=move || set_tickets.tracked_remove(uid) icon=FaTrashSolid/>
    //                           }
    //                       }>
    //                         <TicketControl ticket=gv/>
    //                       </Field>
    //                     }
    //                 }
    //             })
    //             .collect_view()
    //     })
    // };

    // let add_ticket = move || set_tickets.tracked_push(Ticket::new(event().standard_ticket.clone()));

    // let build_booking = move || {
    //     let tickets: Vec<Ticket> = tickets().into();
    //     NewBooking {
    //         tickets,
    //         ..booking()
    //     }
    // };

    // let create_order = create_action(move |_: &()| {
    //     let new_booking = build_booking();
    //     async move { create_booking(new_booking).await }
    // });

    // let create_order_pending = create_order.pending();
    // let create_order_value = create_order.value();
    // let create_order_text = move || match create_order_value() {
    //     Some(Ok(v)) => format!("Order Created: id: {} ", v),
    //     Some(Err(e)) => format!("Error Creating Order: {}", e.to_string()),
    //     None => "Pending..".to_string(),
    // };
    // let (create_error_seen, set_create_error_seen) = create_signal::<usize>(0);

    // let error_data = move || {
    //     create_order.value().with(|x| {
    //         if let Some(Err(err)) = x {
    //             Some(err.to_string())
    //         } else {
    //             None
    //         }
    //     })
    // };

    // // let _navigate_to_payment = create_effect(move |_| {
    // //     link_action.value().with(|x| {
    // //         if let Some(Ok(res)) = x {
    // //             let _ = window().location().set_href(res);
    // //         }
    // //     })
    // // });

    // // let validation = move || booking().validate();
    // // let is_invalid = Signal::derive(move || validation().is_err());
    // // let pending = link_action.pending();

    // view! {
    //   <section class="section">
    //     <div class="container">
    //       <h1 class="title">Little Stukeley Christmas Dinner</h1>
    //       <p class="subtitle">Get your tickets for the final village event of the year!</p>

    //       <div class="box">
    //         <Field label=|| "Booking Contact">
    //           <Name get=|| person().full_name() disabled=true/>
    //           <Email get=|| person().email disabled=true/>
    //         </Field>
    //         <Field>
    //           <PhoneNumber get=person().phone disabled=true/>
    //         </Field>
    //         // {badgers}

    //         <div class="field is-grouped is-flex-wrap-wrap">
    //           <p class="control">
    //             <IconButton icon=FaPlusSolid color=Color::Secondary on_click=add_ticket>
    //               "Add Another Ticket"
    //             </IconButton>
    //           </p>

    //         // <p class="control">
    //         // <IconButton
    //         // disabled=is_invalid
    //         // icon=FaBasketShoppingSolid
    //         // color=Color::Primary
    //         // on_click=move || create_order.dispatch(())
    //         // />
    //         // // {move || { if pending() { "Generating Link..." } else { "Proceed to Payment" } }}
    //         // </p>
    //         // <Show when=move || without_payment>
    //         // <p class="control">
    //         // <IconButton
    //         // disabled=is_invalid
    //         // icon=FaBasketShoppingSolid
    //         // color=Color::Primary
    //         // on_click=move || create_order.dispatch(())
    //         // >
    //         // {move || { if pending() { "Creating Order..." } else { "Create Order without Paying" } }}
    //         // </IconButton>
    //         // </p>
    //         // </Show>
    //         </div>
    //       </div>
    //     </div>

    //   // <Modal
    //   // active=move || error_data().is_some() && link_action.version()() != error_seen()
    //   // close_requested=move || set_error_seen(link_action.version()())
    //   // title="Oh dear"
    //   // footer=move || {
    //   // view! {}
    //   // }
    //   // >

    //   // <div class="block">Something went wrong trying to generate a payment link for you to buy your tickets.</div>
    //   // <div class="block">
    //   // Terribly sorry about that. Could you please let me (Phil Scott) know and tell me what it says below and I will get it sorted
    //   // </div>
    //   // <div class="block">
    //   // <pre style="white-space: pre-wrap;">{error_data}</pre>
    //   // </div>
    //   // </Modal>

    //   // <Modal
    //   // active=move || !create_order_pending() && create_order.version()() != create_error_seen()
    //   // close_requested=move || set_create_error_seen(create_order.version()())
    //   // title="Create Order Results"
    //   // footer=move || {
    //   // view! {}
    //   // }
    //   // >

    //   // <div class="block">Hi Sally</div>
    //   // <div class="block">
    //   // <pre style="white-space: pre-wrap;">{create_order_text}</pre>
    //   // </div>
    //   // </Modal>

    //   </section>
    // }
}

