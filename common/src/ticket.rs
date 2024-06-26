use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Ticket {
    pub ticket_type: TicketType,
    pub vegetarian: bool,
    pub gluten_free: bool,
    pub dietary_requirements: String,
    pub slot_name: Option<String>,
}

impl Ticket {
    pub fn new(ticket_type: TicketType) -> Self {
        Self {
            ticket_type,
            vegetarian: false,
            gluten_free: false,
            dietary_requirements: "".to_string(),
            slot_name: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TicketType {
    pub name: String,
    pub price: Decimal,
    pub square_item_id: String,
    pub square_catalog_version: i64,
    pub available: Option<i64>,
}

pub type TicketTypes = Vec<TicketType>;

