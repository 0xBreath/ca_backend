use crate::Price;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceListResponse {
    pub invoices: Vec<Invoice>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub version: u64,
    pub location_id: String,
    pub order_id: String,
    pub payment_requests: Vec<PaymentRequest>,
    pub invoice_number: String,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_at: Option<String>,
    pub status: String,
    pub timezone: String,
    pub created_at: String,
    pub updated_at: String,
    pub primary_recipient: Recipient,
    pub accepted_payment_methods: PaymentMethod,
    pub delivery_method: String,
    pub sale_or_service_date: Option<String>,
    pub public_url: Option<String>,
    pub store_payment_method_enabled: bool,
    pub subscription_id: Option<String>,
    pub next_payment_amount_money: Option<Price>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub bank_account: bool,
    pub buy_now_pay_later: bool,
    pub card: bool,
    pub cash_app_pay: bool,
    pub square_gift_card: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub automatic_payment_source: String,
    pub card_id: String,
    pub computed_amount_money: Price,
    pub due_date: String,
    pub request_type: String,
    pub tipping_enabled: bool,
    pub total_completed_amount_money: Price,
    pub uid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    pub customer_id: String,
    pub email_address: String,
    pub family_name: String,
    pub given_name: String,
    pub phone_number: String,
}

// ========================= Invoice Webhook Payment Made =========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceWebhookResponse {
    pub merchant_id: String,
    pub location_id: String,
    pub event_id: String,
    pub created_at: String,
    pub data: PaymentMadeData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMadeData {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: String,
    pub object: PaymentMadeDataObject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMadeDataObject {
    pub invoice: Invoice,
}
