use crate::event::{Event, EventId};
use crate::generic_id::Id;
use crate::person::{Person, PersonId};
use crate::schema::Schema;
use crate::ticket::Ticket;
use leptos::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub type BookingId = Id<Booking>;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Booking {
    pub id: BookingId,
    pub tickets: Vec<Ticket>,
    pub status: Status,
    pub payments: Vec<Payment>,
    pub square_order: Option<String>,
    pub contact: Person,
    pub event: Event,
}

impl Schema for Booking {
    const TABLE: &'static str = "booking";
}

impl Booking {
    pub fn total_paid(&self) -> Decimal {
        self.payments
            .iter()
            .fold(Decimal::new(0, 2), |a, p| a + p.amount())
    }

    pub fn total_ticket_value(&self) -> Decimal {
        self.tickets
            .iter()
            .fold(Decimal::new(0, 2), |a, t| a + t.ticket_type.price)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DbBooking {
    pub id: surrealdb::sql::Thing,
    pub tickets: Vec<Ticket>,
    pub status: Status,
    pub payments: Vec<Payment>,
    pub square_order: Option<String>,
    pub contact: crate::person::db::DbPerson,
    pub event: crate::event::DbEvent,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<DbBooking> for Booking {
    fn from(item: DbBooking) -> Self {
        Self {
            id: item.id.into(),
            contact: item.contact.into(),
            event: item.event.into(),
            tickets: item.tickets,
            status: item.status,
            payments: item.payments,
            square_order: item.square_order,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NewDbBooking {
    pub tickets: Vec<Ticket>,
    pub status: Status,
    pub payments: Vec<Payment>,
    pub square_order: Option<String>,
    pub contact_id: surrealdb::sql::Thing,
    pub event_id: surrealdb::sql::Thing,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum Status {
    #[default]
    Draft,
    Accepted,
    Paid,
    PartiallyPaid,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Payment {
    Cash { amount: Decimal, to: String },
    Card { amount: Decimal, reference: String },
    BankTransfer { amount: Decimal, reference: String },
}

impl Payment {
    pub fn amount(&self) -> Decimal {
        match self {
            Payment::Cash { amount, .. } => *amount,
            Payment::Card { amount, .. } => *amount,
            Payment::BankTransfer { amount, .. } => *amount,
        }
    }
}

#[leptos::server(endpoint = "get_booking")]
pub async fn get_booking(booking_id: BookingId) -> Result<Booking, ServerFnError> {
    backend::get(booking_id).await
}

#[leptos::server(endpoint = "list_bookings")]
pub async fn list_bookings(event_id: EventId) -> Result<Vec<Booking>, ServerFnError> {
    backend::list(event_id).await
}

#[leptos::server(endpoint = "create_booking")]
pub async fn create_booking(
    event: EventId,
    contact: PersonId,
    tickets: Vec<Ticket>,
) -> Result<Booking, ServerFnError> {
    backend::create(event, contact, tickets).await
}

#[leptos::server]
pub async fn create_payment_link(
    booking_id: BookingId,
    redirect_to: String,
) -> Result<String, ServerFnError> {
    backend::create_payment_link(booking_id, redirect_to).await
}

#[leptos::server(endpoint = "check_payment")]
pub async fn check_payment(booking_id: BookingId) -> Result<Booking, ServerFnError> {
    backend::check_payment(booking_id).await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use super::*;
    use crate::event::EventId;
    use crate::AppState;
    use crate::{square_api, surreal};
    use leptos::logging::warn;
    use leptos::ServerFnError::{self, ServerError};
    use phonenumber;
    use sanitizer::StringSanitizer;
    use surrealdb::opt::PatchOp;
    use surrealdb::sql::Thing;
    use tracing::info;

    enum Fail {
        NoState,
        DBError(surrealdb::Error),
        NotFound(String),
        SquareAPI(String),
        NoSquareOrder,
    }

    impl From<Fail> for ServerFnError {
        fn from(f: Fail) -> Self {
            let msg = match f {
                Fail::NoState => "app state not found".to_string(),
                Fail::DBError(e) => format!("database error: {}", e),
                Fail::NotFound(id) => format!("no record with id '{}'", id),
                Fail::SquareAPI(e) => format!("square api call failed: '{}'", e),
                Fail::NoSquareOrder => "no square order associated with booking".to_string(),
            };
            warn!("booking fail: {}", msg);
            ServerError(msg)
        }
    }

    pub async fn get(booking_id: BookingId) -> Result<Booking, ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(Fail::NoState)?;
        let mut bookings: Vec<DbBooking> = app_state
            .db
            .query(
                "SELECT contact_id AS contact,
                          event_id AS event,
                          *
                    FROM booking WHERE id=$id
                    FETCH contact, event",
            )
            .bind(("id", Thing::from(&booking_id)))
            .await
            .map_err(Fail::DBError)?
            .take(0)
            .map_err(Fail::DBError)?;

        let booking = bookings.pop().ok_or(Fail::NotFound(booking_id.into()))?;
        Ok(booking.into())
    }

    pub async fn list(event_id: EventId) -> Result<Vec<Booking>, ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(Fail::NoState)?;

        let bookings: Vec<DbBooking> = app_state
            .db
            .query("select contact_id AS contact, event_id AS event, * from booking where event_id=$event_id and status != 'Draft' FETCH contact, event")
            .bind(("event_id", Thing::from(&event_id)))
            .await
            .map_err(Fail::DBError)?
            .take(0)
            .map_err(Fail::DBError)?;

        Ok(bookings.into_iter().map(|booking| booking.into()).collect())
    }

    pub async fn create(
        event: EventId,
        contact: PersonId,
        tickets: Vec<Ticket>,
    ) -> Result<Booking, ServerFnError> {
        info!("creating draft booking for {:?}/{:?}", event, contact);

        let app_state = use_context::<AppState>().ok_or(ServerFnError::new("No server state"))?;

        let b = NewDbBooking {
            contact_id: contact.into(),
            event_id: event.into(),
            tickets,
            status: Status::Draft,
            payments: Vec::new(),
            square_order: None,
        };

        let mut bs: Vec<crate::surreal::Record> =
            app_state
                .db
                .create("booking")
                .content(b)
                .await
                .map_err(|e| ServerFnError::new(format!("failed to create new booking: {}", e)))?;

        let b = bs
            .pop()
            .ok_or(ServerFnError::new("failed to create new booking"))?;

        get(b.id.into()).await
    }

    pub async fn create_payment_link(
        booking_id: BookingId,
        redirect_to: String,
    ) -> Result<String, ServerFnError> {
        info!("creating payment link for booking: {:?}", booking_id);
        let app_state = use_context::<AppState>().ok_or(Fail::NoState)?;
        let booking = get(booking_id.clone()).await?;
        let contact = booking.contact;
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
            description: booking.event.name,
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

        let req = build_post_request("online-checkout/payment-links").json(&req);
        info!("request: {:?}", req);

        let res = req.send().await.map_err(|e| {
            warn!("failed to call square api: {}", e);
            e
        })?;

        if !res.status().is_success() {
            let error_body = res.text().await?;
            return Err(Fail::SquareAPI(error_body).into());
        }

        let parsed_res = res.json::<square_api::Welcome>().await?;

        let _: surreal::Record = app_state
            .db
            .update(booking.id)
            .patch(PatchOp::replace(
                "/square_order",
                parsed_res.payment_link.order_id,
            ))
            .await
            .map_err(Fail::DBError)?
            .ok_or(Fail::NotFound(booking_id.into()))?;

        Ok(parsed_res.payment_link.long_url)
    }

    pub async fn check_payment(booking_id: BookingId) -> Result<Booking, ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(Fail::NoState)?;

        let booking = get(booking_id.clone()).await?;

        // Call Square API and check status of payment on the order
        let order_id = booking.square_order.clone().ok_or(Fail::NoSquareOrder)?;
        let req = build_get_request(format!("orders/{}", order_id).as_ref());

        let res = req.send().await.map_err(|e| {
            warn!("failed to call square api: {}", e);
            e
        })?;

        if !res.status().is_success() {
            let error_body = res.text().await?;
            return Err(Fail::SquareAPI(error_body).into());
        }

        let parsed_res = res.json::<square_api::RetrieveOrderResponse>().await?;

        let payments: Vec<Payment> = parsed_res
            .order
            .tenders
            .unwrap_or_default()
            .iter()
            .map(|t| Payment::Card {
                amount: Decimal::new(t.amount_money.amount, 2),
                reference: t.payment_id.clone(),
            })
            .collect();

        let total_paid = payments
            .iter()
            .fold(Decimal::new(0, 2), |a, p| a + p.amount());

        let order_total = Decimal::new(parsed_res.order.total_money.amount, 2);

        let status = if (total_paid >= order_total) && booking.status != Status::Cancelled {
            Status::Paid
        } else if total_paid > Decimal::ZERO && booking.status != Status::Cancelled {
            Status::PartiallyPaid
        } else {
            booking.status
        };

        let _: surreal::Record = app_state
            .db
            .update(&booking.id)
            .patch(PatchOp::replace("/payments", payments))
            .patch(PatchOp::replace("/status", status))
            .await
            .map_err(Fail::DBError)?
            .ok_or(Fail::NotFound(booking.id.into()))?;

        get(booking_id.clone()).await
    }

    // TODO - common code between this guy and below
    fn build_post_request(method: &str) -> reqwest::RequestBuilder {
        let app_state = use_context::<AppState>().unwrap();

        reqwest::Client::new()
            .post(format!("{}/{}", app_state.config.square.endpoint, method))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", app_state.config.square.api_key),
            )
    }

    fn build_get_request(method: &str) -> reqwest::RequestBuilder {
        let app_state = use_context::<AppState>().unwrap();

        reqwest::Client::new()
            .get(format!("{}/{}", app_state.config.square.endpoint, method))
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", app_state.config.square.api_key),
            )
    }
}

