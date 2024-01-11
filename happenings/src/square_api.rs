use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Welcome {
    pub payment_link: PaymentLink,
    pub related_resources: RelatedResources,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentLink {
    pub id: String,
    pub version: i64,
    pub description: String,
    pub order_id: String,
    pub url: String,
    pub long_url: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckoutOptions {
    pub allow_tipping: bool,
    pub ask_for_shipping_address: bool,
    pub enable_coupon: bool,
    pub enable_loyalty: bool,
    pub redirect_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedResources {
    pub orders: Vec<Order>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentLinkRequest {
    pub idempotency_key: String,
    pub order: NewOrder,
    pub description: String,
    pub pre_populated_data: Option<PrePopulatedData>,
    pub checkout_options: Option<CheckoutOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub idempotency_key: String,
    pub order: NewOrder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub order: Order,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewOrder {
    pub location_id: String,
    pub customer_id: Option<String>,
    pub line_items: Vec<NewLineItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewLineItem {
    pub quantity: String,
    pub catalog_object_id: String,
    pub catalog_version: i64,
    // pub metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrePopulatedData {
    pub buyer_address: Option<Address>,
    pub buyer_email: Option<String>,
    pub buyer_phone_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    //todo if needed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub location_id: String,
    pub source: Source,
    pub line_items: Vec<LineItem>,
    pub fulfillments: Option<Vec<Fulfillment>>,
    pub net_amounts: NetAmounts,
    pub created_at: String,
    pub updated_at: String,
    pub state: String,
    pub version: i64,
    pub total_money: Money,
    pub total_tax_money: Money,
    pub total_discount_money: Money,
    pub total_tip_money: Money,
    pub total_service_charge_money: Money,
    pub net_amount_due_money: Money,
    pub tenders: Vec<Tender>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tender {
    pub amount_money: Money,
    pub id: String,
    pub payment_id: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetrieveOrderRequest {
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetrieveOrderResponse {
    pub order: Order,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fulfillment {
    pub uid: String,
    #[serde(rename = "type")]
    pub fulfillment_type: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineItem {
    pub uid: String,
    pub name: String,
    pub quantity: String,
    pub catalog_object_id: String,
    pub catalog_version: i64,
    pub variation_name: String,
    pub item_type: String,
    pub base_price_money: Money,
    pub variation_total_price_money: Money,
    pub gross_sales_money: Money,
    pub total_tax_money: Money,
    pub total_discount_money: Money,
    pub total_money: Money,
    pub total_service_charge_money: Money,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Money {
    pub amount: i64,
    pub currency: Currency,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Currency {
    #[serde(rename = "GBP")]
    Gbp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetAmounts {
    pub total_money: Money,
    pub tax_money: Money,
    pub discount_money: Money,
    pub tip_money: Money,
    pub service_charge_money: Money,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub name: String,
}

