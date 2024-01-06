use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Ticket {
    pub ticket_type: TicketType,
    pub vegetarian: bool,
    pub gluten_free: bool,
    pub dietary_requirements: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TicketType {
    pub name: String,
    pub price: Decimal,
    pub square_item_id: String,
    pub square_catalog_version: i64,
}

pub type TicketTypes = Vec<TicketType>;

