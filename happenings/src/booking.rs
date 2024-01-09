use crate::people::PersonID;
use crate::ticket::Ticket;
use derive_builder::Builder;
use happenings_macro::generate_new;
use leptos::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[generate_new]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Builder)]
pub struct Booking {
    pub id: String,
    pub event_id: String,
    pub contact_id: String,
    pub tickets: Vec<Ticket>,
    pub status: Status,
    pub payments: Vec<Payment>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum Status {
    #[default]
    Draft,
    Accepted,
    Paid,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Payment {
    Cash { amount: Decimal, to: PersonID },
    Card { amount: Decimal, reference: String },
    BankTransfer { amout: Decimal, reference: String },
}

#[leptos::server(CreateBooking, "/api", "Url", "create_booking")]
pub async fn create_booking(
    event: String,
    contact: String,
    tickets: Vec<Ticket>,
) -> Result<Booking, ServerFnError> {
    backend::create(event, contact, tickets).await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::*;
    use crate::{db, AppState};
    use leptos::ServerFnError::{self, ServerError};

    pub async fn create(
        event: String,
        contact: String,
        tickets: Vec<Ticket>,
    ) -> Result<Booking, ServerFnError> {
        let app_state =
            use_context::<AppState>().ok_or(ServerError("No server state".to_string()))?;

        let b = NewBooking {
            contact_id: contact,
            event_id: event,
            tickets: tickets,
            status: Status::Draft,
            payments: Vec::new(),
        };

        let mut bs: Vec<Booking> = app_state
            .db
            .create("booking")
            .content(b)
            .await
            .map_err(|e| ServerError(format!("failed to create new booking: {}", e.to_string())))?;

        let b = bs
            .pop()
            .ok_or(ServerError("failed to create new booking".to_string()))?;
        return Ok(b);
    }
}

