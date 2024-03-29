use crate::reactive_list::ReactiveList;
use common::event::SlotDetail;
use common::ticket::Ticket;
use leptos::*;
use std::ops::Deref;

// The state of a particular slot with respect to a particular ticket
#[derive(Debug)]
pub enum SlotStateForTicket {
    InSlot { available: i64, buying: i64 }, // Ticket under consideration already in this slot
    SomeLeft { available: i64, buying: i64 },
    Unlimited,
    NoneLeft { buying: i64 },
    NotFound(String),
}

impl SlotStateForTicket {
    pub fn is_in_slot(&self) -> bool { matches!(self, Self::InSlot { .. }) }
    pub fn can_take(&self) -> bool {
        use SlotStateForTicket::*;
        match self {
            InSlot { .. } => true,
            SomeLeft { available, buying } => buying <= available,
            Unlimited => true,
            NoneLeft { .. } => false,
            NotFound { .. } => false,
        }
    }

    pub fn description(&self) -> String {
        use SlotStateForTicket::*;
        match self {
            InSlot { available, .. } if *available <= 0 => format!("No more available."),
            InSlot { available, .. } => format!("{available} more available."),
            SomeLeft { available, .. } => format!("{} available.", available),
            Unlimited => "".to_string(),
            NoneLeft { buying: 0 } => "Sold out.".to_string(),
            NoneLeft { .. } => format!("No more available."),
            NotFound(name) => format!("Error! slot {name} found."),
        }
    }
}

pub fn slot_state_for_ticket(
    slots: StoredValue<Vec<SlotDetail>>, // Details of the slots for the event
    slot: impl Deref<Target = str>,      // The name of the slot
    tickets: ReactiveList<Ticket>,       // All the tickets in the booking being created
    ticket: Option<RwSignal<Ticket>>,    // The ticket that this slot select is for
) -> SlotStateForTicket {
    use SlotStateForTicket::*;
    let name = slot.deref();

    let Some(details) = slots().into_iter().find(|s| s.name == name) else {
        return SlotStateForTicket::NotFound(name.to_string());
    };

    let Some(cap) = details.capacity else {
        return SlotStateForTicket::Unlimited;
    };

    let in_slot = |&t: &&RwSignal<Ticket>| t().slot_name.is_some_and(|sn| sn == name);
    let buying = tickets.values().filter(in_slot).count() as i64;
    let available = cap - details.sold - buying;

    // Signals compare equal when they wrap the same underlying value.
    let is_our_ticket = |t: &RwSignal<Ticket>| ticket.is_some_and(|ft| ft == *t);

    let already_there = tickets.values().filter(in_slot).any(is_our_ticket);
    match (already_there, available > 0) {
        (true, _) => InSlot { available, buying },
        (false, true) => SomeLeft { available, buying },
        (false, false) => NoneLeft { buying },
    }
}

