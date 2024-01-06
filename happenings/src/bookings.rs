use crate::people::PersonID;
use derive_builder::Builder;
use happenings_macro::generate_new;
use leptos::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use {crate::people::Person, crate::tickets::{Ticket, TicketType}, crate::AppState};

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
pub async fn create_booking(e: NewBooking) -> Result<String, ServerFnError> {
    backend::create(e).await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::*;
    use crate::{db, AppState};
    use leptos::ServerFnError::{self, ServerError};

    pub async fn create(e: NewBooking) -> Result<String, ServerFnError> {
        let app_state =
            use_context::<AppState>().ok_or(ServerError("No server state".to_string()))?;

        let r: db::Record = app_state
            .db
            .create("booking")
            .content(e)
            .await?
            .pop()
            .ok_or(ServerError("failed to create new event".to_string()))?;
        return Ok(r.id.to_string());
    }
}

