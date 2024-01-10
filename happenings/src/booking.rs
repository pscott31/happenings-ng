use crate::person::PersonID;
use crate::ticket::Ticket;
use happenings_macro::{generate_db, generate_new, generate_new_db};
use leptos::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[generate_new]
#[generate_db]
#[generate_new_db]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    BankTransfer { amount: Decimal, reference: String },
}

#[leptos::server]
pub async fn create_booking(
    event: String,
    contact: String,
    tickets: Vec<Ticket>,
) -> Result<Booking, ServerFnError> {
    backend::create(event, contact, tickets).await
}

#[leptos::server]
pub async fn create_payment_link(
    booking_id: String,
    redirect_to: String,
) -> Result<String, ServerFnError> {
    backend::create_payment_link(booking_id, redirect_to).await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::*;
    use crate::event::{DbEvent, Event};
    use crate::person::{DbPerson, Person};
    use crate::square_api;
    use crate::AppState;
    use axum::extract::Host;
    use leptos::logging::warn;
    use leptos::ServerFnError::{self, ServerError};
    use phonenumber;
    use sanitizer::StringSanitizer;
    use surrealdb::sql::thing;
    use tracing::info;

    enum Fail {
        NoState,
        NoHost,
        InvalidID(String),
        DBError(surrealdb::Error),
        NotFound(String),
        SquareAPI(String),
    }

    impl From<Fail> for ServerFnError {
        fn from(f: Fail) -> Self {
            let msg = match f {
                Fail::NoState => "app state not found".to_string(),
                Fail::NoHost => "host not found".to_string(),
                Fail::InvalidID(id) => format!("invalid id '{}'", id),
                Fail::DBError(e) => format!("database error: {}", e.to_string()),
                Fail::NotFound(id) => format!("no record with id '{}'", id),
                Fail::SquareAPI(e) => format!("square api call failed: '{}'", e),
            };
            warn!("booking fail: {}", msg);
            ServerError(msg)
        }
    }
    pub async fn create(
        event: String,
        contact: String,
        tickets: Vec<Ticket>,
    ) -> Result<Booking, ServerFnError> {
        info!("creating draft booking for {:?}/{:?}", event, contact);

        let app_state =
            use_context::<AppState>().ok_or(ServerError("No server state".to_string()))?;

        let b = NewDbBooking {
            contact_id: thing(contact.as_ref())
                .map_err(|_| ServerError("invalid contact id".to_string()))?,
            event_id: thing(event.as_ref())
                .map_err(|_| ServerError("invalid event id".to_string()))?,
            tickets: tickets,
            status: Status::Draft,
            payments: Vec::new(),
        };

        let mut bs: Vec<DbBooking> = app_state
            .db
            .create("booking")
            .content(b)
            .await
            .map_err(|e| ServerError(format!("failed to create new booking: {}", e.to_string())))?;

        let b = bs
            .pop()
            .ok_or(ServerError("failed to create new booking".to_string()))?;
        return Ok(b.into());
    }

    pub async fn create_payment_link(
        booking_id: String,
        redirect_to: String,
    ) -> Result<String, ServerFnError> {
        info!("creating payment link for booking: {:?}", booking_id);
        let app_state = use_context::<AppState>().ok_or(Fail::NoState)?;
        let host = use_context::<Host>().ok_or(Fail::NoHost)?;
        let id_thing =
            thing(booking_id.as_ref()).map_err(|_| Fail::InvalidID(booking_id.clone()))?;
        let booking: DbBooking = app_state
            .db
            .select(id_thing)
            .await
            .map_err(|e| Fail::DBError(e))?
            .ok_or(Fail::NotFound(booking_id))?;

        let event: DbEvent = app_state
            .db
            .select(&booking.event_id)
            .await
            .map_err(|e| Fail::DBError(e))?
            .ok_or(Fail::NotFound(booking.event_id.to_string()))?;

        let contact: DbPerson = app_state
            .db
            .select(&booking.contact_id)
            .await
            .map_err(|e| Fail::DBError(e))?
            .ok_or(Fail::NotFound(booking.contact_id.to_string()))?;
        let contact: Person = contact.into();

        let phone = match contact.phone.as_ref() {
            Some(phone_str) => {
                match phonenumber::parse(Some(phonenumber::country::Id::GB), phone_str) {
                    Ok(phone) => Some(phone.format().mode(phonenumber::Mode::E164).to_string()),
                    Err(_) => None,
                }
            }
            None => None,
        };

        let mut sanitizer = StringSanitizer::from(contact.full_name());
        sanitizer.trim().to_snake_case();
        let customer_id = sanitizer.get();

        let line_items = booking
            .tickets
            .iter()
            .map(|t| square_api::NewLineItem {
                quantity: "1".to_string(),
                catalog_version: t.ticket_type.square_catalog_version,
                catalog_object_id: t.ticket_type.square_item_id.clone(),
            })
            .collect::<Vec<_>>();

        let new_order = square_api::NewOrder {
            customer_id: Some(customer_id),
            location_id: app_state.config.square.location_id,
            line_items,
        };

        let req = square_api::CreatePaymentLinkRequest {
            idempotency_key: uuid::Uuid::new_v4().to_string(),
            description: event.name,
            order: new_order,
            checkout_options: Some(square_api::CheckoutOptions {
                allow_tipping: false,
                ask_for_shipping_address: false,
                enable_coupon: false,
                enable_loyalty: false,
                redirect_url: redirect_to,
            }),
            pre_populated_data: Some(square_api::PrePopulatedData {
                buyer_address: None,
                buyer_email: Some(contact.email),
                buyer_phone_number: phone,
            }),
        };

        let req = build_request("online-checkout/payment-links").json(&req);
        info!("request: {:?}", req);

        let res = req.send().await.map_err(|e| {
            warn!("failed to call square api: {}", e);
            e
        })?;

        if res.status().is_success() {
            let parsed_res = res.json::<square_api::Welcome>().await?;
            return Ok(parsed_res.payment_link.long_url);
        }

        let error_body = res.text().await?;
        Err(Fail::SquareAPI(error_body).into())
    }

    fn build_request(method: &str) -> reqwest::RequestBuilder {
        let app_state = use_context::<AppState>().unwrap();

        reqwest::Client::new()
            .post(format!("{}/{}", app_state.config.square.endpoint, method))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", app_state.config.square.api_key),
            )
    }
}

