use crate::components::controls::*;
use happenings_lib::ticket::Ticket;
use leptos::*;

#[component]
pub fn TicketControl(ticket: RwSignal<Ticket>) -> impl IntoView {
    let tt = Signal::derive(move || ticket().ticket_type);
    let set_tt = move |new| ticket.update(|g| g.ticket_type = new);

    let veg = Signal::derive(move || ticket().vegetarian);
    let set_veg = move |new| ticket.update(|g| g.vegetarian = new);

    let gf = Signal::derive(move || ticket().gluten_free);
    let set_gf = move |new| ticket.update(|g| g.gluten_free = new);

    let reqs = Signal::derive(move || ticket().dietary_requirements);
    let set_reqs = move |new| ticket.update(|g| g.dietary_requirements = new);

    view! {
      <TicketType get=tt set=set_tt/>
      <Checkbox label="Vegetarian" get=veg set=set_veg/>
      <Checkbox label="Gluten Free" get=gf set=set_gf/>
      <Text placeholder="Other dietary requirements" get=reqs set=set_reqs/>
    }
}

